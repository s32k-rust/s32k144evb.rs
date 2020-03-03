# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
### Changed
### Removed

## [0.8.0] - 2020-03-03
### Added
- CSEc module, interfacing with the board's hardware-accelerated cryptographic functions (RNG, AES-128-CBC, CMAC)

### Changed
### Removed

## [0.7.0] - 2018-02-10
### Added
 - SPC SW module to handle Power and Clocking HW modules (SCG, SMC and PMC but not PCC).
 - Reset function for watchdog timer.
### Changed
 - Only use upper Part of RAM, solved problems with init of .data.
 - Watchdog now needs the WDOG peripheral to work.
 - CAN MessageBufferCode is converted with `decode` instead of `from` since the conversion is fallible
 - Use FIFO buffer when receiving LPUART
 - Updated s32k144 version
 - Ported crate to stable release channel, Rust edition 2018
 - Removed deprecated default feature `abort-on-panic` from `cortex-m-rt`; use `extern crate panic_halt;` or use a panic handler in `src/panic.rs` (default: `panic-over-serial`)
### Removed
