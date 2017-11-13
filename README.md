# s32k144evb.rs [![Build Status](https://travis-ci.org/kjetilkjeka/s32k144evb.rs.svg?branch=master)](https://travis-ci.org/kjetilkjeka/s32k144evb.rs) [![Crates.io](https://img.shields.io/crates/v/s32k144evb.svg)](https://crates.io/crates/s32k144evb)

> Board support crate for NXP S32K144EVB evaluation board

# Programming and debugging

## OpenSDA
This boards uses OpenSDAv1 for programming and debuging
### J-Link interface
J-Link supply firmware for OpenSDA hardware, with this you will be able to use the evaluation board as a J-Link programmer/debugger. For instructions how to do this look [here](https://www.segger.com/products/debug-probes/j-link-oem/j-link-firmware-models/opensda-sda-v2/), i've used this method with the OpenSDAv1 generic firmware supplied [here](https://www.segger.com/downloads/jlink#JLinkOpenSDABoardSpecificFirmwares)

## Bobbin-CLI
[Bobbin-cli](https://github.com/bobbin-rs/bobbin-cli) greatly simplifies compilation and flashing embedded software written in Rust. After installing the j-link tools, it can be installed by running ```cargo install bobbin-cli```. Run the LED example on a connected evaluation board with j-link firmware (or an external j-link) with ```bobbin load --example led``` or check out the [readme](https://github.com/bobbin-rs/bobbin-cli/blob/master/README.md) for a full description of what you can do.

# Getting started coding (quickstart)
There exists a quickstart template for s32k144evb projects. You can find the documentation [here](https://docs.rs/s32k144evb-quickstart)

# License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
