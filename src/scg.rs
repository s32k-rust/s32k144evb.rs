//! The System Clock Generator module

use s32k144;

/// Configurations for the System Clock Generator
#[derive(Default, Debug, PartialEq, Clone)]
pub struct Config {
    /// Set the configuration of XTAL and EXTAL pins.
    pub system_oscillator: SystemOscillatorInput,

    /// Set the divider for the soscdiv1_clk
    ///
    /// This should be configured to 40MHz or less in RUN/HSRUN mode.
    pub soscdiv1: SystemOscillatorOutput,

    /// Set the divider for the soscdiv1_clk
    ///
    /// This should be configured to 40MHz or less in RUN/HSRUN mode.
    pub soscdiv2: SystemOscillatorOutput,
}

/// Set the configuration of XTAL and EXTAL pins.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SystemOscillatorInput {
    /// Neither a crystal oscillator nor an external clock is connected.
    None,

    /// A crystal oscillator is connected between XTAL and EXTAL pins.
    ///
    /// The `u32` value specifies the oscillator frequency in Hz
    Crystal(u32),

    /// An external clock reference is connected to the EXTAL pins.
    ///
    /// The `u32` value specifies the reference frequency in Hz
    Reference(u32),
}

impl SystemOscillatorInput {
    pub(crate) fn clock_frequency(&self) -> Option<u32> {
        match *self {
            SystemOscillatorInput::Crystal(f) | SystemOscillatorInput::Reference(f) => Some(f),
            SystemOscillatorInput::None => None,
        }
    }
}

impl Default for SystemOscillatorInput {
    fn default() -> Self {
        SystemOscillatorInput::None
    }
}

/// SCG Run Modes
///
/// See section 26.4.1 in datasheet for a full description
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    /// Run mode
    ///
    /// - `CORE_CLK` and `SYS_CLK` clock freuqency must be 80M Hz or less (but not configured to be less than `BUS_CLK`).
    /// - `BUS_CLK` clock frequency must be 48 Mhz or less (when using PLL as system clock source maximum bus clock frequency is 40 MHz).
    /// - `FLASH_CLK` clock frequency must be 26.67 MHz or less.
    /// - The core clock to flash clock ratio is limited to a max value of 8.
    Run(RunMode),

    /// High Speed Run mode
    ///
    /// - `CORE_CLK` and `SYS_CLK` clock freuqency must be 112M Hz or less.
    /// - `BUS_CLK` clock frequency must be 56 Mhz or less.
    /// - `FLASH_CLK` clock frequency must be 28 MHz or less.
    /// - The core clock to flash clock ratio is limited to a max value of 8.
    HighSpeed(HighSpeedMode),

    /// Very low power mode
    VeryLowPower(VeryLowPowerMode),
}

/// Clock selection modes available in `Mode::Run(_)`
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RunMode {
    /// System Oscillator Clock
    SOSC,
    
    /// Slow Internal Reference Clock
    SIRC,

    /// Fast internal Reference Clock
    FIRC,
    
    /// Sys PLL
    SPLL,
}

/// Clock selection modes available in `Mode::HighSpeed(_)`
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HighSpeedMode {
    /// Fast internal Reference Clock
    FIRC,
    
    /// Sys PLL
    SPLL,
}

/// Clock selection modes available in `Mode::VeryLowPower(_)`
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VeryLowPowerMode {
    /// Slow Internal Reference Clock
    SIRC,
}

/// Clock divider options for system oscillator.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SystemOscillatorOutput {
    /// Output disabled
    Disable = 0,

    /// Divide by 1
    Div1 = 1,
    
    /// Divide by 2
    Div2 = 2,
    
    /// Divide by 4
    Div4 = 3,
    
    /// Divide by 8
    Div8 = 4,
    
    /// Divide by 16
    Div16 = 5,
    
    /// Divide by 32
    Div32 = 6,
    
    /// Divide by 64
    Div64 = 7,
}

impl From<SystemOscillatorOutput> for u8 {
    fn from(div: SystemOscillatorOutput) -> u8 {
        div as u8
    }
}

impl From<SystemOscillatorOutput> for usize {
    fn from(div: SystemOscillatorOutput) -> usize {
        div as usize
    }
}

impl From<SystemOscillatorOutput> for isize {
    fn from(div: SystemOscillatorOutput) -> isize {
        div as isize
    }
}

impl Default for SystemOscillatorOutput {
    fn default() -> Self {
        SystemOscillatorOutput::Disable
    }
}

/// The System Clock Generator instance
pub struct Scg<'a> {
    register_block: &'a s32k144::scg::RegisterBlock,
    config: Config,
}

impl<'a> Scg<'a> {
    /// Initialized the System Clock Generator with the given configs
    pub fn init(scg: &'a s32k144::scg::RegisterBlock, config: Config) -> Self {
        match config.system_oscillator {
            SystemOscillatorInput::None => {
                scg.sosccsr.modify(|_, w| w.soscen()._0());
            },
            SystemOscillatorInput::Crystal(f) => {
                scg.sosccsr.modify(|_, w| w.soscen()._1());
                scg.sosccfg.modify(|_, w| w
                                   .erefs()._1()
                                   .hgo()._1()
                ); 
 
                if f >= 8_000_000 {
                    scg.sosccfg.modify(|_, w| w.range()._11());
                } else {
                    scg.sosccfg.modify(|_, w| w.range()._10());
                }

            },
            SystemOscillatorInput::Reference(_) => {
                scg.sosccsr.modify(|_, w| w.soscen()._1());
                scg.sosccfg.modify(|_, w| w.erefs()._1());
            },
        }
        
        
        scg.soscdiv.modify(|_, w| w.soscdiv1().bits(config.soscdiv1.into()));
        scg.soscdiv.modify(|_, w| w.soscdiv2().bits(config.soscdiv2.into()));
        
        
        Scg {
            register_block: scg,
            config: config,
        }
    }

    /// Return the frequency of socdiv1 clock if running
    pub fn soscdiv1_freq(&self) -> Option<u32> {
        let freq = self.config.system_oscillator.clock_frequency()?;
        match self.config.soscdiv1 {
            SystemOscillatorOutput::Disable => None,
            oscillator_output => {
                let div = 1 << (usize::from(oscillator_output) - 1);
                Some(freq / div)
            },
        }
    }
    
    /// Return the frequency of socdiv2 clock if running
    pub fn soscdiv2_freq(&self) -> Option<u32> {
        let freq = self.config.system_oscillator.clock_frequency()?;
        match self.config.soscdiv2 {
            SystemOscillatorOutput::Disable => None,
            oscillator_output => {
                let div = 1 << (usize::from(oscillator_output) - 1);
                Some(freq / div)
            },
        }       
    }
}

