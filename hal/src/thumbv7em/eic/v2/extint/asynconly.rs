use crate::gpio::v2::InterruptConfig;

use crate::eic::v2::*;

use super::ExtInt;
use crate::set_sense_anyextint;

pub struct AsyncExtInt<I, C, AK, S>
where
    I: GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    S: SenseMode,
{
    pub extint: ExtInt<I, C, AK, S>,
}

// Sealed for AsyncExtInt
impl<I, C, AK, S> Sealed for AsyncExtInt<I, C, AK, S>
where
    I: GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    S: SenseMode,
{
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
    /// TODO
    pub fn set_sense_mode<AK2, S2, N>(
        self,
        // Used to enforce having access to EIController
        _eic: &mut Enabled<EIController<AK2, Configurable>, N>,
        sense: Sense,
    ) -> AsyncExtInt<I, C, AK, S2>
    where
        AK2: AnyClock,
        S2: SenseMode,
        N: Counter,
    {
        self.extint.regs.set_sense_mode(sense);

        AsyncExtInt {
            extint: ExtInt {
                regs: self.extint.regs,
                pin: self.extint.pin,
                clockmode: PhantomData,
                sensemode: PhantomData,
            },
        }
    }
    set_sense_anyextint! {self, "AsyncExt", None}
    set_sense_anyextint! {self, "AsyncExt", High}
    set_sense_anyextint! {self, "AsyncExt", Low}
    set_sense_anyextint! {self, "AsyncExt", Both}
    set_sense_anyextint! {self, "AsyncExt", Rise}
    set_sense_anyextint! {self, "AsyncExt", Fall}
}

