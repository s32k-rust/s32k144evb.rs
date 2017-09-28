use cortex_m::peripheral::ITM;

use core;
use cortex_m;

#[cfg(feature = "panic-over-itm")]
#[lang = "panic_fmt"]
unsafe extern "C" fn panic_fmt(
    msg: core::fmt::Arguments,
    file: &'static str,
    line: u32,
    _column: u32) -> ! {


    cortex_m::interrupt::free(|cs| {
        let itm = ITM.borrow(cs);
        iprintln!(&itm.stim[0], "Panicked at '{}', {}:{}", msg, file, line);
    });

    ::core::intrinsics::abort()
}
