#![feature(used)]
#![no_std]

extern crate cortex_m;
extern crate s32k144;
#[macro_use]
extern crate s32k144evb;
extern crate embedded_types;

use core::fmt::Write;

use s32k144evb::{
    wdog,
};

fn main() {
    let mut wdog_settings = wdog::WatchdogSettings::default();
    wdog_settings.enable = false;
    wdog::configure(wdog_settings).unwrap();

    let peripherals = unsafe{ s32k144::Peripherals::all() };
    
    let mut serial = s32k144evb::serial::Serial::init(peripherals.LPUART1);

    writeln!(serial, "This is a println");
    writeln!(serial, "Next a panic will be demonstrated by overflowing an integer");
    let mut i: u8 = 0;

    loop {
        writeln!(serial, "I count: {}", i);
        i += 1;
    }
}
