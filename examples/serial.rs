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
    pc,
};

fn main() {
    let mut wdog_settings = wdog::WatchdogSettings::default();
    wdog_settings.enable = false;
    wdog::configure(wdog_settings).unwrap();

    let peripherals = unsafe{ s32k144::Peripherals::all() };
    
    let pc_config = pc::Config{
        system_oscillator: pc::SystemOscillatorInput::Crystal(8_000_000),
        soscdiv2: pc::SystemOscillatorOutput::Div1,
        .. Default::default()
    };
    
    let pc = pc::Pc::init(
        peripherals.SCG,
        peripherals.SMC,
        peripherals.PMC,
        pc_config
    ).unwrap();
    
    let mut serial = s32k144evb::serial::Serial::init(peripherals.LPUART1, &pc);

    writeln!(serial, "This is a println").unwrap();
    writeln!(serial, "Next a panic will be demonstrated by overflowing an integer").unwrap();
    let mut i: u8 = 0;

    loop {
        writeln!(serial, "I count: {}", i).unwrap();
        i += 1;
    }
}
