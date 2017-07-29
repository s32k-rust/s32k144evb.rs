#![feature(used)]
#![no_std]

#[macro_use]
extern crate cortex_m;
extern crate s32k144;
extern crate s32k144evb;

use cortex_m::asm;

use s32k144::{
    SCG,
};

use s32k144evb::{
    can,
    wdog,
};

use s32k144evb::can::{
    CanSettings,
    CanMessage,
    CanID,
    MessageBufferHeader,
};


fn main() {
    
    let mut wdog_settings = wdog::WatchdogSettings::default();
    wdog_settings.enable = false;
    wdog::configure(wdog_settings);

    let mut can_settings = CanSettings::default();    
    can_settings.source_frequency = 8000000;
    can_settings.self_reception = false;

    let can_mb_settings = [MessageBufferHeader::default_transmit(), MessageBufferHeader::default_receive()];

    // Enable and configure the system oscillator
    cortex_m::interrupt::free(|cs| {
        let scg = SCG.borrow(cs);
        
        scg.sosccfg.modify(|_, w| w
                           .range()._11()
                           .hgo()._1()
                           .erefs()._1()
        );
        
        scg.soscdiv.modify(|_, w| w
                           .soscdiv2().bits(0b001)
        );

        scg.sosccsr.modify(|_, w| w.soscen()._1());
    });

    can::init(&can_settings, &can_mb_settings).unwrap();

    let message = CanMessage{
        id: CanID::Standard(0),
        dlc: 0,
        data: [0; 8],
    };
    
    loop {

        let loop_max = 100000;
        for i in 0..loop_max {
            if i == 0 {
                can::transmit(&message, 0).unwrap();           
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
