extern crate cortex_m;

use bit_field::BitField;

use s32k144::{
    CAN0,
};

use s32k144::can0::EMBEDDEDRAM;

use s32k144::can0::mcr::IDAMW;

pub enum CanID {
    Standard(u16),
    Extended(u32),
}

pub struct CanMessage {
    id: CanID,
    dlc: u8,
    data: [u8; 8],
}

pub struct CanSettings {

    pub enable: bool,

    /// This bit controls whether the Rx FIFO feature is enabled or not. When RFEN is set, MBs 0 to 5 cannot be
    /// used for normal reception and transmission because the corresponding memory region (0x80-0xDC) is
    /// used by the FIFO engine as well as additional MBs (up to 32, depending on CAN_CTRL2[RFFN] setting)
    /// which are used as Rx FIFO ID Filter Table elements. RFEN also impacts the definition of the minimum
    /// number of peripheral clocks per CAN bit as described in the table "Minimum Ratio Between Peripheral
    /// Clock Frequency and CAN Bit Rate"
    pub fifo_enabled: bool,

    /// When asserted, this bit enables the generation of the TWRNINT and RWRNINT flags in the Error and
    /// Status Register 1 (ESR1). If WRNEN is negated, the TWRNINT and RWRNINT flags will always be zero,
    /// independent of the values of the error counters, and no warning interrupt will ever be generated. This bit
    /// can be written in Freeze mode only because it is blocked by hardware in other modes.
    pub warning_interrupt: bool,

    /// This bit defines whether FlexCAN is allowed to receive frames transmitted by itself. If this bit is asserted,
    /// frames transmitted by the module will not be stored in any MB, regardless if the MB is programmed with
    /// an ID that matches the transmitted frame, and no interrupt flag or interrupt signal will be generated due to
    /// the frame reception.
    pub self_reception: bool,

    /// This bit indicates whether Rx matching process will be based either on individual masking and queue or
    /// on masking scheme with CAN_RXMGMASK, CAN_RX14MASK, CAN_RX15MASK and
    /// CAN_RXFGMASK.
    pub individual_masking: bool,
    
    /// The DMA Enable bit controls whether the DMA feature is enabled or not. The DMA feature can only be
    /// used in Rx FIFO, consequently the bit CAN_MCR[RFEN] must be asserted. When DMA and RFEN are
    /// set, the CAN_IFLAG1[BUF5I] generates the DMA request and no RX FIFO interrupt is generated
    pub dma_enable: bool,

    /// This 2-bit field identifies the format of the Rx FIFO ID Filter Table elements. Note that all elements of the
    /// table are configured at the same time by this field (they are all the same format). See Section "Rx FIFO
    /// Structure".
    pub id_acceptance_mode: IdAcceptanceMode,

    /// Number Of The Last Message Buffer
    ///
    /// This 7-bit field defines the number of the last Message Buffers that will take part in the matching and
    /// arbitration processes. The reset value (0x0F) is equivalent to a 16 MB configuration.
    ///
    /// Additionally, the definition of MAXMB value must take into account the region of MBs occupied by Rx
    /// FIFO and its ID filters table space defined by RFFN bit in CAN_CTRL2 register. MAXMB also impacts the
    /// definition of the minimum number of peripheral clocks per CAN bit as described in Table "Minimum Ratio
    /// Between Peripheral Clock Frequency and CAN Bit Rate" 
    pub last_message_buffer: u8,

    /// This 8-bit field defines the ratio between the PE clock frequency and the Serial Clock (Sclock) frequency.
    /// The Sclock period defines the time quantum of the CAN protocol. For the reset value, the Sclock
    /// frequency is equal to the PE clock frequency. The Maximum value of this field is 0xFF, that gives a
    /// minimum Sclock frequency equal to the PE clock frequency divided by 256. See Section "Protocol
    /// Timing". This field can be written only in Freeze mode because it is blocked by hardware in other modes.
    /// Sclock frequency = PE clock frequency / (PRESDIV + 1)
    pub prescale_divisor: u8,

    /// This bit configures FlexCAN to operate in Loop-Back mode. In this mode, FlexCAN performs an internal
    /// loop back that can be used for self test operation. The bit stream output of the transmitter is fed back
    /// internally to the receiver input. The Rx CAN input pin is ignored and the Tx CAN output goes to the
    /// recessive state (logic 1). FlexCAN behaves as it normally does when transmitting, and treats its own
    /// transmitted message as a message received from a remote node.
    pub loopback_mode: bool,

    /// This bit selects the clock source to the CAN Protocol Engine (PE) to be either the peripheral clock or the
    /// oscillator clock. The selected clock is the one fed to the prescaler to generate the Serial Clock (Sclock). In
    /// order to guarantee reliable operation
    pub clock_source: ClockSource,

    pub source_frequency: u32,
    pub can_frequency: u32,
    
}

impl Default for CanSettings {
    fn default() -> Self {
        CanSettings{
            enable: false,
            fifo_enabled: false,
            warning_interrupt: false,
            self_reception: false,
            individual_masking: false,
            dma_enable: false,
            id_acceptance_mode: IdAcceptanceMode::FormatA,
            last_message_buffer: 0b0001111,
            prescale_divisor: 0,
            loopback_mode: false,
            can_frequency: 1000000,
            clock_source: ClockSource::Oscilator,
            source_frequency: 0,
        }
    }
}

pub enum ClockSource {
    Peripheral,
    Oscilator,
}

impl From<ClockSource> for bool {
    fn from(cs: ClockSource) -> bool {
        match cs {
            ClockSource::Peripheral => true,
            ClockSource::Oscilator => false,
        }
    }
}
    
 
pub enum IdAcceptanceMode {
    /// Format A: One full ID (standard and extended) per ID Filter Table element
    FormatA,
    /// Format B: Two full standard IDs or two partial 14-bit (standard and extended) IDs per ID Filter Table element.
    FormatB,
    /// Format C: Four partial 8-bit Standard IDs per ID Filter Table element.
    FormatC,
    /// Format D: All frames rejected.
    FormatD,
}

#[derive(Clone, Copy)]
pub enum MessageBufferCode {
    Receive(ReceiveBufferCode),
    Transmit(TransmitBufferCode),
}

#[derive(Clone, Copy)]
pub enum ReceiveBufferCode {
    /// MB is not active
    Inactive,

    /// MB is active and empty
    Empty,

    /// MB is full
    Full,

    /// MV is beeing overwritten into a full buffer
    Overrun,

    /// A frame was configured to recongnize a Remote Reuqest Frame and transmit a response Frame in return
    Ranswer,

    /// FlexCAN is updating the contents of the MB, the CPU must not access the MB
    Busy,
}

#[derive(Clone, Copy)]
pub enum TransmitBufferCode {
    /// MB is not active
    Inactive,

    /// MB is aborted
    Abort,

    /// MB is a tx data frame or tx RTR frame depending on RTR bit
    DataRemote,

    /// MV is a Tx response frame from an incoming RTR frame
    Tanswer,
}

impl From<MessageBufferCode> for u8 {
    fn from(code: MessageBufferCode) -> u8 {
        match code {
            MessageBufferCode::Receive(ref r) => match *r {
                ReceiveBufferCode::Inactive => 0b0000,
                ReceiveBufferCode::Empty => 0b0100,
                ReceiveBufferCode::Full => 0b0010,
                ReceiveBufferCode::Overrun => 0b0110,
                ReceiveBufferCode::Ranswer => 0b1010,
                ReceiveBufferCode::Busy => 0b0001, // really 0bxxx1
            },
            MessageBufferCode::Transmit(ref t) => match *t {
                TransmitBufferCode::Inactive => 0b1000,
                TransmitBufferCode::Abort => 0b1001,
                TransmitBufferCode::DataRemote => 0b1100,
                TransmitBufferCode::Tanswer => 0b1110,    
            },
        }
    }
}
   

fn enter_freeze(can: &CAN0) {
    can.mcr.modify(|_, w| w
                   .mdis()._1()
                   .frz()._1()
                   .halt()._1()
    );
}

pub enum CanError {
    FreezeModeError,
    SettingsError,
    ConfigurationFailed,
}

pub fn configure(settings: CanSettings) -> Result<(), CanError> {

    if settings.dma_enable && !settings.fifo_enabled {
        return Err(CanError::SettingsError);
    }            

    if settings.source_frequency % settings.can_frequency != 0 {
        return Err(CanError::SettingsError);
    }

    if settings.source_frequency < settings.can_frequency*5 {
        return Err(CanError::SettingsError);
    }
    
    let presdiv = (settings.source_frequency / settings.can_frequency) / 25;
    let tqs = ( settings.source_frequency / (presdiv + 1) ) / settings.can_frequency;

    // Table 50-26 in datasheet, can standard compliant settings
    let (pseg2, rjw) =
        if tqs >= 8 && tqs < 10 {
            (1, 1)
        } else if tqs >= 10 && tqs < 15 {
            (3, 2)
        } else if tqs >= 15 && tqs < 20 {
            (6, 2)
        } else if tqs >= 20 && tqs < 26 {
            (7, 3)
        } else {
            panic!("there should be between 8 and 25 tqs in an bit");
        };
    
    let pseg1 = ( (tqs - (pseg2 + 1) ) / 2 ) - 1;
    let propseg = tqs - (pseg2 + 1) - (pseg1 + 1) - 1;
            

    cortex_m::interrupt::free(|cs| {
        
        let can = CAN0.borrow(cs);
        enter_freeze(can);

        // TODO: add wait for freeze mode
        
        can.mcr.modify(|_, w| { w
                                .mdis().bit(!settings.enable)
                                .rfen().bit(settings.fifo_enabled)
                                .srxdis().bit(settings.self_reception)
                                .irmq().bit(settings.individual_masking)
                                .dma().bit(settings.dma_enable);

                                match settings.id_acceptance_mode {
                                    IdAcceptanceMode::FormatA => w.idam().variant(IDAMW::_00),
                                    IdAcceptanceMode::FormatB => w.idam().variant(IDAMW::_01),
                                    IdAcceptanceMode::FormatC => w.idam().variant(IDAMW::_10),
                                    IdAcceptanceMode::FormatD => w.idam().variant(IDAMW::_11),
                                };
                                
                                unsafe { w.maxmb().bits(settings.last_message_buffer) };

                                w
        });
        
        can.ctrl1.modify(|_, w| { unsafe { w
                                           .presdiv().bits(settings.prescale_divisor)
                                           .pseg1().bits(pseg1 as u8)
                                           .pseg2().bits(pseg2 as u8)
                                           .propseg().bits(propseg as u8)
                                           .rjw().bits(rjw as u8)
                                           .lpb().bit(settings.loopback_mode)                                
        }});

        // TODO: Remember to enable and recover from freeze, but first do something to the message boxes

        // Make some acceptance test to see if the configurations have been applied

        return Ok(());
    })       
}
 

struct MailboxSettings{
    code: MessageBufferCode,
    rtr: bool,
    ide: bool,
    id: u32,
}
                              
unsafe fn mailbox_write(embedded_ram: &[EMBEDDEDRAM], start_adress: usize, settings: MailboxSettings, data: &[u8]) {
    embedded_ram[start_adress].write(|w| w.bits(
        0u32.set_bits(24..28, u8::from(settings.code.clone()) as u32)
            .set_bit(21, settings.ide)
            .set_bit(20, settings.rtr)
            .set_bits(16..20, data.len() as u32)
            .get_bits(0..32)
    ));

    if settings.ide {
        embedded_ram[start_adress+1].write(|w| w.bits(
            0u32.set_bits(0..29, settings.id)
                .get_bits(0..32)
        ));
    } else {
        embedded_ram[start_adress+1].write(|w| w.bits(
            0u32.set_bits(18..29, settings.id)
                .get_bits(0..32)
        ));
    }

    for index in 0..data.len() {
        embedded_ram[start_adress+2 + index/4].modify(|r, w| {
            let mut bitmask = r.bits();
            bitmask.set_bits((8*index%4) as u8..(8*(1+index%4)) as u8, data[index] as u32);
            w.bits(bitmask)
        });
    }   

}
