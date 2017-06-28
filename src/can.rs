
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

    /// The DMA Enable bit controls whether the DMA feature is enabled or not. The DMA feature can only be
    /// used in Rx FIFO, consequently the bit CAN_MCR[RFEN] must be asserted. When DMA and RFEN are
    /// set, the CAN_IFLAG1[BUF5I] generates the DMA request and no RX FIFO interrupt is generated
    pub dma_enable: bool,

    /// This bit enables the Pretended Networking functionality. Once in Stop mode, CAN_PE sub-block is kept
    /// operational, able to process Rx message filtering as defined by the Pretended Networking configuration
    /// registers.
    pub pretend_networking: bool,

    /// This bit enables the CAN with Flexible Data rate (CAN FD) operation
    pub can_fd: bool,

    /// This 2-bit field identifies the format of the Rx FIFO ID Filter Table elements. Note that all elements of the
    /// table are configured at the same time by this field (they are all the same format). See Section "Rx FIFO
    /// Structure".
    pub id_acceptance_mode: IdAcceptanceMode,

    /// Number Of The Last Message Buffer
    ///
    /// This 7-bit field defines the number of the last Message Buffers that will take part in the matching and
    /// arbitration processes. The reset value (0x0F) is equivalent to a 16 MB configuration.
    pub last_message_buffer: u8,

    /// This 8-bit field defines the ratio between the PE clock frequency and the Serial Clock (Sclock) frequency.
    /// The Sclock period defines the time quantum of the CAN protocol. For the reset value, the Sclock
    /// frequency is equal to the PE clock frequency. The Maximum value of this field is 0xFF, that gives a
    /// minimum Sclock frequency equal to the PE clock frequency divided by 256. See Section "Protocol
    /// Timing". This field can be written only in Freeze mode because it is blocked by hardware in other modes.
    /// Sclock frequency = PE clock frequency / (PRESDIV + 1)
    pub prescale_factor: u8,

    /// This bit configures FlexCAN to operate in Loop-Back mode. In this mode, FlexCAN performs an internal
    /// loop back that can be used for self test operation. The bit stream output of the transmitter is fed back
    /// internally to the receiver input. The Rx CAN input pin is ignored and the Tx CAN output goes to the
    /// recessive state (logic 1). FlexCAN behaves as it normally does when transmitting, and treats its own
    /// transmitted message as a message received from a remote node.
    pub loopback_mode: bool,
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
    
