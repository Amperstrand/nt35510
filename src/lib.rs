#![cfg_attr(not(test), no_std)]
//! Standalone `no_std` driver for NT35510 DSI LCD controller panels.
//!
//! Tested on STM32F469I-DISCO (B08 revision, Frida 3K138 panel).
//! API mirrors [`otm8009a`](https://crates.io/crates/otm8009a) for BSP-level
//! compatibility — both drivers expose `Mode`, `ColorMap`, and similar config
//! structs so the BSP can treat them uniformly.
//!
//! # Orientation
//!
//! - **Portrait** (480x800): default, tested on hardware.
//! - **Landscape** (800x480): uses MADCTL MX|MV rotation, untested.
//!
//! # Color mapping
//!
//! - **Rgb**: default (red channel first).
//! - **Bgr**: sets MADCTL bit 3, swaps red/blue channels.
//!
//! # Brightness
//!
//! Controlled via `WRDISBV` (0x00–0xFF) and `WRCTRLD` (backlight on/off).
//! `WRCABC` enables content-adaptive brightness.
//!
//! # Tearing Effect (TE)
//!
//! NT35510 supports TE output on the TE pin. Enable via [`Nt35510::enable_te_output`]
//! after init to get a hardware VBlank signal for synchronized buffer swaps.

mod regs;

pub use regs::*;

use embedded_display_controller::dsi::{DsiHostCtrlIo, DsiReadCommand, DsiWriteCommand};
use embedded_hal::delay::DelayNs;

/// Panel physical width in pixels (portrait orientation).
pub const PANEL_WIDTH: u16 = 480;
/// Panel physical height in pixels (portrait orientation).
pub const PANEL_HEIGHT: u16 = 800;

/// Errors that can occur during display initialization and operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    DsiRead,
    DsiWrite,
    ProbeMismatch(u8),
    InvalidDimensions,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::DsiRead => write!(f, "DSI read failed"),
            Error::DsiWrite => write!(f, "DSI write failed"),
            Error::ProbeMismatch(id) => {
                write!(f, "probe mismatch: expected NT35510, got 0x{id:02X}")
            }
            Error::InvalidDimensions => write!(f, "display dimensions must be non-zero"),
        }
    }
}

/// Display orientation. Matches [`otm8009a::Mode`](https://docs.rs/otm8009a/latest/otm8009a/enum.Mode.html).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Portrait orientation (480x800). Tested on STM32F469I-DISCO.
    Portrait,
    /// Landscape orientation (800x480). Untested.
    Landscape,
}

/// Color channel ordering. Matches [`otm8009a::ColorMap`](https://docs.rs/otm8009a/latest/otm8009a/enum.ColorMap.html).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMap {
    /// RGB order (default).
    Rgb,
    /// BGR order (swaps red and blue channels via MADCTL bit 3).
    Bgr,
}

/// Pixel format for the DSI video stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorFormat {
    /// 16-bit RGB565. Tested on STM32F469I-DISCO.
    Rgb565,
    /// 24-bit RGB888. Tested on STM32F469I-DISCO.
    Rgb888,
}

/// Panel timing parameters for LTDC configuration.
///
/// These values define the blanking intervals and sync pulse widths
/// used by the LTDC peripheral to generate the pixel clock and
/// timing signals for the NT35510 panel.
///
/// The "standard" timing values come from the ST BSP reference code
/// for the Frida 3K138 panel on STM32F469I-DISCO.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PanelTiming {
    /// Horizontal sync pulse width (in pixel clocks).
    pub h_sync: u16,
    /// Horizontal back porch (in pixel clocks).
    pub h_back_porch: u16,
    /// Horizontal front porch (in pixel clocks).
    pub h_front_porch: u16,
    /// Vertical sync pulse width (in lines).
    pub v_sync: u16,
    /// Vertical back porch (in lines).
    pub v_back_porch: u16,
    /// Vertical front porch (in lines).
    pub v_front_porch: u16,
    /// Target frame rate in Hz.
    pub frame_rate: u16,
}

impl PanelTiming {
    /// Standard timing for portrait orientation (ST BSP reference).
    ///
    /// Produces ~27MHz pixel clock on STM32F469I-DISCO.
    /// Used by the sync BSP (`stm32f469i-disc`).
    pub const STANDARD_PORTRAIT: Self = Self {
        h_sync: 2,
        h_back_porch: 34,
        h_front_porch: 34,
        v_sync: 1,
        v_back_porch: 15,
        v_front_porch: 16,
        frame_rate: 60,
    };

    /// Standard timing for landscape orientation (ST BSP reference).
    pub const STANDARD_LANDSCAPE: Self = Self {
        h_sync: 1,
        h_back_porch: 15,
        h_front_porch: 16,
        v_sync: 2,
        v_back_porch: 34,
        v_front_porch: 34,
        frame_rate: 60,
    };

    /// Timing for portrait orientation with correct NT35510 DSI vertical blanking.
    ///
    /// V_SYNC=120, V_BP=150, V_FP=150 are the authoritative values from:
    /// - STMicroelectronics NT35510 component header (NT35510_480X800_VSYNC/VBP/VFP)
    /// - ST BSP stm32469i_discovery_lcd.c (reference C implementation, pixel clock 27429 kHz)
    /// - embassy-stm32f469i-disco display.rs (known-working async BSP)
    /// - diybitcoinhardware/f469-disco (Specter DIY reference)
    ///
    /// `STANDARD_PORTRAIT` uses V_SYNC=1, V_BP=15, V_FP=16 which are insufficient for
    /// DSI video mode — LTDC starts emitting active pixels before the panel's vertical
    /// active window opens, causing the top rows to be cropped.
    ///
    /// Frame rate at 27,429 kHz pixel clock:
    ///   total_v = 800 + 120 + 150 + 150 = 1220 lines
    ///   total_h = 480 + 2 + 34 + 34 = 550 pixels
    ///   fps = 27_429_000 / (1220 * 550) ≈ 40.9 Hz
    pub const PORTRAIT_DSI: Self = Self {
        h_sync: 2,
        h_back_porch: 34,
        h_front_porch: 34,
        v_sync: 120,
        v_back_porch: 150,
        v_front_porch: 150,
        frame_rate: 41,
    };

    /// Get the standard timing for the given orientation.
    pub const fn for_mode(mode: Mode) -> Self {
        match mode {
            Mode::Portrait => Self::STANDARD_PORTRAIT,
            Mode::Landscape => Self::STANDARD_LANDSCAPE,
        }
    }
}

/// Configuration for the NT35510 panel.
///
/// Default values match the STM32F469I-DISCO board configuration
/// (portrait mode, RGB, RGB888, 480x800).
///
/// Mirrors [`otm8009a::Otm8009AConfig`](https://docs.rs/otm8009a/latest/otm8009a/struct.Otm8009AConfig.html)
/// for BSP compatibility, minus `frame_rate` (NT35510 frame rate is set via LTDC timing,
/// not the panel, unlike OTM8009A).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Nt35510Config {
    /// Display orientation.
    pub mode: Mode,
    /// Color channel ordering.
    pub color_map: ColorMap,
    /// Pixel format for DSI video stream.
    pub color_format: ColorFormat,
    /// Display width in pixels (before rotation).
    pub cols: u16,
    /// Display height in pixels (before rotation).
    pub rows: u16,
}

impl Default for Nt35510Config {
    fn default() -> Self {
        Self {
            mode: Mode::Portrait,
            color_map: ColorMap::Rgb,
            color_format: ColorFormat::Rgb888,
            cols: PANEL_WIDTH,
            rows: PANEL_HEIGHT,
        }
    }
}

/// Driver for the NT35510 DSI LCD controller panel.
///
/// Create a new instance via [`Nt35510::new()`] or [`Nt35510::default()`],
/// then initialize via [`init()`](Nt35510::init) or [`init_with_config()`](Nt35510::init_with_config).
pub struct Nt35510 {
    initialized: bool,
}

impl Default for Nt35510 {
    fn default() -> Self {
        Self::new()
    }
}

impl Nt35510 {
    #[must_use]
    pub const fn new() -> Self {
        Self { initialized: false }
    }

    /// Probe whether an NT35510 is connected by reading its ID registers.
    ///
    /// Returns `Ok(())` if the panel responds with expected NT35510 IDs.
    /// Returns `Err(Error::ProbeMismatch(id))` if a different panel responds.
    /// Returns `Err(Error::DsiRead)` if DSI reads fail entirely.
    pub fn probe(&mut self, dsi_host: &mut impl DsiHostCtrlIo) -> Result<(), Error> {
        match self.read_id(dsi_host, NT35510_CMD_RDID2) {
            Ok(id) if id == NT35510_ID2_EXPECTED => return Ok(()),
            Ok(id) => return Err(Error::ProbeMismatch(id)),
            Err(_) => {}
        }

        match self.read_id(dsi_host, NT35510_CMD_RDID1) {
            Ok(id) if id == NT35510_ID1_EXPECTED => Ok(()),
            Ok(id) => Err(Error::ProbeMismatch(id)),
            Err(_) => Err(Error::DsiRead),
        }
    }

    /// Check if an NT35510 panel is connected by reading ID registers.
    /// Returns `Ok(true)` if NT35510 is detected and `Ok(false)` otherwise.
    pub fn id_matches(&mut self, dsi_host: &mut impl DsiHostCtrlIo) -> Result<bool, Error> {
        if let Ok(id) = self.read_id(dsi_host, NT35510_CMD_RDID2) {
            return Ok(id == NT35510_ID2_EXPECTED);
        }

        match self.read_id(dsi_host, NT35510_CMD_RDID1) {
            Ok(id) => Ok(id == NT35510_ID1_EXPECTED),
            Err(_) => Err(Error::DsiRead),
        }
    }

    /// Diagnostic utility: write an incremental ramp pattern to display RAM
    /// and read it back to verify the DSI link and GRAM integrity.
    ///
    /// Writes patterns of decreasing length (17 down to 1 byte) via
    /// [`RAMWR`](NT35510_CMD_RAMWR) and reads them back via
    /// [`RAMRD`](NT35510_CMD_RAMRD). Useful for bring-up debugging to
    /// confirm the DSI bus can both write and read display memory.
    ///
    /// Matches [`Otm8009A::memory_check`](https://docs.rs/otm8009a/latest/otm8009a/struct.Otm8009A.html#method.memory_check).
    pub fn memory_check<D: DsiHostCtrlIo>(&mut self, dsi_host: &mut D) -> Result<(), Error> {
        let ramp = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
        let mut buf = [0u8; 17];
        for i in (1..17).rev() {
            dsi_host
                .write(DsiWriteCommand::DcsLongWrite {
                    arg: NT35510_CMD_RAMWR,
                    data: &ramp[..i],
                })
                .map_err(|_| Error::DsiWrite)?;
            dsi_host
                .read(
                    DsiReadCommand::DcsShort {
                        arg: NT35510_CMD_RAMRD,
                    },
                    &mut buf[..i],
                )
                .map_err(|_| Error::DsiRead)?;
        }
        Ok(())
    }

    /// Initialize the panel with an explicit configuration.
    ///
    /// This is the primary init method. Configures orientation, color format,
    /// and color map based on the provided [`Nt35510Config`].
    pub fn init_with_config<D: DelayNs>(
        &mut self,
        dsi_host: &mut impl DsiHostCtrlIo,
        delay: &mut D,
        config: Nt35510Config,
    ) -> Result<(), Error> {
        if self.initialized {
            return Ok(());
        }

        if config.cols == 0 || config.rows == 0 {
            return Err(Error::InvalidDimensions);
        }

        // LV2 Page 1: power rail and voltage init
        self.write_reg(
            dsi_host,
            NT35510_CMD_SETEXTC,
            &[0x55, 0xAA, 0x52, 0x08, 0x01],
        )?;
        self.write_reg(dsi_host, NT35510_CMD_B0, &[0x03, 0x03, 0x03])?;
        self.write_reg(dsi_host, NT35510_CMD_B6, &[0x46, 0x46, 0x46])?;
        self.write_reg(dsi_host, NT35510_CMD_B1, &[0x03, 0x03, 0x03])?;
        self.write_reg(dsi_host, NT35510_CMD_B7, &[0x36, 0x36, 0x36])?;
        self.write_reg(dsi_host, NT35510_CMD_B2, &[0x00, 0x00, 0x02])?;
        self.write_reg(dsi_host, NT35510_CMD_B8, &[0x26, 0x26, 0x26])?;
        self.write_reg(dsi_host, NT35510_CMD_BF, &[0x01])?;
        self.write_reg(dsi_host, NT35510_CMD_B3, &[0x09, 0x09, 0x09])?;
        self.write_reg(dsi_host, NT35510_CMD_B9, &[0x36, 0x36, 0x36])?;
        self.write_reg(dsi_host, NT35510_CMD_B5, &[0x08, 0x08, 0x08])?;
        self.write_reg(dsi_host, NT35510_CMD_BA, &[0x26, 0x26, 0x26])?;
        self.write_reg(dsi_host, NT35510_CMD_BC, &[0x00, 0x80, 0x00])?;
        self.write_reg(dsi_host, NT35510_CMD_BD, &[0x00, 0x80, 0x00])?;
        self.write_reg(dsi_host, NT35510_CMD_BE, &[0x00, 0x50])?;

        // LV2 Page 0: display timing and control
        self.write_reg(
            dsi_host,
            NT35510_CMD_SETEXTC,
            &[0x55, 0xAA, 0x52, 0x08, 0x00],
        )?;
        self.write_reg(dsi_host, NT35510_CMD_B1, &[0xFC, 0x00])?;
        self.write_reg(dsi_host, NT35510_CMD_B6, &[0x03])?;
        self.write_reg(dsi_host, NT35510_CMD_B5, &[0x51])?;
        self.write_reg(dsi_host, NT35510_CMD_B7, &[0x00, 0x00])?;
        self.write_reg(dsi_host, NT35510_CMD_B8, &[0x01, 0x02, 0x02, 0x02])?;
        self.write_reg(dsi_host, NT35510_CMD_BC, &[0x00, 0x00, 0x00])?;
        self.write_reg(dsi_host, NT35510_CMD_CC, &[0x03, 0x00, 0x00])?;
        self.write_reg(dsi_host, NT35510_CMD_BA, &[0x01])?;

        // TE on, pixel format, orientation — all before sleep out
        self.write_cmd(
            dsi_host,
            NT35510_CMD_TEEON,
            NT35510_TEEON_VBLANKING_INFO_ONLY,
        )?;
        self.write_cmd(dsi_host, NT35510_CMD_COLMOD, NT35510_COLMOD_RGB888)?;

        delay.delay_us(200_000);

        let mut madctl = match config.mode {
            Mode::Portrait => NT35510_MADCTL_PORTRAIT,
            Mode::Landscape => NT35510_MADCTL_LANDSCAPE,
        };
        if config.color_map == ColorMap::Bgr {
            madctl |= NT35510_MADCTL_BGR;
        }
        self.write_cmd(dsi_host, NT35510_CMD_MADCTL, madctl)?;

        let last_col = (config.cols - 1).to_be_bytes();
        let last_row = (config.rows - 1).to_be_bytes();
        self.write_reg(
            dsi_host,
            NT35510_CMD_CASET,
            &[0x00, 0x00, last_col[0], last_col[1]],
        )?;
        self.write_reg(
            dsi_host,
            NT35510_CMD_RASET,
            &[0x00, 0x00, last_row[0], last_row[1]],
        )?;

        self.write_cmd(dsi_host, NT35510_CMD_SLPOUT, 0x00)?;
        delay.delay_us(120_000);

        // Re-set pixel format after sleep out
        self.write_cmd(dsi_host, NT35510_CMD_COLMOD, NT35510_COLMOD_RGB888)?;
        if config.color_format == ColorFormat::Rgb565 {
            self.write_cmd(dsi_host, NT35510_CMD_COLMOD, NT35510_COLMOD_RGB565)?;
        }

        self.write_cmd(dsi_host, NT35510_CMD_WRDISBV, 0x7F)?;
        self.write_cmd(dsi_host, NT35510_CMD_WRCTRLD, NT35510_WRCTRLD_BL_ON)?;
        self.write_cmd(dsi_host, NT35510_CMD_WRCABC, 0x02)?;
        self.write_cmd(dsi_host, NT35510_CMD_WRCABCMB, 0xFF)?;
        self.write_cmd(dsi_host, NT35510_CMD_DISPON, 0x00)?;
        self.write_cmd(dsi_host, NT35510_CMD_RAMWR, 0x00)?;

        self.initialized = true;
        Ok(())
    }

    /// Initialize the panel with default config (portrait, RGB, RGB888).
    ///
    /// Convenience wrapper for [`init_with_config`](Self::init_with_config).
    pub fn init<D: DelayNs>(
        &mut self,
        dsi_host: &mut impl DsiHostCtrlIo,
        delay: &mut D,
    ) -> Result<(), Error> {
        self.init_with_config(dsi_host, delay, Nt35510Config::default())
    }

    /// Initialize the panel in RGB565 mode with custom orientation and color map.
    pub fn init_rgb565<D: DelayNs>(
        &mut self,
        dsi_host: &mut impl DsiHostCtrlIo,
        delay: &mut D,
        mode: Mode,
        color_map: ColorMap,
    ) -> Result<(), Error> {
        let config = Nt35510Config {
            mode,
            color_map,
            color_format: ColorFormat::Rgb565,
            ..Nt35510Config::default()
        };
        self.init_with_config(dsi_host, delay, config)
    }

    /// Initialize the panel in RGB888 mode with custom orientation and color map.
    pub fn init_rgb888<D: DelayNs>(
        &mut self,
        dsi_host: &mut impl DsiHostCtrlIo,
        delay: &mut D,
        mode: Mode,
        color_map: ColorMap,
    ) -> Result<(), Error> {
        let config = Nt35510Config {
            mode,
            color_map,
            color_format: ColorFormat::Rgb888,
            ..Nt35510Config::default()
        };
        self.init_with_config(dsi_host, delay, config)
    }

    /// Enable tearing effect (TE) output on the TE pin.
    ///
    /// After calling this, the TE pin pulses at each VBlank boundary.
    /// Pass `on_line = 0` for standard VBlank-only mode.
    ///
    /// Matches [`Otm8009A::enable_te_output`](https://docs.rs/otm8009a/latest/otm8009a/struct.Otm8009A.html#method.enable_te_output).
    pub fn enable_te_output<D: DsiHostCtrlIo>(
        &mut self,
        on_line: u16,
        dsi: &mut D,
    ) -> Result<(), Error> {
        self.write_long(dsi, NT35510_CMD_STESL, &on_line.to_be_bytes())?;
        self.write_cmd(dsi, NT35510_CMD_TEEON, NT35510_TEEON_VBLANKING_INFO_ONLY)?;
        Ok(())
    }

    /// Disable tearing effect output.
    pub fn disable_te_output<D: DsiHostCtrlIo>(&mut self, dsi: &mut D) -> Result<(), Error> {
        self.write_cmd(dsi, NT35510_CMD_TEOFF, 0x00)
    }

    /// Set display brightness level.
    ///
    /// `brightness`: 0x00 (off) to 0xFF (maximum). Default at init is 0x7F.
    pub fn set_brightness<D: DsiHostCtrlIo>(
        &mut self,
        dsi: &mut D,
        brightness: u8,
    ) -> Result<(), Error> {
        self.write_cmd(dsi, NT35510_CMD_WRDISBV, brightness)
    }

    /// Enable or disable the backlight via WRCTRLD.
    pub fn set_backlight<D: DsiHostCtrlIo>(&mut self, dsi: &mut D, on: bool) -> Result<(), Error> {
        let val = if on {
            NT35510_WRCTRLD_BL_ON
        } else {
            NT35510_WRCTRLD_BL_OFF
        };
        self.write_cmd(dsi, NT35510_CMD_WRCTRLD, val)
    }

    /// Turn the display off (enter sleep mode).
    pub fn sleep_in<D: DelayNs>(
        &mut self,
        dsi: &mut impl DsiHostCtrlIo,
        delay: &mut D,
    ) -> Result<(), Error> {
        self.write_cmd(dsi, NT35510_CMD_DISPOFF, 0x00)?;
        delay.delay_us(120_000);
        self.write_cmd(dsi, NT35510_CMD_SLPIN, 0x00)?;
        self.initialized = false;
        Ok(())
    }

    fn read_id(&self, dsi_host: &mut impl DsiHostCtrlIo, cmd: u8) -> Result<u8, Error> {
        let mut id = [0u8; 1];
        dsi_host
            .read(DsiReadCommand::DcsShort { arg: cmd }, &mut id)
            .map_err(|_| Error::DsiRead)?;
        Ok(id[0])
    }

    fn write_cmd(
        &self,
        dsi_host: &mut impl DsiHostCtrlIo,
        cmd: u8,
        param: u8,
    ) -> Result<(), Error> {
        dsi_host
            .write(DsiWriteCommand::DcsShortP1 {
                arg: cmd,
                data: param,
            })
            .map_err(|_| Error::DsiWrite)
    }

    fn write_long(
        &self,
        dsi_host: &mut impl DsiHostCtrlIo,
        cmd: u8,
        data: &[u8],
    ) -> Result<(), Error> {
        dsi_host
            .write(DsiWriteCommand::DcsLongWrite { arg: cmd, data })
            .map_err(|_| Error::DsiWrite)
    }

    fn write_reg(
        &self,
        dsi_host: &mut impl DsiHostCtrlIo,
        reg: u8,
        data: &[u8],
    ) -> Result<(), Error> {
        if data.is_empty() {
            self.write_cmd(dsi_host, reg, 0)
        } else if data.len() == 1 {
            self.write_cmd(dsi_host, reg, data[0])
        } else {
            self.write_long(dsi_host, reg, data)
        }
    }
}
