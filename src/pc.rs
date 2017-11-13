//! The Power and Clocking (PC) SW module
//!
//! This consists of the following HW modules
//!
//! - SCG (System Clock Generator)
//! - SMC (System Mode Controller)
//! - PMC (Power Management Controller)

use s32k144;

/// Configurations for the System Clock Generator
#[derive(Default, Debug, PartialEq, Clone)]
pub struct Config {
    /// Set the power mode and system clock source
    pub mode: Mode,

    /// Clock divider for `CORE_CLK` and `SYS_CLK`.    
    pub div_core: DivCore,
    
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

impl Default for Mode {
    fn default() -> Self {
        Mode::Run(RunMode::FIRC)
    }
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

/// Clock divider for `CORE_CLK` and `SYS_CLK`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DivCore {
    /// Divide by 1
    Div1 = 0b0000,
    /// Divide by 2
    Div2 = 0b0001,
    /// Divide by 3
    Div3 = 0b0010,
    /// Divide by 4
    Div4 = 0b0011,
    /// Divide by 5
    Div5 = 0b0100,
    /// Divide by 6
    Div6 = 0b0101,
    /// Divide by 7
    Div7 = 0b0110,
    /// Divide by 8
    Div8 = 0b0111,
    /// Divide by 9
    Div9 = 0b1000,
    /// Divide by 10
    Div10 = 0b1001,
    /// Divide by 11
    Div11 = 0b1010,
    /// Divide by 12
    Div12 = 0b1011,
    /// Divide by 13
    Div13 = 0b1100,
    /// Divide by 14
    Div14 = 0b1101,
    /// Divide by 15
    Div15 = 0b1110,
    /// Divide by 16
    Div16 = 0b1111,
}

impl Default for DivCore {
    fn default() -> Self {
        DivCore::Div1
    }
}

impl From<DivCore> for u8 {
    fn from(d: DivCore) -> u8 {
        d as u8
    }
}

impl From<DivCore> for u32 {
    fn from(d: DivCore) -> u32 {
        d as u32
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
pub struct Pc<'a> {
    scg: &'a s32k144::scg::RegisterBlock,
    smc: &'a s32k144::smc::RegisterBlock,
    pmc: &'a s32k144::pmc::RegisterBlock,
    config: Config,
}

/// The valid error types for Pc::init()
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    NoSystemOscillator,
}

impl<'a> Pc<'a> {
    /// Initialized the System Clock Generator with the given configs
    pub fn init(
        scg: &'a s32k144::scg::RegisterBlock,
        smc: &'a s32k144::smc::RegisterBlock,
        pmc: &'a s32k144::pmc::RegisterBlock,
        config: Config
    ) -> Result<Self, Error> {
      
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

        // TODO: wait untill system oscillator is valid if configured
        
        scg.soscdiv.modify(|_, w| w.soscdiv1().bits(config.soscdiv1.into()));
        scg.soscdiv.modify(|_, w| w.soscdiv2().bits(config.soscdiv2.into()));

        // Allowing a transition into HSRUN or VLPR
        smc.pmprot.write(|w| w
                         .ahsrun()._1()
                         .avlp()._1()
        );

        // When configuring this, we should already have configured the source and make sure it's valid.      
        match config.mode {
            Mode::Run(mode) => {
                // Set the dividers
                scg.rccr.modify(|_, w| w.divcore().bits(u8::from(config.div_core)));
                match mode {
                    RunMode::SOSC => {
                        if let SystemOscillatorInput::None = config.system_oscillator {
                            return Err(Error::NoSystemOscillator)
                        }
                        scg.rccr.modify(|_, w| w.scs()._0001());
                    },
                    RunMode::SIRC => {
                        unimplemented!("Mode::Run(RunMode::SIRC) is is not supported yet");
                        scg.rccr.modify(|_, w| w.scs()._0010());
                    },
                    RunMode::FIRC => {
                        scg.rccr.modify(|_, w| w.scs()._0011())
                    },
                    RunMode::SPLL => {
                        unimplemented!("Mode::Run(RunMode::SPLL) is is not supported yet");
                        scg.rccr.modify(|_, w| w.scs()._0110())
                    },
                }
                // transition into run mode
                smc.pmctrl.modify(|_, w| w.runm()._00());
                while smc.pmstat.read().pmstat().bits() != 0000_001 {}
            },
            Mode::HighSpeed(mode) => {
                // Set the dividers
                scg.hccr.modify(|_, w| w.divcore().bits(u8::from(config.div_core)));
                unimplemented!("High speed more is not supported yet");
            },
            Mode::VeryLowPower(_mode) => {
                // Set the dividers
                scg.vccr.modify(|_, w| w.divcore().bits(u8::from(config.div_core)));
                unimplemented!("Very low power mode is not supported yet");
            },
        }
        
        
        Ok(Pc {
            scg: scg,
            smc: smc,
            pmc: pmc,
            config: config,
        })
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
    
    /// Return the frequency of `CORE_CLK` in MHz
    pub fn core_freq(&self) -> u32 {
        match self.config.mode {
            Mode::Run(mode) => {
                match mode {
                    RunMode::SOSC => {
                        let freq = self.config.system_oscillator.clock_frequency().unwrap();
                        freq / u32::from(self.config.div_core)
                    },
                    RunMode::SIRC => {
                        unimplemented!("Mode::Run(RunMode::SIRC) is is not supported yet");
                    },
                    RunMode::FIRC => {
                        48_000_000 / u32::from(self.config.div_core)
                    },
                    RunMode::SPLL => {
                        unimplemented!("Mode::Run(RunMode::SPLL) is is not supported yet");
                    },
                }
            },
            Mode::HighSpeed(mode) => {
                unimplemented!("High speed more is not supported yet");
            },
            Mode::VeryLowPower(_mode) => {
                unimplemented!("Very low power mode is not supported yet");
            },
        }
    }
}

