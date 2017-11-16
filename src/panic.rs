use core::fmt;

use embedded_types::io::Write;

use s32k144;

use cortex_m;
use cortex_m::peripheral::ITM;

use spc;
use console;

#[cfg(feature = "panic-over-itm")]
#[lang = "panic_fmt"]
unsafe extern "C" fn panic_fmt(
    msg: fmt::Arguments,
    file: &'static str,
    line: u32,
    _column: u32) -> ! {


    cortex_m::interrupt::free(|cs| {
        let itm = ITM.borrow(cs);
        iprintln!(&itm.stim[0], "Panicked at '{}', {}:{}", msg, file, line);
    });

    // If running in debug mode, stop. If not, abort.
    if cfg!(debug_assertions) {
        loop {}
    }

    ::core::intrinsics::abort()
}

#[cfg(feature = "panic-over-serial")]
#[lang = "panic_fmt"]
unsafe extern "C" fn panic_fmt(
    msg: fmt::Arguments,
    file: &'static str,
    line: u32,
    _column: u32) -> ! {

    // This function is diverging, so if any settings have been previously made we will mess with them freely.
    
    let spc_config = spc::Config{
        system_oscillator: spc::SystemOscillatorInput::Crystal(8_000_000),
        soscdiv2: spc::SystemOscillatorOutput::Div1,
        .. Default::default()
    };
    
    cortex_m::interrupt::free(|cs| {
        
        let spc = spc::Spc::init(
            s32k144::SCG.borrow(cs),
            s32k144::SMC.borrow(cs),
            s32k144::PMC.borrow(cs),
            spc_config
        ).unwrap();
        
        let mut serial = console::LpuartConsole::init(s32k144::LPUART1.borrow(cs), &spc);

        writeln!(serial, "Panicked at '{}', {}:{}", msg, file, line).unwrap();
    });
                              
    // If running in debug mode, stop. If not, abort.
    if cfg!(debug_assertions) {
        loop {}
    }
    
    ::core::intrinsics::abort()
}
