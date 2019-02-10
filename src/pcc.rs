//! The peripheral clock controller (PCC) SW module

use s32k144;

#[derive(Debug, PartialEq)]
pub enum Error {
    RegisterNotPresent,
    AlreadyEnabled,
}

#[derive(Debug, PartialEq)]
/// Clock source
///
/// Is used on the following peripherals
/// - LPSPI
/// - LPIT
/// - FlexIO
/// - LPI2C
/// - LPUART
pub enum ClockSource {
    None,
    Soscdiv2,
    Sircdiv2,
    Fircdiv2,
    Splldiv2,
}

impl From<ClockSource> for u8 {
    fn from(cs: ClockSource) -> u8 {
        match cs {
            ClockSource::None => 0b000,
            ClockSource::Soscdiv2 => 0b001,
            ClockSource::Sircdiv2 => 0b010,
            ClockSource::Fircdiv2 => 0b011,
            ClockSource::Splldiv2 => 0b110,
        }
    }
}

pub struct PortC<'a> {
    pcc: &'a s32k144::pcc::RegisterBlock,
}

pub struct PortD<'a> {
    pcc: &'a s32k144::pcc::RegisterBlock,
}

pub struct PortE<'a> {
    pcc: &'a s32k144::pcc::RegisterBlock,
}

pub struct Lpuart1<'a> {
    pcc: &'a s32k144::pcc::RegisterBlock,
}

pub struct Can0<'a> {
    pcc: &'a s32k144::pcc::RegisterBlock,
}

pub struct Pcc<'a> {
    pcc: &'a s32k144::pcc::RegisterBlock,
}

impl<'a> Pcc<'a> {
    pub fn init(pcc: &'a s32k144::pcc::RegisterBlock) -> Self {
        Pcc { pcc: pcc }
    }

    pub fn enable_portc(&self) -> Result<PortC, Error> {
        let reg_value = self.pcc.pcc_portc.read();
        if reg_value.pr().is_0() {
            Err(Error::RegisterNotPresent)
        } else if reg_value.cgc().is_1() {
            Err(Error::AlreadyEnabled)
        } else {
            self.pcc.pcc_portc.modify(|_, w| w.cgc()._1());
            Ok(PortC { pcc: self.pcc })
        }
    }

    pub fn enable_portd(&self) -> Result<PortD, Error> {
        let reg_value = self.pcc.pcc_portd.read();
        if reg_value.pr().is_0() {
            Err(Error::RegisterNotPresent)
        } else if reg_value.cgc().is_1() {
            Err(Error::AlreadyEnabled)
        } else {
            self.pcc.pcc_portd.modify(|_, w| w.cgc()._1());
            Ok(PortD { pcc: self.pcc })
        }
    }

    pub fn enable_porte(&self) -> Result<PortE, Error> {
        let reg_value = self.pcc.pcc_porte.read();
        if reg_value.pr().is_0() {
            Err(Error::RegisterNotPresent)
        } else if reg_value.cgc().is_1() {
            Err(Error::AlreadyEnabled)
        } else {
            self.pcc.pcc_porte.modify(|_, w| w.cgc()._1());
            Ok(PortE { pcc: self.pcc })
        }
    }

    pub fn enable_lpuart1(&self, source: ClockSource) -> Result<Lpuart1, Error> {
        let reg_value = self.pcc.pcc_lpuart1.read();
        if reg_value.pr().is_0() {
            Err(Error::RegisterNotPresent)
        } else if reg_value.cgc().is_1() {
            Err(Error::AlreadyEnabled)
        } else {
            self.pcc
                .pcc_lpuart1
                .modify(|_, w| w.pcs().bits(u8::from(source)));
            self.pcc.pcc_lpuart1.modify(|_, w| w.cgc()._1());
            Ok(Lpuart1 { pcc: self.pcc })
        }
    }

    pub fn enable_can0(&self) -> Result<Can0, Error> {
        let reg_value = self.pcc.pcc_flex_can0.read();
        if reg_value.pr().is_0() {
            Err(Error::RegisterNotPresent)
        } else if reg_value.cgc().is_1() {
            Err(Error::AlreadyEnabled)
        } else {
            self.pcc.pcc_flex_can0.modify(|_, w| w.cgc()._1());
            Ok(Can0 { pcc: self.pcc })
        }
    }
}

impl<'a> Drop for PortC<'a> {
    fn drop(&mut self) {
        self.pcc.pcc_portc.reset();
    }
}

impl<'a> Drop for PortD<'a> {
    fn drop(&mut self) {
        self.pcc.pcc_portd.reset();
    }
}

impl<'a> Drop for PortE<'a> {
    fn drop(&mut self) {
        self.pcc.pcc_porte.reset();
    }
}

impl<'a> Drop for Lpuart1<'a> {
    fn drop(&mut self) {
        self.pcc.pcc_lpuart1.reset();
    }
}

impl<'a> Drop for Can0<'a> {
    fn drop(&mut self) {
        self.pcc.pcc_flex_can0.reset();
    }
}
