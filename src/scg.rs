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
}

