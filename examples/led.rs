#![feature(used)]
#![no_std]

#[macro_use]
extern crate cortex_m;
extern crate s32k144evb;

use cortex_m::asm;

use s32k144evb::{
    led,
    wdog,
};


fn main() {
    
    let mut wdog_settings = wdog::WatchdogSettings::default();
    wdog_settings.enable = false;
    wdog::configure(wdog_settings);
    
    led::init();
    led::RED.off();
    led::GREEN.off();
    led::BLUE.off();

    loop {
        
        let loop_max = 3000;
        for i in 0..8*loop_max {
        
           
            if (i / loop_max) % 2 != 0 {
                led::RED.on();
            } else {
                led::RED.off();
            }
            
            if (i / 2 / loop_max) % 2 != 0 {
                led::BLUE.on();
            } else {
                led::BLUE.off();
            }
            
            if (i / 4 / loop_max) % 2 != 0{
                led::GREEN.on();
            } else {
                led::GREEN.off();
            }
            
        
        }
    }
    
}
