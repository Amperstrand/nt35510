# nt35510

Standalone `no_std` NT35510 DSI LCD controller driver.

This crate is transport-agnostic and accepts any DSI host implementing
`embedded_display_controller::dsi::DsiHostCtrlIo`.

## Usage

```rust
use embedded_display_controller::dsi::DsiHostCtrlIo;
use embedded_hal::delay::DelayNs;
use nt35510::{Nt35510, Nt35510Config};

fn init_display(dsi: &mut impl DsiHostCtrlIo, delay: &mut impl DelayNs) {
    let mut panel = Nt35510::new();
    let _ = panel.probe(dsi);

    panel.init(dsi, delay).unwrap();
}
```

`Nt35510Config::default()` matches STM32F469I-DISCO tested settings
(portrait, RGB, RGB888, 480x800). Use `init_with_config()` for custom
orientation, color map, or pixel format. `init_rgb565()` is available
for 16-bit mode.
