#![no_std]


extern crate s32k144;
#[cfg_attr(feature = "itm", macro_use)]
extern crate cortex_m;
extern crate cortex_m_rt;
extern crate bit_field;
extern crate embedded_types;

pub mod led;
pub mod wdog;
pub mod can;
pub mod lpuart;
pub mod spc;
pub mod pcc;

pub mod console;

#[cfg(any(feature = "panic-over-itm", feature = "panic-over-serial"))]
mod panic;
