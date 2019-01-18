//! Periodical toggling of the green LED using RTFM.
//!
//! s32k144evb::led::RgbLed cannot safely be shared yet, hence the unsafe code.

#![no_main]
#![no_std]

extern crate cortex_m;
extern crate panic_halt;
extern crate s32k144;
extern crate s32k144evb;

use s32k144::Interrupt;

use rtfm::{app, Instant};

use s32k144evb::{led, pcc, wdog};

const PERIOD: u32 = 16_000_000;

#[app(device = s32k144)]
const APP: () = {
    // Resources
    static mut ON: bool = false;
    static mut PTD: s32k144::PTD = ();

    #[init(schedule = [toggle])]
    fn init() {
        let mut wdog_settings = wdog::WatchdogSettings::default();
        wdog_settings.enable = false;
        let _wdog = wdog::Watchdog::init(&device.WDOG, wdog_settings);

        let pcc = pcc::Pcc::init(&device.PCC);
        let pcc_portd = pcc.enable_portd().unwrap();

        let led = led::RgbLed::init(&device.PTD, &device.PORTD, &pcc_portd);
        led.set(false, false, false);

        schedule.toggle(Instant::now() + PERIOD.cycles()).unwrap();

        PTD = device.PTD;
    }

    #[idle]
    fn idle() -> ! {
        // Sleep
        loop {
            rtfm::pend(Interrupt::DMA0);
        }
    }

    #[task(resources = [ON, PTD], schedule = [toggle])]
    fn toggle() {
        let r = resources;

        if *r.ON {
            r.PTD.pcor.write(|w| unsafe { w.ptco().bits(1 << 16) });
        } else {
            r.PTD.psor.write(|w| unsafe { w.ptso().bits(1 << 16) });
        }

        *r.ON = !(*r.ON);

        schedule.toggle(scheduled + PERIOD.cycles()).unwrap();
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn DMA0();
    }
};
