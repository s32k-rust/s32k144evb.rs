//! Integration testing of the CSEc module. Tests the following:
//! - initializes the CSEc module;
//! - loads a plaintext key;
//! - randomizes 128 bits of data;
//! - encrypts a byte string to said key, using the randomized bits as initialization vector;
//! - decrypts the encrypted strings (ensuring the string matches before encryption and after
//! decryption);
//! - generates a MAC for a string and verifies it.
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use s32k144;
use s32k144evb::{csec, led, pcc::Pcc, wdog};

const MSG: &[u8] = b"Key:0123456789abKey:0123456789abKey:0123456789abKey:0123456789abKey:0123456789abKey:0123456789abKey:0123456789abKey:0123456789abKey:0123456789abKey:0123456789ab";
const MSG_LEN: usize = 16 * 10;
const PLAINKEY: [u8; 16] = [
    0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
];

#[entry]
unsafe fn main() -> ! {
    let p = s32k144::Peripherals::take().unwrap();

    // Disable watchdog
    let wdog_settings = wdog::WatchdogSettings {
        enable: false,
        ..Default::default()
    };
    let _wdog = wdog::Watchdog::init(&p.WDOG, wdog_settings).unwrap();

    let mut buffer: [u8; MSG_LEN] = [0; MSG_LEN];

    // Initialize CSEc module
    let csec = csec::CSEc::init(p.FTFC, p.CSE_PRAM);
    csec.init_rng().unwrap();
    csec.load_plainkey(&PLAINKEY).unwrap();

    // Encrypt `MSG`
    let rnd_buf = csec.generate_rnd().unwrap();
    buffer.copy_from_slice(MSG);
    csec.encrypt_cbc(&rnd_buf, &mut buffer).unwrap();

    // Decrypt `MSG` and verify it
    csec.decrypt_cbc(&rnd_buf, &mut buffer).unwrap();
    assert!(MSG == &buffer[..]);

    // Generate a MAC for `MSG` and verify it
    let cmac = csec.generate_mac(&MSG).unwrap();
    assert!(csec.verify_mac(&MSG, &cmac).unwrap());

    // light green LED
    let pcc = Pcc::init(&p.PCC);
    let pcc_portd = pcc.enable_portd().unwrap();
    let led = led::RgbLed::init(&p.PTD, &p.PORTD, &pcc_portd);
    led.set(false, false, true);

    loop {}
}
