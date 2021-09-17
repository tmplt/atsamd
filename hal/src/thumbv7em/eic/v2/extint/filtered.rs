use crate::gpio::v2::InterruptConfig;

use crate::eic::v2::*;

use super::ExtInt;
use crate::set_sense_anyextint;

pub struct FilteredExtInt<I, C, AK, S>
where
    I: GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    S: SenseMode,
{
    pub extint: ExtInt<I, C, AK, S>,
}

impl<I, C, AK, S> FilteredExtInt<I, C, AK, S>
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
    pub fn disable_filtering<N>(
        self,
        eic: &mut Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
    ) -> ExtInt<I, C, AK, S>
    where
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.disable_filtering::<I::EINum>();
        // Return the inner ExtInt<...>
        self.extint
    }

    /// TODO
    pub fn set_sense_mode<AK2, S2, N>(
        self,
        // Used to enforce having access to EIController
        _eic: &mut Enabled<EIController<AK2, Configurable>, N>,
        sense: Sense,
    ) -> FilteredExtInt<I, C, AK, S2>
    where
        AK2: AnyClock,
        S2: SenseMode,
        N: Counter,
    {
        self.extint.regs.set_sense_mode(sense);

        FilteredExtInt {
            extint: ExtInt {
                regs: self.extint.regs,
                pin: self.extint.pin,
                clockmode: PhantomData,
                sensemode: PhantomData,
            },
        }
    }
    set_sense_anyextint! {self, "FilteredExt", None}
    set_sense_anyextint! {self, "FilteredExt", High}
    set_sense_anyextint! {self, "FilteredExt", Low}
    set_sense_anyextint! {self, "FilteredExt", Both}
    set_sense_anyextint! {self, "FilteredExt", Rise}
    set_sense_anyextint! {self, "FilteredExt", Fall}
}
