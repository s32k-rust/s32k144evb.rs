use cortex_m;

use bit_field::BitField;

use s32k144::{
    CAN0,
    PCC,
    PORTE,
};

use s32k144::can0::mcr::IDAMW;

pub trait CanFrame {
    fn with_data(id: u32, extended_id: bool, data: &[u8]) -> Self;
    fn extended_id(&self) -> bool;
    fn id(&self) -> u32;
    fn data(&self) -> &[u8];
}
    
pub struct CanSettings {

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
            fifo_enabled: false,
            warning_interrupt: false,
            self_reception: true,
            individual_masking: false,
            dma_enable: false,
            id_acceptance_mode: IdAcceptanceMode::FormatA,
            loopback_mode: false,
            can_frequency: 1000000,
            clock_source: ClockSource::Oscilator,
            source_frequency: 0,
        }
    }
}

#[derive(Clone, Copy)]
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

#[derive(Clone, PartialEq)]
pub enum MessageBufferCode {
    Receive(ReceiveBufferCode),
    Transmit(TransmitBufferState),
}

#[derive(Clone, PartialEq)]
pub struct ReceiveBufferCode {
    pub state: ReceiveBufferState,
    /// FlexCAN is updating the contents of the MB, the CPU must not access the MB
    pub busy: bool,
}

#[derive(Clone, PartialEq)]
pub enum ReceiveBufferState {
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
}

#[derive(Clone, PartialEq)]
pub enum TransmitBufferState {
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
            MessageBufferCode::Receive(ref r) => match r.state {
                ReceiveBufferState::Inactive => 0u8.set_bit(0, r.busy).set_bits(1..4, 0b000).get_bits(0..4),
                ReceiveBufferState::Empty => 0u8.set_bit(0, r.busy).set_bits(1..4, 0b010).get_bits(0..4),
                ReceiveBufferState::Full => 0u8.set_bit(0, r.busy).set_bits(1..4, 0b001).get_bits(0..4),
                ReceiveBufferState::Overrun => 0u8.set_bit(0, r.busy).set_bits(1..4, 0b011).get_bits(0..4),
                ReceiveBufferState::Ranswer => 0u8.set_bit(0, r.busy).set_bits(1..4, 0b101).get_bits(0..4),
            },
            MessageBufferCode::Transmit(ref t) => match *t {
                TransmitBufferState::Inactive => 0b1000,
                TransmitBufferState::Abort => 0b1001,
                TransmitBufferState::DataRemote => 0b1100,
                TransmitBufferState::Tanswer => 0b1110,    
            },
        }
    }
}

impl From<u8> for MessageBufferCode {
    fn from(code: u8) -> Self {
        match code {
            0b1000 => MessageBufferCode::Transmit(TransmitBufferState::Inactive),
            0b1001 => MessageBufferCode::Transmit(TransmitBufferState::Abort),
            0b1100 => MessageBufferCode::Transmit(TransmitBufferState::DataRemote),
            0b1110 => MessageBufferCode::Transmit(TransmitBufferState::Tanswer),
            0b0000 => MessageBufferCode::Receive(ReceiveBufferCode{state: ReceiveBufferState::Inactive, busy: false}),
            0b0001 => MessageBufferCode::Receive(ReceiveBufferCode{state: ReceiveBufferState::Inactive, busy: true}),
            0b0100 => MessageBufferCode::Receive(ReceiveBufferCode{state: ReceiveBufferState::Empty, busy: false}),
            0b0101 => MessageBufferCode::Receive(ReceiveBufferCode{state: ReceiveBufferState::Empty, busy: true}),
            0b0010 => MessageBufferCode::Receive(ReceiveBufferCode{state: ReceiveBufferState::Full, busy: false}),
            0b0011 => MessageBufferCode::Receive(ReceiveBufferCode{state: ReceiveBufferState::Full, busy: true}),
            0b0110 => MessageBufferCode::Receive(ReceiveBufferCode{state: ReceiveBufferState::Overrun, busy: false}),
            0b0111 => MessageBufferCode::Receive(ReceiveBufferCode{state: ReceiveBufferState::Overrun, busy: true}),
            0b1010 => MessageBufferCode::Receive(ReceiveBufferCode{state: ReceiveBufferState::Ranswer, busy: false}),
            0b1011 => MessageBufferCode::Receive(ReceiveBufferCode{state: ReceiveBufferState::Ranswer, busy: true}),
            _ => panic!("Value: {}, is not a valid MessageBufferCode", code),
        }
    }
}


pub struct MessageBufferHeader {
    /// This bit distinguishes between CAN format and CAN FD format frames. The EDL bit
    /// must not be set for Message Buffers configured to RANSWER with code field 0b1010
    pub extended_data_length: bool,

    /// This bit defines whether the bit rate is switched inside a CAN FD format frame
    pub bit_rate_switch: bool,

    /// This bit indicates if the transmitting node is error active or error passive.
    pub error_state_indicator: bool,

    /// This 4-bit field can be accessed (read or write) by the CPU and by the FlexCAN module
    /// itself, as part of the message buffer matching and arbitration process.
    pub code: MessageBufferCode,

    /// Fixed recessive bit, used only in extended format. It must be set to one by the user for
    /// transmission (Tx Buffers) and will be stored with the value received on the CAN bus for
    /// Rx receiving buffers. It can be received as either recessive or dominant. If FlexCAN
    /// receives this bit as dominant, then it is interpreted as an arbitration loss.
    pub substitute_remote_request: bool,

    /// This field identifies whether the frame format is standard or extended.
    pub id_extended: bool,

    /// This bit affects the behavior of remote frames and is part of the reception filter. See Table
    /// 50-10, Table 50-11, (in datasheet) and the description of the RRS bit in Control 2 Register
    /// (CAN_CTRL2) for additional details.
    pub remote_transmission_request: bool,

    /// This 4-bit field is the length (in bytes) of the Rx or Tx data, which is located in offset 0x8
    /// through 0xF of the MB space (see Table 50-9). In reception, this field is written by the
    /// FlexCAN module, copied from the DLC (Data Length Code) field of the received frame.
    /// In transmission, this field is written by the CPU and corresponds to the DLC field value
    /// of the frame to be transmitted. When RTR = 1, the frame to be transmitted is a remote
    /// frame and does not include the data field, regardless of the DLC field (see Table 50-12).
    pub data_length_code: u8,

    /// This 16-bit field is a copy of the Free-Running Timer, captured for Tx and Rx frames at
    /// the time when the beginning of the Identifier field appears on the CAN bus
    pub time_stamp: u16,

    /// This 3-bit field is used only when LPRIO_EN bit is set in CAN_MCR, and it only makes
    /// sense for Tx mailboxes. These bits are not transmitted. They are appended to the regular
    /// ID to define the transmission priority.
    pub priority: u8,

    /// In standard frame format, only the 11 most significant bits (28 to 18) are used for frame
    /// identification in both receive and transmit cases. The 18 least significant bits are ignored.
    /// In extended frame format, all bits are used for frame identification in both receive and
    /// transmit cases.
    pub id: u32,
}

impl MessageBufferHeader {
    pub fn default_transmit() -> Self {
        MessageBufferHeader{
            extended_data_length: false,
            bit_rate_switch: false,
            error_state_indicator: false,
            code: MessageBufferCode::Transmit(TransmitBufferState::Inactive),
            substitute_remote_request: false,
            id_extended: false,
            remote_transmission_request: false,
            data_length_code: 0,
            time_stamp: 0,
            priority: 0,
            id: 0,
        }
    }

    pub fn default_receive() -> Self {
        MessageBufferHeader{
            extended_data_length: false,
            bit_rate_switch: false,
            error_state_indicator: false,
            code: MessageBufferCode::Receive(ReceiveBufferCode{state: ReceiveBufferState::Empty, busy: false}),
            substitute_remote_request: false,
            id_extended: false,
            remote_transmission_request: false,
            data_length_code: 0,
            time_stamp: 0,
            priority: 0,
            id: 0,
        }
    }
}


fn enable(can: &CAN0) {
    can.mcr.modify(|_, w| w.mdis()._0());
    while can.mcr.read().lpmack().is_1() {}
}

fn reset(can: &CAN0) {
    can.mcr.modify(|_, w| w.mdis()._1());
    while can.mcr.read().lpmack().is_0() {}
    can.ctrl1.modify(|_, w| w.clksrc()._1());
    can.mcr.modify(|_, w| w.mdis()._0());
    while can.mcr.read().lpmack().is_1() {}
    can.mcr.modify(|_, w| w.softrst()._1());
    while can.mcr.read().softrst().is_1() {}
    can.mcr.modify(|_, w| w.mdis()._1());
    while can.mcr.read().lpmack().is_0() {}
}

fn enter_freeze(can: &CAN0) {
    can.mcr.modify(|_, w| w
                   .frz()._1()
                   .halt()._1()
    );
    while can.mcr.read().frzack().is_0() {}
}

fn leave_freeze(can: &CAN0) {
    can.mcr.modify(|_, w| w
                   .halt()._0()
                   .frz()._0()
    );
    while can.mcr.read().frzack().is_1() {}
}    

#[derive(Debug)]
pub enum CanError {
    FreezeModeError,
    SettingsError,
    ConfigurationFailed,
}

pub fn init(settings: &CanSettings, message_buffer_settings: &[MessageBufferHeader]) -> Result<(), CanError> {

    if settings.dma_enable && !settings.fifo_enabled {
        return Err(CanError::SettingsError);
    }            

    if settings.source_frequency % settings.can_frequency != 0 {
        return Err(CanError::SettingsError);
    }

    if settings.source_frequency < settings.can_frequency*5 {
        return Err(CanError::SettingsError);
    }

    // TODO: check if message_buffer_settings are longer than max MB available
    
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
    let propseg = tqs - (pseg2 + 1) - (pseg1 + 1) - 2;
            

    cortex_m::interrupt::free(|cs| {
        
        let can = CAN0.borrow(cs);
        let pcc = PCC.borrow(cs);
        let porte = PORTE.borrow(cs);
        
        // Configure the can i/o pins
        pcc.pcc_porte.modify(|_, w| w.cgc()._1());
        porte.pcr4.modify(|_, w| w.mux()._101());
        porte.pcr5.modify(|_, w| w.mux()._101());
        
        pcc.pcc_flex_can0.modify(|_, w| w.cgc()._1());

        reset(can);

        // first set clock source
        can.ctrl1.modify(|_, w| w.clksrc().bit(settings.clock_source.clone().into()));
        
        enable(can);
        enter_freeze(can);
        
        
        can.mcr.modify(|_, w| { w
                                .rfen().bit(settings.fifo_enabled)
                                .srxdis().bit(!settings.self_reception)
                                .irmq().bit(settings.individual_masking)
                                .dma().bit(settings.dma_enable);

                                match settings.id_acceptance_mode {
                                    IdAcceptanceMode::FormatA => w.idam().variant(IDAMW::_00),
                                    IdAcceptanceMode::FormatB => w.idam().variant(IDAMW::_01),
                                    IdAcceptanceMode::FormatC => w.idam().variant(IDAMW::_10),
                                    IdAcceptanceMode::FormatD => w.idam().variant(IDAMW::_11),
                                };
                                
                                unsafe { w.maxmb().bits(message_buffer_settings.len() as u8-1) };

                                w
        });
        
        can.ctrl1.modify(|_, w| { unsafe { w
                                           .presdiv().bits(presdiv as u8)
                                           .pseg1().bits(pseg1 as u8)
                                           .pseg2().bits(pseg2 as u8)
                                           .propseg().bits(propseg as u8)
                                           .rjw().bits(rjw as u8)
                                           .lpb().bit(settings.loopback_mode)                                
        }});

        // set filter mask to accept all
        // TODO: Make better logic for setting filters
        can.rxmgmask.write(unsafe {|w| w.bits(0)});

        /*
        • Initialize the Message Buffers
        • The Control and Status word of all Message Buffers must be initialized
        • If Rx FIFO was enabled, the ID filter table must be initialized
        • Other entries in each Message Buffer should be initialized as required
         */

        for mb in 0..message_buffer_settings.len() {
            configure_messagebuffer(can, &message_buffer_settings[mb], mb as usize);
        }

        leave_freeze(can);

        // Make some acceptance test to see if the configurations have been applied

        return Ok(());
    })       
}

fn configure_messagebuffer(can: &CAN0, header: &MessageBufferHeader, mailbox: usize) {
    let start_adress = mailbox*4;

    can.embedded_ram[start_adress + 0].write(|w| unsafe{ w.bits(0u32
                                                                .set_bit(31, header.extended_data_length)
                                                                .set_bit(30, header.bit_rate_switch)
                                                                .set_bit(29, header.error_state_indicator)
                                                                .set_bits(24..28, u8::from(header.code.clone()) as u32)
                                                                .set_bit(22, header.substitute_remote_request)
                                                                .set_bit(21, header.id_extended)
                                                                .set_bit(20, header.remote_transmission_request)
                                                                .set_bits(16..20, header.data_length_code as u32)
                                                                .set_bits(0..15, header.time_stamp as u32)
                                                                .get_bits(0..32))
    });
                                                        
    can.embedded_ram[start_adress + 1].write(|w| {
        let mut bm = 0u32;
        bm.set_bits(29..32, header.priority as u32);
        if header.id_extended {
            bm.set_bits(0..29, header.id);
        } else {
            bm.set_bits(18..29, header.id);
        }
        unsafe{w.bits(bm)}
    });
}

#[derive(Debug)]
pub enum TransmitError {
    MailboxBusy,
    MailboxConfigurationError,
    MailboxNonExisting,
}

pub fn transmit<T: CanFrame>(message: &T, mailbox: usize) -> Result<(), TransmitError> {
    let start_adress = mailbox*4;
    
    cortex_m::interrupt::free(|cs| {

        let can = CAN0.borrow(cs);

        // 1. Check whether the respective interrupt bit is set and clear it.
        can.iflag1.write(|w| unsafe{w.bits(1<<mailbox)} );
        
        /* 2. If the MB is active (transmission pending), write the ABORT code (0b1001) to the
        CODE field of the Control and Status word to request an abortion of the
        transmission. Wait for the corresponding IFLAG bit to be asserted by polling the
        CAN_IFLAG register or by the interrupt request if enabled by the respective IMASK
        bit. Then read back the CODE field to check if the transmission was aborted or
        transmitted (see Transmission abort mechanism). If backwards compatibility is
        desired (CAN_MCR[AEN] bit is negated), just write the INACTIVE code (0b1000)
        to the CODE field to inactivate the MB but then the pending frame may be
        transmitted without notification (see Mailbox inactivation). */
        let current_code = can.embedded_ram[start_adress].read().bits().get_bits(24..28) as u8;

        if MessageBufferCode::from(current_code) == MessageBufferCode::Transmit(TransmitBufferState::DataRemote) {
            return Err(TransmitError::MailboxBusy);
        } else if MessageBufferCode::from(current_code) != MessageBufferCode::Transmit(TransmitBufferState::Inactive) && MessageBufferCode::from(current_code) != MessageBufferCode::Receive(ReceiveBufferCode{state: ReceiveBufferState::Inactive, busy: false}) {
            return Err(TransmitError::MailboxConfigurationError);
        }
        
        // 3. Write the ID word.
        if message.extended_id() {
            unsafe {can.embedded_ram[start_adress+1].modify(|_, w| w.bits(
                0u32.set_bits(0..29, message.id())
                    .get_bits(0..32)
            ))};
        } else {
            unsafe {can.embedded_ram[start_adress+1].modify(|_, w| w.bits(
                0u32.set_bits(18..29, message.id())
                    .get_bits(0..32)
            ))};
        }
        
        // 4. Write the data bytes.
        for index in 0..message.data().len() as usize {
            can.embedded_ram[start_adress+2 + index/4].modify(|r, w| {
                let mut bitmask = r.bits();
                bitmask.set_bits(32-(8*(1+index%4)) as u8..(32-8*(index%4)) as u8, message.data()[index] as u32);
                unsafe{ w.bits(bitmask) }
            });
        }   

        
        // 5. Write the DLC, Control, and CODE fields of the Control and Status word to activate
        // the MB. When CAN_MCR[FDEN] is set, write also the EDL, BRS and ESI bits.
        can.embedded_ram[start_adress].write(|w| unsafe {w.bits(
            0u32.set_bits(24..28, u8::from(MessageBufferCode::Transmit(TransmitBufferState::DataRemote)) as u32)
                .set_bit(21, message.extended_id())
                .set_bits(16..20, message.data().len() as u32)
                .get_bits(0..32)
        )});
        
        Ok(())
    })
}
