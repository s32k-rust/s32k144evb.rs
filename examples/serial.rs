#![feature(used)]
#![no_std]

#[macro_use]
extern crate cortex_m;
extern crate s32k144;
extern crate s32k144evb;

use s32k144evb::{
    lpuart,
    wdog,
};

use s32k144::LPUART1;
use s32k144::PCC;
use s32k144::PORTC;
use s32k144::SCG;

fn main() {
    let mut wdog_settings = wdog::WatchdogSettings::default();
    wdog_settings.enable = false;
    wdog::configure(wdog_settings);

    
    let mut uart_settings = lpuart::UartSettings::default();
    //uart_settings.baudrate = 115200;

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

        let pcc = PCC.borrow(cs);
        pcc.pcc_lpuart1.modify(|_, w| w.cgc()._0());
        pcc.pcc_lpuart1.modify(|_, w| w.pcs()._001());
        pcc.pcc_lpuart1.modify(|_, w| w.cgc()._1());
        pcc.pcc_portc.modify(|_, w| w.cgc()._1());

        let portc = PORTC.borrow(cs);
        portc.pcr6.modify(|_, w| w.mux()._010());
        portc.pcr7.modify(|_, w| w.mux()._010());
        
        
        let lpuart = LPUART1.borrow(cs);

        
        //cortex_m::asm::bkpt();
        lpuart::configure(lpuart, uart_settings, 8000000);
        //cortex_m::asm::bkpt();
        lpuart::transmit(lpuart, 0xaa);
    });

    loop{
        for i in 0..100 {
            if i == 0 {
                cortex_m::interrupt::free(|cs| {
                    let lpuart = LPUART1.borrow(cs);
                    lpuart::transmit(lpuart, 0xaa);
                });
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
    cortex_m::asm::bkpt();
}
