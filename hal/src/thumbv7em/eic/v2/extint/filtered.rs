use crate::gpio::v2::InterruptConfig;

use crate::eic::v2::*;

use super::ExtInt;

pub struct FilteredExtInt<I, C, K, S>
where
    I: GetEINum,
    C: InterruptConfig,
    K: Clock,
    S: SenseMode,
{
    pub extint: ExtInt<I, C, K, S>,
}

impl<I, C, K, S> FilteredExtInt<I, C, K, S>
where
    I: GetEINum,
    C: InterruptConfig,
    K: Clock,
    S: SenseMode,
{
    // Do not need access to the EIController here
    /// Read the pin state of the ExtInt
    /// TODO
    pub fn pin_state(&self) -> bool {
        self.extint.pin_state()
    }
}
