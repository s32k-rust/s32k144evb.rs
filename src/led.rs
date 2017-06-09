extern crate cortex_m;

use s32k144::{PORTD, PTD};

pub static RED: Led = Led{pin: 15};
pub static GREEN: Led = Led{pin: 16};
pub static BLUE: Led = Led{pin: 0};

pub struct Led {
    pin: u8,
}

impl Led {
    pub fn on(&self) {
        cortex_m::interrupt::free(|cs| {
            let ptd = PTD.borrow(cs);
            ptd.pcor.write(|w| unsafe{ w.ptco().bits(1<<self.pin) } );
        });
    }

    pub fn off(&self) {
        cortex_m::interrupt::free(|cs| {
            let ptd = PTD.borrow(cs);
            ptd.psor.write(|w| unsafe{ w.ptso().bits(1<<self.pin) } );
        });
    }
    
}
        


pub fn init() {
    cortex_m::interrupt::free(|cs | {
        let portd = PORTD.borrow(cs);
        let ptd = PTD.borrow(cs);        

        ptd.pddr.write(|w| unsafe{ w.pdd().bits(ptd.pddr.read().bits() | (1<<0) | (1<<15) | (1<<16) ) } );
        
        portd.pcr0.modify(|_, w| w.mux().bits(0b001));
        portd.pcr0.modify(|_, w| w.dse().bits(0b1));
        
        portd.pcr15.modify(|_, w| w.mux().bits(0b001));
        portd.pcr15.modify(|_, w| w.dse().bits(0b1));

        portd.pcr16.modify(|_, w| w.mux().bits(0b001));
        portd.pcr16.modify(|_, w| w.dse().bits(0b1));
    });
}
