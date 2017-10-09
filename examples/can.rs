#![feature(used)]
#![no_std]

#[macro_use]
extern crate cortex_m;
extern crate s32k144;
extern crate s32k144evb;
extern crate embedded_types;

use cortex_m::asm;

use s32k144::{
    SCG,
};

use s32k144evb::{
    can,
    wdog,
};

use s32k144evb::can::{
    ID,
    BaseID,
    CanSettings,
    MailboxHeader,
};

use embedded_types::can::DataFrame;

fn main() {
    
    let mut wdog_settings = wdog::WatchdogSettings::default();
    wdog_settings.enable = false;
    wdog::configure(wdog_settings);

    s32k144evb::serial::init();

    let peripherals = unsafe{ s32k144::Peripherals::all() };
    
    let mut can_settings = CanSettings::default();    
    can_settings.source_frequency = 8000000;
    can_settings.self_reception = false;

    // Enable and configure the system oscillator
    let scg = peripherals.SCG;
    let porte = peripherals.PORTE;
    let pcc = peripherals.PCC;
        
    scg.sosccfg.modify(|_, w| w
                       .range()._11()
                       .hgo()._1()
                       .erefs()._1()
    );
    
    scg.soscdiv.modify(|_, w| w
                       .soscdiv2().bits(0b001)
    );
    
    scg.sosccsr.modify(|_, w| w.soscen()._1());
    
    // Configure the can i/o pins
    pcc.pcc_porte.modify(|_, w| w.cgc()._1());
    porte.pcr4.modify(|_, w| w.mux()._101());
    porte.pcr5.modify(|_, w| w.mux()._101());
    
    pcc.pcc_flex_can0.modify(|_, w| w.cgc()._1());
    
    let can = can::Can::init(peripherals.CAN0, &can_settings).unwrap();

    loop {

        let loop_max = 100000;
        for n in 0..256 {
            let mut message = DataFrame::new(ID::BaseID(BaseID::new(n as u16)));
            message.set_data_length(8);
            for i in 0..8 {
                message.data_as_mut()[i] = i as u8;
            }
            for i in 0..loop_max {
                if i == 0 {
                    can.transmit(&message.into()).unwrap();           
                }
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
