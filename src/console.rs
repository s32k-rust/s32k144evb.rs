//! This module gives a consistent interface over different "remote consoles"
//!
//! The most common consoles in use for this chip are:
//!  - LPUART (LPUART1 is the one connected to the OpenSDA chip on s32k144evb)
//!  - ITM
// TODO: implement and test ITM

use core::fmt;

use cortex_m;

use s32k144;
use s32k144::LPUART1;
use s32k144::PCC;
use s32k144::PORTC;
use s32k144::SCG;
use s32k144::lpuart0;

use embedded_types;
use embedded_types::io::Write;

use lpuart;
use pc;

impl<'p> embedded_types::io::Write for LpuartConsole<'p> {
    fn write(&mut self, buf: &[u8]) -> embedded_types::io::Result<usize> {
        for i in 0..buf.len() {
            match self.lpuart.transmit(buf[i]) {
                Ok(()) => (),
                Err(embedded_types::io::Error::BufferExhausted) => return Ok(i),
                Err(e) => return Err(e),
            }
        }
        Ok(buf.len())
    }
}

/// Allow usage of uart as a Console
pub struct LpuartConsole<'a> {
    lpuart: lpuart::Lpuart<'a>,
}

impl<'a> LpuartConsole<'a> {
    pub fn init(
        lpuart: &'a s32k144::lpuart0::RegisterBlock,
        pc: &'a pc::Pc<'a>,
    ) -> Self{
        let mut uart_config = lpuart::Config::default();
        uart_config.baudrate = 115200;
        
        cortex_m::interrupt::free(|cs| {
            
            let pcc = PCC.borrow(cs);
            pcc.pcc_lpuart1.modify(|_, w| w.cgc()._0());
            pcc.pcc_lpuart1.modify(|_, w| w.pcs()._001());
            pcc.pcc_lpuart1.modify(|_, w| w.cgc()._1());
            pcc.pcc_portc.modify(|_, w| w.cgc()._1());
            
            let portc = PORTC.borrow(cs);
            portc.pcr6.modify(|_, w| w.mux()._010());
            portc.pcr7.modify(|_, w| w.mux()._010());
        });

        LpuartConsole{
            lpuart: lpuart::Lpuart::init(lpuart, pc, uart_config, 8_000_000).unwrap(),
        }
    }
}


