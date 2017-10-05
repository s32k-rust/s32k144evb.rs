//! This module takes care of interfacing the serial port on the SDAOpen interface

use core::fmt;
use core::fmt::Write;

use cortex_m;

use s32k144::LPUART1;
use s32k144::PCC;
use s32k144::PORTC;
use s32k144::SCG;
use s32k144::lpuart0;

use embedded_types::io::blocking_transmit;

use lpuart;

struct Port<'p>(&'p lpuart0::RegisterBlock);

impl<'p> fmt::Write for Port<'p> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            blocking_transmit(|| lpuart::transmit(self.0, *c));
        }
        Ok(())
    }
}

/// This init functions needs to be called before using any of the functionality in this module
#[cfg(feature = "serial")]
pub fn init() {
    let mut uart_settings = lpuart::UartSettings::default();
    uart_settings.baudrate = 115200;

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
        
        
        lpuart::configure(lpuart, uart_settings, 8000000);
    });
}

#[cfg(feature = "serial")]
pub fn write_str(string: &str) {
    cortex_m::interrupt::free(|cs| {
        let lpuart = LPUART1.borrow(cs);
        Port(&lpuart).write_str(string).ok();
    });
}

#[cfg(feature = "serial")]
pub fn write_fmt(fmt: fmt::Arguments) {
    cortex_m::interrupt::free(|cs| {
        let lpuart = LPUART1.borrow(cs);
        Port(&lpuart).write_fmt(fmt).ok();
    });
}

#[cfg(feature = "print-over-serial")]
#[macro_export]
macro_rules! print {
    ($fmt:expr) => {
        serial::write_str($fmt);
    };
    ($($arg:tt)*) => {
        serial::write_fmt(format_args!($($arg)*));
    };
}

#[cfg(feature = "print-over-serial")]
#[macro_export]
macro_rules! println {
    ($fmt:expr) => {
        print!(concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        print!(concat!($fmt, "\n"), $($arg)*);
    };
}
