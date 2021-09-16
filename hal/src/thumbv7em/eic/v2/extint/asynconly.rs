use crate::gpio::v2::InterruptConfig;

use crate::eic::v2::*;

use super::ExtInt;
use crate::set_sense_anyextint;

pub struct AsyncExtInt<I, C, K, S>
where
    I: GetEINum,
    C: InterruptConfig,
    K: AnyClock,
    S: SenseMode,
{
    pub extint: ExtInt<I, C, K, S>,
}

impl<I, C, AK, S> AsyncExtInt<I, C, AK, S>
where
    I: GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    S: SenseMode,
{
    // Do not need access to the EIController here
    /// Read the pin state of the ExtInt
    /// TODO
    pub fn pin_state(&self) -> bool {
        self.extint.pin_state()
    }
    set_sense_anyextint! {self, "AsyncExt", None}
    set_sense_anyextint! {self, "AsyncExt", High}
    set_sense_anyextint! {self, "AsyncExt", Low}
    set_sense_anyextint! {self, "AsyncExt", Both}
    set_sense_anyextint! {self, "AsyncExt", Rise}
    set_sense_anyextint! {self, "AsyncExt", Fall}
}
