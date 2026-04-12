# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
