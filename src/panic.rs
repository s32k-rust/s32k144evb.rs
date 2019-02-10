//! With the panic handler being `#[inline(never)]` the symbol `rust_begin_unwind` will be
//! available to place a breakpoint on to halt when a panic is happening.

use crate::{console, spc};
use core::{
    panic::PanicInfo,
    sync::atomic::{self, Ordering},
};
use cortex_m;
use embedded_types::io::Write;
use s32k144;

#[cfg(feature = "panic-over-itm")]
#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::free(|cs| {
        let itm = ITM.borrow(cs);
        iprintln!(&itm.stim[0], "{}", info);
    });

    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

#[cfg(feature = "panic-over-serial")]
#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // This function is diverging, so if any settings have been previously made we will mess with them freely.
    let spc_config = spc::Config {
        system_oscillator: spc::SystemOscillatorInput::Crystal(8_000_000),
        soscdiv2: spc::SystemOscillatorOutput::Div1,
        ..Default::default()
    };

    cortex_m::interrupt::free(|_cs| unsafe {
        let pcc = &*s32k144::PCC::ptr();
        let portc = &*s32k144::PORTC::ptr();
        let portd = &*s32k144::PORTD::ptr();

        // turn of all other muxes than the one that muxes to the OpenSDA
        pcc.pcc_portc.modify(|_, w| w.cgc()._1());
        pcc.pcc_portd.modify(|_, w| w.cgc()._1());

        portc.pcr7.modify(|_, w| w.mux()._010());
        portc.pcr9.modify(|_, w| w.mux()._000());
        portd.pcr14.modify(|_, w| w.mux()._000());

        let spc = spc::Spc::init(
            &*s32k144::SCG::ptr(),
            &*s32k144::SMC::ptr(),
            &*s32k144::PMC::ptr(),
            spc_config,
        )
        .unwrap();

        let mut serial = console::LpuartConsole::init(&*s32k144::LPUART1::ptr(), &spc);

        writeln!(serial, "{}", info).unwrap();
    });

    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}
