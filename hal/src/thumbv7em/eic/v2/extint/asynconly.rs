use crate::gpio::v2::InterruptConfig;

use crate::eic::v2::*;

use super::ExtInt;
use crate::set_sense_anyextint;

pub struct AsyncExtInt<I, C, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    AS: AnySenseMode,
{
    pub extint: ExtInt<I, C, AK, AS>,
}

// Sealed for AsyncExtInt
impl<I, C, AK, AS> Sealed for AsyncExtInt<I, C, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    AS: AnySenseMode,
{
}

impl<I, C, AK, AS> AsyncExtInt<I, C, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    AS: AnySenseMode,
{
    // Do not need access to the EIController here
    /// Read the pin state of the ExtInt
    /// TODO
    pub fn pin_state(&self) -> bool {
        self.extint.pin_state()
    }
    /// TODO
    pub fn set_sense_mode<AK2, AS2, N>(
        self,
        // Used to enforce having access to EIController
        _eic: &mut Enabled<EIController<AK2, Configurable>, N>,
        sense: Sense,
    ) -> AsyncExtInt<I, C, AK, AS2>
    where
        AK2: AnyClock,
        AS2: AnySenseMode,
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

