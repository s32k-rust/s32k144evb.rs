use s32k144::LPUART1;

pub enum UartError {
    UnsatisfiableBaud,
}

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

pub fn configure(lpuart: &LPUART1, settings: UartSettings, source_frequency: u32) -> Result<(), UartError> {
    // disable receiver and transmiter
    lpuart.ctrl.modify(|_r, w| w
                       .te().clear_bit()
                       .re().clear_bit()
    );

    // TODO: check that divisor is a sensible value
    let (oversampling_ratio, divisor) = find_decent_div(source_frequency, settings.baudrate)?;
    let bothedge = oversampling_ratio < 8;
    
    lpuart.baud.write(|w| unsafe{ w
                                  .m10().bit(settings.data_bits == DataBits::B10)
                                  .sbns().bit(settings.stop_bits == StopBits::B2)
                                  .bothedge().bit(bothedge)
                                  .osr().bits(oversampling_ratio-1)
                                  .sbr().bits(divisor as u16)
    });

    lpuart.ctrl.write(|w| w
                      .m7().bit(settings.data_bits == DataBits::B7)
                      .m().bit(settings.data_bits == DataBits::B9)
                      .pe().bit(settings.parity != Parity::N)
                      .pt().bit(settings.parity == Parity::O)
    );

    lpuart.fifo.write(|w| w
                      .txfe()._1()
    );
    
    // enable receiver and transmitter 
    lpuart.ctrl.modify(|_r, w| w
                       .te().set_bit()
                       //.re().set_bit()
    );

    Ok(())
}

pub fn transmit(lpuart: &LPUART1, data: u8) {
    lpuart.data.write(|w| unsafe{w.bits(data as u32)});
}

fn find_decent_div(source: u32, baud: u32) -> Result<(u8, u16), UartError> {
    const OVERSAMPLING_MIN: u32 = 4;
    const OVERSAMPLING_MAX: u32 = 32;

    const DIV_MIN: u32 = 1;
    const DIV_MAX: u32 = 8191;
    
    let ratio = (source + baud/2)/baud;
    let alternative_ratio = {
        if ratio == source/baud {
            ratio + 1
        } else {
            ratio - 1
        }
    };

    for i in (OVERSAMPLING_MIN..OVERSAMPLING_MAX+1).rev() {
        if ratio%i == 0 && ratio/i >= DIV_MIN && ratio/i <= DIV_MAX {
            return Ok((i as u8, (ratio/i) as u16));
        }
    }

    for i in (OVERSAMPLING_MIN..OVERSAMPLING_MAX+1).rev() {
        if alternative_ratio%i == 0 && alternative_ratio/i >= DIV_MIN && alternative_ratio/i <= DIV_MAX {
            return Ok((i as u8, (alternative_ratio/i) as u16));
        }
    }

    Err(UartError::UnsatisfiableBaud)
}

