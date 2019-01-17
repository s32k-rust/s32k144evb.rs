use console;
use core::fmt;
use cortex_m;
use embedded_types::io::Write;
use s32k144;
use spc;

#[cfg(feature = "panic-over-itm")]
#[lang = "panic_fmt"]
unsafe extern "C" fn panic_fmt(
    msg: fmt::Arguments,
    file: &'static str,
    line: u32,
    _column: u32,
) -> ! {
    cortex_m::interrupt::free(|cs| {
        let itm = ITM.borrow(cs);
        iprintln!(&itm.stim[0], "Panicked at '{}', {}:{}", msg, file, line);
    });

    // If running in debug mode, stop. If not, abort.
    if cfg!(debug_assertions) {
        loop {}
    }

    loop {}
}

#[cfg(feature = "panic-over-serial")]
#[allow(dead_code)]
unsafe extern "C" fn panic_fmt(
    msg: fmt::Arguments,
    file: &'static str,
    line: u32,
    _column: u32,
) -> ! {
    // This function is diverging, so if any settings have been previously made we will mess with them freely.

    let spc_config = spc::Config {
        system_oscillator: spc::SystemOscillatorInput::Crystal(8_000_000),
        soscdiv2: spc::SystemOscillatorOutput::Div1,
        ..Default::default()
    };

    cortex_m::interrupt::free(|_cs| {
        let peripherals = s32k144::Peripherals::steal();

        // turn of all other muxes than the one that muxes to the OpenSDA
        let pcc = peripherals.PCC;
        let portc = peripherals.PORTC;
        let portd = peripherals.PORTD;

        pcc.pcc_portc.modify(|_, w| w.cgc()._1());
        pcc.pcc_portd.modify(|_, w| w.cgc()._1());

        portc.pcr7.modify(|_, w| w.mux()._010());
        portc.pcr9.modify(|_, w| w.mux()._000());
        portd.pcr14.modify(|_, w| w.mux()._000());

        let spc = spc::Spc::init(
            &peripherals.SCG,
            &peripherals.SMC,
            &peripherals.PMC,
            spc_config,
        )
        .unwrap();

        let mut serial = console::LpuartConsole::init(&peripherals.LPUART1, &spc);

        writeln!(serial, "Panicked at '{}', {}:{}", msg, file, line).unwrap();
    });

    loop {}
}
