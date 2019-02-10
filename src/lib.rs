#![no_std]

extern crate bit_field;
#[cfg_attr(feature = "itm", macro_use)]
extern crate cortex_m;
extern crate cortex_m_rt;
extern crate embedded_types;
extern crate s32k144;

pub mod can;
pub mod led;
pub mod lpuart;
pub mod pcc;
pub mod spc;
pub mod wdog;

pub mod console;

#[cfg(any(feature = "panic-over-itm", feature = "panic-over-serial"))]
mod panic;
