use crate::eic::v2::*;
use crate::gpio::v2::InterruptConfig;

//use super::{AnyExtInt, ExtInt};
use super::ExtInt;

//pub struct DebouncedExtInt<E>
pub struct DebouncedExtInt<I, C, CS, S>
where
    //E: AnyExtInt,
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    S: SenseMode,
{
    //pub extint: E,
    pub extint: ExtInt<I, C, WithClock<CS>, S>,
}

impl<I, C, CS, S> DebouncedExtInt<I, C, CS, S>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    S: SenseMode,
{
    // Do not need access to the EIController here
    /// Read the pin state of the ExtInt
    /// TODO
    pub fn pin_state(&self) -> bool {
        self.extint.pin_state()
    }

    /// TODO
    pub fn set_debouncer_settings<N>(
        &self,
        eic: &mut Enabled<EIController<WithClock<CS>>, N>,
        settings: &DebouncerSettings,
    ) where
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.set_debouncer_settings::<I::EINum>(settings);
    }

    /// TODO
    pub fn disable_debouncing<N>(
        self,
        eic: &mut Enabled<EIController<WithClock<CS>>, N>,
    ) -> ExtInt<I, C, WithClock<CS>, S>
    where
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.disable_debouncing::<I::EINum>();
        // Return the inner ExtInt<...>
        self.extint
    }
}

//impl<E> DebouncedExtInt<E> where E: AnyExtInt<SenseMode = SenseRise> {}

//impl<E> DebouncedExtInt<E> where E: AnyExtInt {}
