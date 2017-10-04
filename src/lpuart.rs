use s32k144::LPUART1;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UartSettings {
    pub baudrate: u32,
    pub data_bits: DataBits,
    pub parity: Parity,
    pub stop_bits: StopBits,
}

impl Default for UartSettings {
    fn default() -> Self {
        UartSettings{
            baudrate: 9600,
            data_bits: DataBits::B8,
            stop_bits: StopBits::B1,
            parity: Parity::N,
        }            
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DataBits {
    B7 = 7,
    B8 = 8,
    B9 = 9,
    B10 = 10,        
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StopBits {
    B1 = 1,
    B2 = 2,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Parity {
    N,
    E,
    O,
}

pub fn configure(lpuart: &LPUART1, settings: UartSettings, source_frequency: u32) {
    // disable receiver and transmiter
    lpuart.ctrl.modify(|_r, w| w
                       .te().clear_bit()
                       .re().clear_bit()
    );

    // TODO: check that divisor is a sensible value
    let divisor = source_frequency / (settings.baudrate * 16);
    
    lpuart.baud.write(|w| unsafe{ w
                                  .m10().bit(settings.data_bits == DataBits::B10)
                                  .sbns().bit(settings.stop_bits == StopBits::B2)
                                  .sbr().bits(divisor as u16)
    });

    lpuart.ctrl.write(|w| w
                      .m7().bit(settings.data_bits == DataBits::B7)
                      .m().bit(settings.data_bits == DataBits::B9)
                      .pe().bit(settings.parity != Parity::N)
                      .pt().bit(settings.parity == Parity::O)
    );

    // enable receiver and transmitter 
    lpuart.ctrl.modify(|_r, w| w
                       .te().set_bit()
                       //.re().set_bit()
    );

}

pub fn transmit(lpuart: &LPUART1, data: u8) {
    lpuart.data.write(|w| unsafe{w.bits(data as u32)});
}

