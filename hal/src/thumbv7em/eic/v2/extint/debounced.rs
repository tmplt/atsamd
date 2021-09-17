use crate::gpio::v2::InterruptConfig;

use super::*;

impl<I, C, AM, CS, AK, S> ExtInt<I, C, AM, AK, S>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    AM: AnyMode<Mode = Normal>,
    AK: AnyClock<Mode = WithClock<CS>>,
    S: DebounceMode + AnySenseMode,
{
    // Methods related to debouncing go here since they require a clock
    // and that SenseMode are one of: Rise, Fall or Both

    /// TODO
    ///
    /// ExtInt sense mode must be either [`Sense::Rise`], [`Sense::Fall`]
    /// or [`Sense::Both`]
    pub fn enable_debouncing<AM2, N>(
        self,
        eic: &mut Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> ExtInt<I, C, AM, AK, S>
    where
        N: Counter,
        AM2: AnyMode<Mode = Debounced>,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.enable_debouncing::<I::EINum>();
        ExtInt {
            token: self.token,
            pin: self.pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}

impl<I, C, AM, CS, AK, S> ExtInt<I, C, AM, AK, S>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    AM: AnyMode<Mode = Debounced>,
    AK: AnyClock<Mode = WithClock<CS>>,
    S: DebounceMode + AnySenseMode,
{
    // Do not need access to the EIController here

    /// TODO
    pub fn disable_debouncing<AM2, N>(
        self,
        eic: &mut Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
    ) -> ExtInt<I, C, AM2, AK, S>
    where
        N: Counter,
        AM2: AnyMode<Mode = Normal>,
    {
        eic.disable_debouncing::<I::EINum>();
        ExtInt {
            token: self.token,
            pin: self.pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }

    /// TODO
    pub fn set_debouncer_settings<N>(
        &self,
        eic: &mut Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
        settings: &DebouncerSettings,
    ) where
        N: Counter,
    {
        eic.set_debouncer_settings::<I::EINum>(settings);
    }
}

