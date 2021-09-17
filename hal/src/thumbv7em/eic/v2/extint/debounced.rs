use super::*;
use crate::gpio::v2::InterruptConfig;

impl<I, C, AM, CS, AK> ExtInt<I, C, AM, AK, SenseBoth>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    AM: AnyMode<Mode = Normal>,
    AK: AnyClock<Mode = WithClock<CS>>,
{
    /// TODO
    ///
    /// ExtInt sense mode must be either [`Sense::Rise`], [`Sense::Fall`]
    /// or [`Sense::Both`]
    pub fn enable_debouncing<AM2, AS, N>(
        self,
        eic: &Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> ExtInt<I, C, AM, AK, AS>
    where
        N: Counter,
        AS: AnySenseMode<Mode = SenseBoth>,
        AM2: AnyMode<Mode = Debounced>,
    {
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
impl<I, C, AM, CS, AK> ExtInt<I, C, AM, AK, SenseRise>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    AM: AnyMode<Mode = Normal>,
    AK: AnyClock<Mode = WithClock<CS>>,
{
    /// TODO
    ///
    /// ExtInt sense mode must be either [`Sense::Rise`], [`Sense::Fall`]
    /// or [`Sense::Both`]
    pub fn enable_debouncing<AM2, AS, N>(
        self,
        eic: &Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> ExtInt<I, C, AM, AK, AS>
    where
        N: Counter,
        AS: AnySenseMode<Mode = SenseRise>,
        AM2: AnyMode<Mode = Debounced>,
    {
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
impl<I, C, AM, CS, AK> ExtInt<I, C, AM, AK, SenseFall>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    AM: AnyMode<Mode = Normal>,
    AK: AnyClock<Mode = WithClock<CS>>,
{
    /// TODO
    ///
    /// ExtInt sense mode must be either [`Sense::Rise`], [`Sense::Fall`]
    /// or [`Sense::Both`]
    pub fn enable_debouncing<AM2, AS, N>(
        self,
        eic: &Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> ExtInt<I, C, AM, AK, AS>
    where
        N: Counter,
        AS: AnySenseMode<Mode = SenseFall>,
        AM2: AnyMode<Mode = Debounced>,
    {
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

impl<I, C, AM, CS, AK, AS> ExtInt<I, C, AM, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    AM: AnyMode<Mode = Debounced>,
    AK: AnyClock<Mode = WithClock<CS>>,
    AS: AnySenseMode,
{
    /// TODO
    pub fn disable_debouncing<AM2, N>(
        self,
        eic: &Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
    ) -> ExtInt<I, C, AM2, AK, AS>
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
        eic: &Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
        settings: &DebouncerSettings,
    ) where
        N: Counter,
    {
        eic.set_debouncer_settings::<I::EINum>(settings);
    }
}
