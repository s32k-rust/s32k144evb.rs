#![feature(used)]
#![no_std]

extern crate cortex_m;
extern crate s32k144;
#[macro_use]
extern crate s32k144evb;
extern crate embedded_types;

use embedded_types::io::Write;
use embedded_types::io::Read;

use s32k144evb::{
    wdog,
    spc,
};

fn main() {
    let peripherals = unsafe{ s32k144::Peripherals::all() };

    let mut wdog_settings = wdog::WatchdogSettings::default();
    wdog_settings.enable = false;
    let _wdog = wdog::Watchdog::init(peripherals.WDOG, wdog_settings);
    
    let pc_config = spc::Config{
        system_oscillator: spc::SystemOscillatorInput::Crystal(8_000_000),
        soscdiv2: spc::SystemOscillatorOutput::Div1,
        .. Default::default()
    };
    
    let spc = spc::Spc::init(
        peripherals.SCG,
        peripherals.SMC,
        peripherals.PMC,
        pc_config
    ).unwrap();
    
    let mut console = s32k144evb::console::LpuartConsole::init(peripherals.LPUART1, &spc);

    writeln!(console, "Please write something").unwrap();
    let mut buf = [0u8; 64];
    let chars = console.read_until(b'\n', &mut buf).unwrap();

    writeln!(console, "Your wrote: \"{}\"", core::str::from_utf8(&buf[0..chars]).unwrap()).unwrap();
    
    writeln!(console, "Next a panic will be demonstrated by overflowing an integer").unwrap();
    let mut i: u8 = 0;

    loop {
        writeln!(console, "I count: {}", i).unwrap();
        i += 1;
    }
}
