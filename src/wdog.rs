extern crate cortex_m;

use s32k144;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WatchdogWindow {
    Disabled,
    Enabled(u16),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WatchdogSettings {

    /// The watchdog counter is continously compared with the timeout value
    /// If the counter reaches the timeout value, the watchfog forces a
    /// reset triggering event.
    pub timeout_value: u16,
    
    /// When the window mode is active the window describes the earliest time that
    /// a refresh is considered active. Refreshing earlier than the window will
    /// result in the watchdog resetting the MCU
    pub window: WatchdogWindow,

    
    /// This is a fixed 256 pre-sxaling of watchfog counter reference clock.
    /// See the block diagram in the data sheet for more information
    pub prescaler: bool,

    pub enable: bool,

    /// When interrupts are enabled a reset-triggering event will first cause
    /// the watchdog to generate an interrupt request.
    /// Next, the watchdog delays 128 bus clock cycles before forcing a reset.
    /// This is to allow the ISR to perform tasks (analyzing stack etc)
    pub interrupt_enable: bool,

    /// This bit allows software to reconfigure the watchdog without a reset
    ///
    /// false: After the initial configuration, the watchdog cannot be later
    /// modified without forcing a reset
    ///
    /// true: Software can modify the watchdog configuration registers within
    /// 128 bus clocks after performing the unlock write sequence
    pub allow_updates: bool,

    /// enables the watchdog when the chip is in debug mode
    pub debug_enable: bool,
    
    /// enables the watchdog when the chip is in wait mode
    pub wait_enable: bool,
    
    /// enables the watchdog when the chip is in stop mode
    pub stop_enable: bool,
}

impl Default for WatchdogSettings {
    fn default() -> Self {
        Self{timeout_value: 0b0000010000000000,
             window: WatchdogWindow::Disabled,
             prescaler: false,
             enable: true,
             interrupt_enable: false,
             allow_updates: false,
             debug_enable: false,
             wait_enable: false,
             stop_enable: false,
        }
    }
}
             

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WatchdogError {
    ReconfigurationDisallowed,
    UnlockFailed,
    ConfigurationFailed,
}

pub struct Watchdog<'a> {
    register_block: &'a s32k144::wdog::RegisterBlock,
}

impl<'a> Watchdog<'a> {
    /// Initializes the watchdog
    ///
    /// This function needs to be called within 128 cycles of startup or it will fail.
    pub fn init(
        wdog: &'a s32k144::wdog::RegisterBlock,
        settings: WatchdogSettings,
    ) -> Result<Self, WatchdogError> {
        let watchdog = Watchdog{
            register_block: wdog,
        };
        watchdog.configure(settings)?;
        Ok(watchdog)
    }

    pub fn reset(&self) {
        cortex_m::interrupt::free(|_cs| self.register_block.cnt.write(|w| unsafe{ w.bits(0xB480_A602)}));
    }
    
    /// pub fn configure(settings: WatchdogSettings) -> Result<(), WatchdogError> 
    ///
    /// reconfigures the watchdog timer and return Ok(()) or an error.
    pub fn configure(&self, settings: WatchdogSettings) -> Result<(), WatchdogError> {
        
        // TODO: find good values for these constants
        const UNLOCK_TRIES: u32 = 3;
        const UNLOCK_CHECKS: u32 = 5000; 
            
        let wdog = self.register_block;

        let mut unlocked_flag = false;
        let mut unlocked = |wdog: &s32k144::wdog::RegisterBlock| {
            if unlocked_flag {
                true
            } else {
                unlocked_flag = wdog.cs.read().ulk().is_1();
                unlocked_flag
            }
        };
            
        let unlock = |wdog: &s32k144::wdog::RegisterBlock| wdog.cnt.write(|w| unsafe{ w.bits(0xd928c520) });
        let under_configuration = |wdog: &s32k144::wdog::RegisterBlock| wdog.cs.read().rcs().is_0();
        
        if !unlocked(wdog) && under_configuration(wdog) {
            return Err(WatchdogError::ReconfigurationDisallowed);
        }

        if !under_configuration(wdog) && !unlocked(wdog){
            for _tries in 0..UNLOCK_TRIES {
                unlock(wdog);
                
                let mut i = UNLOCK_CHECKS;
                while i > 0 {
                    if unlocked(wdog) {
                        i -= 1;
                    } else {
                        break;
                    }
                }
                
                if unlocked(wdog) {
                    break;
                }
            }
        }
        
        if !unlocked(wdog) {
            return Err(WatchdogError::UnlockFailed);
        }
        
        self.apply_settings(settings);
        
        // TODO: write some logic (acceptance test) that detects if the reconfiguration fails
        while under_configuration(wdog) {}
        Ok(())
    }

    fn apply_settings(&self, settings: WatchdogSettings) {
        let (win_value, win_enabled) = match settings.window {
            WatchdogWindow::Enabled(x) => (x, true),
            WatchdogWindow::Disabled => (0x0000, false),
        };
        
        unsafe{ self.register_block.toval.write(|w| w.bits(settings.timeout_value as u32)); }
        unsafe{ self.register_block.win.write(|w|  w.bits(win_value as u32)); }
        
        self.register_block.cs.modify(|_, w| w
                                      .stop().bit(settings.stop_enable)
                                      .wait().bit(settings.wait_enable)
                                      .dbg().bit(settings.debug_enable)
                                      .update().bit(settings.allow_updates)
                                      .int().bit(settings.interrupt_enable)
                                      .en().bit(settings.enable)
                                      .pres().bit(settings.prescaler)
                                      .cmd32en()._1()
                                      .win().bit(win_enabled)
        );
    }
    

}


    
