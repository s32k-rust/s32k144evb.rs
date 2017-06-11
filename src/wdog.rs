extern crate cortex_m;

use s32k144::{
    WDOG,
    Wdog,
};

pub enum WatchdogWindow {
    Disabled,
    Enabled(u16),
}

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
             

pub enum WatchdogError {
    ReconfigurationDisallowed,
    UnlockFailed,
    ConfigurationFailed,
}


/// pub fn configure(settings: WatchdogSettings) -> Result<(), WatchdogError> 
///
/// configures the watchdog timer and return () or an error.
///
/// can be used as an initial configuration within 128 cycles of startup
/// or to reconfigure if reconfiguring is allowed.
///
/// Since this functions needs to wait for the settings to either
/// change or fail changing, the function can spend significant time
/// in an interrupt free context
pub fn configure(settings: WatchdogSettings) -> Result<(), WatchdogError> {
    
    cortex_m::interrupt::free(|cs| {
        // TODO: find good values for these constants
        const UNLOCK_TRIES: u32 = 3;
        const UNLOCK_CHECKS: u32 = 5000; 
        
        let wdog = WDOG.borrow(cs);

        let mut unlocked_flag = false;
        let mut unlocked = |wdog: &Wdog| {
            if unlocked_flag {
                true
            } else {
                unlocked_flag = wdog.cs.read().ulk().is_1();
                unlocked_flag
            }
        };
                
        let unlock = |wdog: &Wdog| wdog.cnt.write(|w| unsafe{ w.bits(0xd928c520) });
        let under_configuration = |wdog: &Wdog| wdog.cs.read().rcs().is_0();
        
        if !unlocked(wdog) && under_configuration(wdog) {
            return Err(WatchdogError::ReconfigurationDisallowed);
        }
        
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

        if !unlocked(wdog) {
            return Err(WatchdogError::UnlockFailed);
        }
        
        apply_settings(settings, wdog);
        
        // TODO: write some logic (acceptance test) that detects if the reconfiguration fails
        while under_configuration(wdog) {}
        Ok(())
    })    
        
}
    
fn apply_settings(settings: WatchdogSettings, wdog: &Wdog) {
    let (win_value, win_enabled) = match settings.window {
        WatchdogWindow::Enabled(x) => (x, true),
        WatchdogWindow::Disabled => (0x0000, false),
    };
    
    unsafe{ wdog.toval.write(|w| w.bits(settings.timeout_value as u32)); }
    unsafe{ wdog.win.write(|w|  w.bits(win_value as u32)); }
    
    wdog.cs.modify(|_, w| w
                   .stop().bits(settings.stop_enable as u8)
                   .wait().bits(settings.wait_enable as u8)
                   .dbg().bits(settings.debug_enable as u8)
                   .update().bits(settings.allow_updates as u8)
                   .int().bits(settings.interrupt_enable as u8)
                   .en().bits(settings.enable as u8)
                   .pres().bits(settings.prescaler as u8)
                   .cmd32en()._1()
                   .win().bits(win_enabled as u8)
    );
}

