#![no_main]
#![no_std]

extern crate cortex_m_rt;
extern crate embedded_types;
extern crate s32k144;
extern crate s32k144evb;

use cortex_m_rt::entry;

use s32k144evb::{can, spc, wdog};

use s32k144evb::pcc::Pcc;

use s32k144evb::can::{CanSettings, ID};

use embedded_types::can::{BaseID, DataFrame};

#[entry]
fn main() -> ! {
    let peripherals = s32k144::Peripherals::take().unwrap();

    let wdog_settings = wdog::WatchdogSettings {
        //timeout_value: 0xffff,
        enable: false,
        ..Default::default()
    };
    let wdog = wdog::Watchdog::init(&peripherals.WDOG, wdog_settings).unwrap();
    wdog.reset();

    let spc_config = spc::Config {
        system_oscillator: spc::SystemOscillatorInput::Crystal(8_000_000),
        soscdiv2: spc::SystemOscillatorOutput::Div1,
        ..Default::default()
    };

    let spc = spc::Spc::init(
        &peripherals.SCG,
        &peripherals.SMC,
        &peripherals.PMC,
        spc_config,
    )
    .unwrap();

    let mut can_settings = CanSettings::default();
    can_settings.self_reception = false;

    // Enable and configure the system oscillator
    let pcc = Pcc::init(&peripherals.PCC);
    let _pcc_can0 = pcc.enable_can0().unwrap();
    let _pcc_porte = pcc.enable_porte().unwrap();

    // Configure the can i/o pins
    let porte = peripherals.PORTE;
    porte.pcr4.modify(|_, w| w.mux()._101());
    porte.pcr5.modify(|_, w| w.mux()._101());

    let can = can::Can::init(&peripherals.CAN0, &spc, &can_settings).unwrap();

    loop {
        let loop_max = 100000;
        for n in 0..256 {
            let mut message = DataFrame::new(ID::BaseID(BaseID::new(n as u16)));
            message.set_data_length(8);
            for i in 0..8 {
                message.data_as_mut()[i] = i as u8;
            }
            for i in 0..loop_max {
                if i == 0 {
                    can.transmit(&message.into()).unwrap();
                }
                if i & 1000 == 0 {
                    wdog.reset();
                }
            }
        }
    }
}
