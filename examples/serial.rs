#![feature(used)]
#![no_std]

extern crate cortex_m;
extern crate s32k144;
#[macro_use]
extern crate s32k144evb;
extern crate embedded_types;

use s32k144evb::{
    wdog,
};

fn main() {
    let mut wdog_settings = wdog::WatchdogSettings::default();
    wdog_settings.enable = false;
    wdog::configure(wdog_settings).unwrap();

    s32k144evb::serial::init();

    println!("This is a println");
    println!("Next a panic will be demonstrated by overflowing an integer");
    let mut i: u8 = 0;

    loop {
        println!("I count: {}", i);
        i += 1;
    }
}
