//! This module takes care of interfacing the serial port on the SDAOpen interface

use core::fmt;
use core::fmt::Write;

use cortex_m;

use s32k144;
use s32k144::LPUART1;
use s32k144::PCC;
use s32k144::PORTC;
use s32k144::SCG;
use s32k144::lpuart0;

use embedded_types::io::blocking;

use lpuart;
use pc;

impl<'p> fmt::Write for Serial<'p> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            blocking(|| lpuart::transmit(self.lpuart, *c)).unwrap();
        }
        Ok(())
    }
}

pub struct Serial<'a> {
    lpuart: &'a lpuart0::RegisterBlock,
    //_pc: &'a pc::Pc<'a>,
}

impl<'a> Serial<'a> {
    pub fn init(
        lpuart: &'a s32k144::lpuart0::RegisterBlock,
        pc: &'a pc::Pc<'a>,
    ) -> Self{
        init(lpuart);
        Serial{
            lpuart: lpuart,
            //_pc: pc,
        }
    }
}

/// This init functions needs to be called before using any of the functionality in this module
#[cfg(feature = "serial")]
fn init(lpuart: & s32k144::lpuart0::RegisterBlock) {
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
                
        lpuart::configure(lpuart, uart_config, 8000000).unwrap();
    });
}

#[cfg(feature = "serial")]
pub fn write_str(string: &str) {
    cortex_m::interrupt::free(|cs| {
        let lpuart = LPUART1.borrow(cs);
        Serial{lpuart: &lpuart}.write_str(string).ok();
    });
}

#[cfg(feature = "serial")]
pub fn write_fmt(fmt: fmt::Arguments) {
    cortex_m::interrupt::free(|cs| {
        let lpuart = LPUART1.borrow(cs);
        Serial{lpuart: &lpuart}.write_fmt(fmt).ok();
    });
}

#[cfg(feature = "panic-over-serial")]
#[lang = "panic_fmt"]
unsafe extern "C" fn panic_fmt(
    msg: fmt::Arguments,
    file: &'static str,
    line: u32,
    _column: u32) -> ! {

    write_fmt(format_args!("Panicked at '{}', {}:{}\n", msg, file, line));

    // If running in debug mode, stop. If not, abort.
    if cfg!(debug_assertions) {
        loop {}
    }
    
    ::core::intrinsics::abort()
}
