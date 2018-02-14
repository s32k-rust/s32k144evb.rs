#![feature(used)]
#![no_std]

#[macro_use]
extern crate cortex_m;
extern crate s32k144evb;
extern crate s32k144;

use s32k144evb::{
    led,
    wdog,
};

use s32k144evb::pcc::Pcc;


fn main() {

    let peripherals = s32k144::Peripherals::take().unwrap();
    
    let mut wdog_settings = wdog::WatchdogSettings::default();
    wdog_settings.enable = false;
    let _wdog = wdog::Watchdog::init(&peripherals.WDOG, wdog_settings);

    let pcc = Pcc::init(&peripherals.PCC);
    let pcc_portd = pcc.enable_portd().unwrap();
    
    let led = led::RgbLed::init(
        &peripherals.PTD,
        &peripherals.PORTD,
        &pcc_portd,
    );

    loop {
        
        let loop_max = 3000;
        
        for i in 0..8*loop_max {
        
            match i/loop_max {
                0 => led.set(false, false, false),
                1 => led.set(false, false, true),
                2 => led.set(false, true, false),
                3 => led.set(false, true, true),
                4 => led.set(true, false, false),
                5 => led.set(true, false, true),
                6 => led.set(true, true, false),
                7 => led.set(true, true, true),
                _ => unreachable!(),
            }
            
        
        }
    }
    
}
