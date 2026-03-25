//! NT35510 register and command definitions.
//!
//! Reference: STMicroelectronics NT35510 driver (BSP_DISCO_F469NI),
//! Frida 3K138 DSI LCD Display, datasheet v0.80.

// Standard DCS commands
/// NOP command.
pub const NT35510_CMD_NOP: u8 = 0x00;
/// Software reset.
pub const NT35510_CMD_SWRESET: u8 = 0x01;
/// Read display ID.
pub const NT35510_CMD_RDDID: u8 = 0x04;
/// Sleep in.
pub const NT35510_CMD_SLPIN: u8 = 0x10;
/// Sleep out.
pub const NT35510_CMD_SLPOUT: u8 = 0x11;
/// Normal display mode on.
pub const NT35510_CMD_NORON: u8 = 0x13;
/// Display inversion off.
pub const NT35510_CMD_INVOFF: u8 = 0x20;
/// Display inversion on.
pub const NT35510_CMD_INVON: u8 = 0x21;
/// Display off.
pub const NT35510_CMD_DISPOFF: u8 = 0x28;
/// Display on.
pub const NT35510_CMD_DISPON: u8 = 0x29;
/// Column address set.
pub const NT35510_CMD_CASET: u8 = 0x2A;
/// Row address set.
pub const NT35510_CMD_RASET: u8 = 0x2B;
/// Memory write.
pub const NT35510_CMD_RAMWR: u8 = 0x2C;
/// Memory read.
pub const NT35510_CMD_RAMRD: u8 = 0x2E;
/// Tearing effect line off.
pub const NT35510_CMD_TEOFF: u8 = 0x34;
/// Tearing effect line on.
pub const NT35510_CMD_TEEON: u8 = 0x35;
/// Memory data access control.
pub const NT35510_CMD_MADCTL: u8 = 0x36;
/// Idle mode off.
pub const NT35510_CMD_IDMOFF: u8 = 0x38;
/// Idle mode on.
pub const NT35510_CMD_IDMON: u8 = 0x39;
/// Interface pixel format.
pub const NT35510_CMD_COLMOD: u8 = 0x3A;
/// Set tearing effect scan line.
pub const NT35510_CMD_STESL: u8 = 0x44;
/// Get scan line.
pub const NT35510_CMD_GSL: u8 = 0x45;
/// Write display brightness (0x00=off, 0xFF=max).
pub const NT35510_CMD_WRDISBV: u8 = 0x51;
/// Read display brightness.
pub const NT35510_CMD_RDDISBV: u8 = 0x52;
/// Write CTRL display (0x2C = BL on).
pub const NT35510_CMD_WRCTRLD: u8 = 0x53;
/// Write content adaptive brightness control.
pub const NT35510_CMD_WRCABC: u8 = 0x55;
/// Write CABC minimum brightness.
pub const NT35510_CMD_WRCABCMB: u8 = 0x5E;
/// Read ID1.
pub const NT35510_CMD_RDID1: u8 = 0xDA;
/// Read ID2.
pub const NT35510_CMD_RDID2: u8 = 0xDB;
/// Read ID3.
pub const NT35510_CMD_RDID3: u8 = 0xDC;
/// Enable command set extension.
pub const NT35510_CMD_SETEXTC: u8 = 0xF0;

// Proprietary register blocks (accessed via SETEXTC page switching)
pub const NT35510_CMD_B0: u8 = 0xB0;
pub const NT35510_CMD_B1: u8 = 0xB1;
pub const NT35510_CMD_B2: u8 = 0xB2;
pub const NT35510_CMD_B3: u8 = 0xB3;
pub const NT35510_CMD_B5: u8 = 0xB5;
pub const NT35510_CMD_B6: u8 = 0xB6;
pub const NT35510_CMD_B7: u8 = 0xB7;
pub const NT35510_CMD_B8: u8 = 0xB8;
pub const NT35510_CMD_B9: u8 = 0xB9;
pub const NT35510_CMD_BA: u8 = 0xBA;
pub const NT35510_CMD_BB: u8 = 0xBB;
pub const NT35510_CMD_BC: u8 = 0xBC;
pub const NT35510_CMD_BD: u8 = 0xBD;
pub const NT35510_CMD_BE: u8 = 0xBE;
pub const NT35510_CMD_BF: u8 = 0xBF;
pub const NT35510_CMD_CC: u8 = 0xCC;

// ID register expected values
/// Expected ID1 value (0x00).
pub const NT35510_ID1_EXPECTED: u8 = 0x00;
/// Expected ID2 value (0x80).
pub const NT35510_ID2_EXPECTED: u8 = 0x80;

// Pixel format values for COLMOD command
/// RGB565 (16-bit) pixel format.
pub const NT35510_COLMOD_RGB565: u8 = 0x55;
/// RGB888 (24-bit) pixel format.
pub const NT35510_COLMOD_RGB888: u8 = 0x77;

// MADCTL (Memory Data Access Control) bit definitions
/// Portrait: MY=0, MX=0, MV=0, ML=0.
pub const NT35510_MADCTL_PORTRAIT: u8 = 0x00;
/// Landscape: MY=0, MX=1, MV=1, ML=0 (same as OTM8009A pattern).
pub const NT35510_MADCTL_LANDSCAPE: u8 = 0x60;
/// BGR bit (bit 3) — swap red/blue channels.
pub const NT35510_MADCTL_BGR: u8 = 0x08;

// TE (Tearing Effect) configuration
/// TE output mode: vblanking info only.
pub const NT35510_TEEON_VBLANKING_INFO_ONLY: u8 = 0x00;
/// TE output mode: vblanking and hblanking info.
pub const NT35510_TEEON_VBLANKING_HBLANKING_INFO: u8 = 0x01;

// Display control
/// WRCTRLD value: backlight on.
pub const NT35510_WRCTRLD_BL_ON: u8 = 0x2C;
/// WRCTRLD value: backlight off.
pub const NT35510_WRCTRLD_BL_OFF: u8 = 0x00;
