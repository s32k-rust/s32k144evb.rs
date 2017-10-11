#![feature(used)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m;
extern crate s32k144;
extern crate s32k144evb;
extern crate cortex_m_rtfm as rtfm;

use cortex_m::peripheral::SystClkSource;

use rtfm::{app, Threshold};

use s32k144evb::{
    led,
    wdog,
};
use s32k144::Interrupt;


app! {
    device: s32k144,

    resources: {
        static ON: bool = false;
        static COUNT: u32 = 0;
    },
    
    tasks: {
        SYS_TICK: {
            path: toggle,
            resources: [ON, COUNT],
        },
    },
}


fn init(p: init::Peripherals, _r: init::Resources) {

    let mut wdog_settings = wdog::WatchdogSettings::default();
    wdog_settings.enable = false;
    wdog::configure(wdog_settings);
    
    led::init();
    led::RED.off();
    led::GREEN.off();
    led::BLUE.off();

    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(400_000); // Frequency 100Hz
    p.SYST.enable_interrupt();
    p.SYST.enable_counter();
}

fn idle() -> ! {
    // Sleep
    loop {
        rtfm::wfi();
    }
}

fn toggle(_t: &mut Threshold, r: SYS_TICK::Resources) {
    **r.COUNT += 1;

    if **r.COUNT >= 100 { //1s
        **r.COUNT = 0;
        **r.ON = !**r.ON;
        if **r.ON {
            led::GREEN.on();
        } else {
            led::GREEN.off();
        }
    }
}
