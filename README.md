[![crates.io](https://img.shields.io/crates/v/nt35510.svg)](https://crates.io/crates/nt35510)
[![docs.rs](https://docs.rs/nt35510/badge.svg)](https://docs.rs/nt35510)
[![License](https://img.shields.io/badge/license-0BSD-blue.svg)](LICENSE)

# nt35510

`no_std` driver for NT35510 DSI LCD controller panels.

Transport-agnostic — accepts any DSI host implementing
`embedded_display_controller::dsi::DsiHostCtrlIo`.

Tested on STM32F469I-Discovery (B08 revision, Frida 3K138 panel).

## Installation

```toml
[dependencies]
nt35510 = "0.2"
```

This crate depends on [`embedded-display-controller`](https://crates.io/crates/embedded-display-controller) v0.2 for the `DsiHostCtrlIo` trait. Add it to your dependencies as well:

```toml
[dependencies]
embedded-display-controller = "0.2"
```

## Supported Hardware

The NT35510 is a MIPI DSI display controller used in various TFT LCD panels.

Verified hardware:

- **STM32F469I-Discovery** (B08 revision, Frida 3K138 panel, 480x800 portrait)

Other STM32 MCUs with a DSI host peripheral should work with the appropriate HAL implementation of `DsiHostCtrlIo`.

## Features

- Portrait (480x800) and landscape (800x480) orientation
- RGB565 and RGB888 pixel formats
- RGB/BGR color channel ordering
- Panel probe via ID register read
- Brightness and backlight control
- Tearing effect (TE) output for VSync
- Sleep in/out for power management

## Usage

```rust
use embedded_display_controller::dsi::DsiHostCtrlIo;
use embedded_hal::delay::DelayNs;
use nt35510::{ColorFormat, Mode, Nt35510, Nt35510Config};

fn init_display(dsi: &mut impl DsiHostCtrlIo, delay: &mut impl DelayNs) {
    let mut panel = Nt35510::new();
    let _ = panel.probe(dsi, delay);

    let config = Nt35510Config {
        mode: Mode::Portrait,
        color_format: ColorFormat::Rgb888,
        ..Nt35510Config::default()
    };
    panel.init_with_config(dsi, delay, config).unwrap();
}
```

### Convenience wrappers

```rust
panel.init(dsi, delay)?;                        // default: portrait, RGB565
panel.init_rgb565(dsi, delay, mode, color_map)?; // RGB565 with custom orientation
panel.init_rgb888(dsi, delay, mode, color_map)?; // RGB888 with custom orientation
```

### After init

```rust
panel.set_brightness(dsi, 0xFF)?;    // max brightness
panel.set_backlight(dsi, true)?;     // backlight on
panel.enable_te_output(dsi, 0)?;     // TE on VBlank only
panel.sleep_in(dsi, delay)?;         // enter sleep mode
```

## Timing Configuration

Two portrait timing variants are provided:

| Constant | V_SYNC | V_BP | V_FP | Frame Rate | Use Case |
|----------|--------|------|------|------------|----------|
| `STANDARD_PORTRAIT` | 1 | 15 | 16 | ~60 Hz | RGB565, legacy |
| `PORTRAIT_DSI` | 120 | 150 | 150 | ~41 Hz | ARGB8888, DSI video mode |

### Why two variants?

`STANDARD_PORTRAIT` values (V_SYNC=1/V_BP=15/V_FP=16) are insufficient for DSI video
mode — the LTDC starts emitting active pixels before the panel's vertical active window
opens, causing top rows to be cropped and DSI/LTDC timing desynchronization artifacts.

`PORTRAIT_DSI` uses the authoritative values from the STMicroelectronics NT35510 component
driver header (`NT35510_480X800_VSYNC`/`VBP`/`VFP`), verified across four independent
implementations:

- STMicroelectronics NT35510 component header (`nt35510.h`)
- ST BSP (`stm32469i_discovery_lcd.c`, pixel clock 27,429 kHz)
- embassy-stm32f469i-disco (`display.rs`, known-working async BSP)
- Specter DIY (`f469-disco`, MicroPython reference)

### Critical rule: DSI and LTDC must use the same timing

When configuring DSI video mode, both the DSI host **and** the LTDC must receive identical
timing configuration. A mismatch (e.g. PORTRAIT_DSI for DSI but STANDARD_PORTRAIT for LTDC)
causes the LTDC background color to bleed through as visible artifacts on the right side of
the display, because the two peripherals disagree on frame boundaries.

### Frame rate

At 27,429 kHz pixel clock with `PORTRAIT_DSI`:
```
total_v = 800 + 120 + 150 + 150 = 1220 lines
total_h = 480 + 2 + 34 + 34 = 550 pixels
fps = 27,429,000 / (1220 × 550) ≈ 40.9 Hz
```

## API compatibility

This crate mirrors the [`otm8009a`](https://crates.io/crates/otm8009a) API for BSP-level
compatibility. Both drivers expose `Mode`, `ColorMap`, and similar config structs.

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/nt35510).

## Architecture

```mermaid
flowchart LR
    Application --> nt35510
    nt35510 --> DsiHostCtrlIo["DsiHostCtrlIo trait"]
    DsiHostCtrlIo --> DSIHost["DSI Host (MCU-specific)"]
    DSIHost --> Panel["NT35510 Panel"]
```

The driver is transport-agnostic. It communicates with the panel entirely through the
`DsiHostCtrlIo` trait from `embedded-display-controller`, so it works with any MCU that
provides a DSI host implementation.

## Minimum Supported Rust Version

Rust 1.75+ (for `impl Trait` in trait bounds).

## License

0BSD
