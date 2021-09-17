use crate::eic::v2::*;
use crate::gpio::v2::InterruptConfig;

use super::ExtInt;
use crate::set_sense_anyextint;

pub struct DebouncedExtInt<I, C, AK, S>
where
    I: GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    S: SenseMode,
{
    pub extint: ExtInt<I, C, AK, S>,
}

impl<I, C, AK, S> DebouncedExtInt<I, C, AK, S>
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
    pub fn disable_debouncing<N>(
        self,
        eic: &mut Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
    ) -> ExtInt<I, C, AK, S>
    where
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.disable_debouncing::<I::EINum>();
        // Return the inner ExtInt<...>
        self.extint
    }

    /// TODO
    pub fn set_debouncer_settings<N>(
        &self,
        eic: &mut Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
        settings: &DebouncerSettings,
    ) where
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.set_debouncer_settings::<I::EINum>(settings);
    }

    /// TODO
    pub fn set_sense_mode<AK2, S2, N>(
        self,
        // Used to enforce having access to EIController
        _eic: &mut Enabled<EIController<AK2, Configurable>, N>,
        sense: Sense,
    ) -> DebouncedExtInt<I, C, AK, S2>
    where
        AK2: AnyClock,
        S2: SenseMode,
        N: Counter,
    {
        self.extint.regs.set_sense_mode(sense);

        DebouncedExtInt {
            extint: ExtInt {
                regs: self.extint.regs,
                pin: self.extint.pin,
                clockmode: PhantomData,
                sensemode: PhantomData,
            },
        }
    }
    set_sense_anyextint! {self, "DebouncedExt", None}
    set_sense_anyextint! {self, "DebouncedExt", High}
    set_sense_anyextint! {self, "DebouncedExt", Low}
    set_sense_anyextint! {self, "DebouncedExt", Both}
    set_sense_anyextint! {self, "DebouncedExt", Rise}
    set_sense_anyextint! {self, "DebouncedExt", Fall}
}
