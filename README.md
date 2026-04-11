# nt35510

`no_std` driver for NT35510 DSI LCD controller panels.

Transport-agnostic — accepts any DSI host implementing
`embedded_display_controller::dsi::DsiHostCtrlIo`.

Tested on STM32F469I-Discovery (B08 revision, Frida 3K138 panel).

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

## API compatibility

This crate mirrors the [`otm8009a`](https://crates.io/crates/otm8009a) API for BSP-level
compatibility. Both drivers expose `Mode`, `ColorMap`, and similar config structs.

## Minimum Supported Rust Version

Rust 1.75+ (for `impl Trait` in trait bounds).

## License

0BSD
