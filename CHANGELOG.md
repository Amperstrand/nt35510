# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2026-05-08

### Added
- `Default` impls for `Mode`, `ColorMap`, `ColorFormat`, and `PanelTiming`
- `Debug`, `Clone`, `Copy`, `PartialEq`, and `Eq` derives for `Nt35510`
- Optional `defmt` feature with `defmt::Format` derives on public API types
- Public `is_initialized`, `sleep_out`, `soft_reset`, `set_inversion`, `set_display_on`, `set_display_off`, `read_brightness`, `read_id`, `get_scan_line`, and `PanelTiming::for_mode_dsi` APIs
- Rustdoc coverage for public items that previously lacked documentation

### Fixed
- README examples now match the current `probe()` signature and `init()` defaults

## [0.2.0] - 2026-04-12

### Added
- `memory_check()` diagnostic method for RAM verification
- Private `regs` module (register constants no longer re-exported)
- `Display` impl for `Error` type
- `init_rgb888()` convenience method
- `#[derive(Default)]` for `Nt35510Config`
- `#[must_use]` annotations
- Comprehensive README documentation

### Changed
- Init sequence aligned with hardware-verified BSP values
- 18 unit tests covering all public API surface

### Fixed
- Broken intra-doc link resolved

## [0.1.0] - 2026-02-27

### Added
- Initial release with RGB565 support
- `init_rgb565()` convenience method
- Basic DSI panel initialization
