#![feature(used)]
#![no_std]

#[macro_use]
extern crate cortex_m;
extern crate s32k144evb;
extern crate bit_field;

use bit_field::BitField;

use cortex_m::asm;

use s32k144evb::{
    can,
    wdog,
};

use s32k144evb::can::{
    CanSettings,
    CanMessage,
    CanID,
};


fn main() {
    
    let mut wdog_settings = wdog::WatchdogSettings::default();
    wdog_settings.enable = false;
    wdog::configure(wdog_settings);

    let mut can_settings = CanSettings::default();
    
    can_settings.source_frequency = 12000000;
    can_settings.clock_source = can::ClockSource::Peripheral;

    can_settings.self_reception = false;
    
    can::init(&can_settings).unwrap();

    let message = CanMessage{
        id: CanID::Standard(0),
        dlc: 0,
        data: [0; 8],
    };
    
    loop {
        
        let loop_max = 10000;
        for i in 0..loop_max {
            if i == 0 {
                can::transmit(&message, 6).unwrap();           
            }
        }
    }
    
}

// As we are not using interrupts, we just register a dummy catch all handler
#[allow(dead_code)]
#[used]
#[link_section = ".vector_table.interrupts"]
static INTERRUPTS: [extern "C" fn(); 240] = [default_handler; 240];

extern "C" fn default_handler() {
    asm::bkpt();
}