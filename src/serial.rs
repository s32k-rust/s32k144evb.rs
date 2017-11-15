//! This module takes care of interfacing the serial port on the SDAOpen interface

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

impl<'p> embedded_types::io::Write for Serial<'p> {
    fn write(&mut self, buf: &[u8]) -> embedded_types::io::Result<usize> {
        for i in 0..buf.len() {
            match lpuart::transmit(self.lpuart, buf[i]) {
                Ok(()) => (),
                Err(embedded_types::io::Error::BufferExhausted) => return Ok(i),
                Err(e) => return Err(e),
            }
        }
        Ok(buf.len())
    }
}

pub struct Serial<'a> {
    lpuart: &'a lpuart0::RegisterBlock,
    _pc: &'a pc::Pc<'a>,
}

impl<'a> Serial<'a> {
    pub fn init(
        lpuart: &'a s32k144::lpuart0::RegisterBlock,
        pc: &'a pc::Pc<'a>,
    ) -> Self{
        init(lpuart);
        Serial{
            lpuart: lpuart,
            _pc: pc,
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

#[cfg(feature = "panic-over-serial")]
#[lang = "panic_fmt"]
unsafe extern "C" fn panic_fmt(
    msg: fmt::Arguments,
    file: &'static str,
    line: u32,
    _column: u32) -> ! {

    // This function is diverging, so if any settings have been previously made we will mess with them freely.
    
    let pc_config = pc::Config{
        system_oscillator: pc::SystemOscillatorInput::Crystal(8_000_000),
        soscdiv2: pc::SystemOscillatorOutput::Div1,
        .. Default::default()
    };
    
    cortex_m::interrupt::free(|cs| {
        
        let pc = pc::Pc::init(
            s32k144::SCG.borrow(cs),
            s32k144::SMC.borrow(cs),
            s32k144::PMC.borrow(cs),
            pc_config
        ).unwrap();
        
        let mut serial = Serial::init(LPUART1.borrow(cs), &pc);

        writeln!(serial, "Panicked at '{}', {}:{}", msg, file, line).unwrap();
    });
                              
    // If running in debug mode, stop. If not, abort.
    if cfg!(debug_assertions) {
        loop {}
    }
    
    ::core::intrinsics::abort()
}
