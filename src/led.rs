extern crate cortex_m;

use s32k144;

pub struct RgbLed<'a>(&'a s32k144::ptd::RegisterBlock);

impl<'a> RgbLed<'a> {
    const RED_PIN: u32 = 15;
    const GREEN_PIN: u32 = 16;
    const BLUE_PIN: u32 = 0;

    pub fn init(ptd: &'a s32k144::ptd::RegisterBlock, portd: &'a s32k144::portd::RegisterBlock) -> Self {
        ptd.pddr.write(|w| unsafe{ w.pdd().bits(ptd.pddr.read().bits() | (1<<0) | (1<<15) | (1<<16) ) } );
        
        portd.pcr0.modify(|_, w| w.mux().bits(0b001));
        portd.pcr0.modify(|_, w| w.dse()._1());
        portd.pcr0.modify(|_, w| w.pe()._0());
            
        portd.pcr15.modify(|_, w| w.mux().bits(0b001));
        portd.pcr15.modify(|_, w| w.dse()._1());
        portd.pcr15.modify(|_, w| w.pe()._0());

        portd.pcr16.modify(|_, w| w.mux().bits(0b001));
        portd.pcr16.modify(|_, w| w.dse()._1());
        portd.pcr16.modify(|_, w| w.pe()._0());

        RgbLed(ptd)
    }

    pub fn set(&self, red: bool, blue: bool, green: bool) {
        if red {
            self.0.pcor.write(|w| unsafe{ w.ptco().bits(1<<Self::RED_PIN) } );
        } else {
            self.0.psor.write(|w| unsafe{ w.ptso().bits(1<<Self::RED_PIN) } );
        }
        if green {
            self.0.pcor.write(|w| unsafe{ w.ptco().bits(1<<Self::GREEN_PIN) } );
        } else {
            self.0.psor.write(|w| unsafe{ w.ptso().bits(1<<Self::GREEN_PIN) } );
        }
        if blue {
            self.0.pcor.write(|w| unsafe{ w.ptco().bits(1<<Self::BLUE_PIN) } );
        } else {
            self.0.psor.write(|w| unsafe{ w.ptso().bits(1<<Self::BLUE_PIN) } );
        }
    }

    pub fn off(&self) {
    }
    
}
        


