#![cfg_attr(feature = "panic-over-itm", feature(core_intrinsics))]
#![cfg_attr(feature = "panic-over-itm", feature(lang_items))]

#![no_std]


extern crate s32k144;
#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rt;
extern crate bit_field;

pub mod led;
pub mod wdog;
pub mod can;
pub mod lpuart;

#[cfg(feature = "panic-over-itm")]
mod panic;
