use crate::gpio::v2::InterruptConfig;

use super::*;
impl<I, C, AM, CS, AK, AS> ExtInt<I, C, AM, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    AM: AnyMode<Mode = Normal>,
    AK: AnyClock<Mode = WithClock<CS>>,
    AS: AnySenseMode,
{
    /// Enable filtering mode
    ///
    /// Filter acts as a majority vote
    ///
    /// Samples the ExtInt 3 times and outputs
    /// the value which occurs 2 or more times.
    ///
    /// Example:
    ///
    /// * Samples `[0, 0, 1]`   Out: `0`
    /// * Samples `[0, 1, 1]`   Out: `1`
    /// * Samples `[1, 1, 1]`   Out: `1`
    /// * Samples `[1, 0, 0]`   Out: `0`
    pub fn enable_filtering<N>(
        self,
        eic: &Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> ExtInt<I, C, Filtered, AK, AS>
    where
        N: Counter,
    {
        eic.enable_filtering::<I::EINum>();

        ExtInt {
            token: self.token,
            pin: self.pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }

    /// Enable asynchronous filtering mode
    ///
    /// Filter acts as a majority vote
    ///
    /// Samples the ExtInt 3 times and outputs
    /// the value which occurs 2 or more times.
    ///
    /// Example:
    ///
    /// * Samples `[0, 0, 1]`   Out: `0`
    /// * Samples `[0, 1, 1]`   Out: `1`
    /// * Samples `[1, 1, 1]`   Out: `1`
    /// * Samples `[1, 0, 0]`   Out: `0`
    pub fn enable_filtering_async<N>(
        self,
        eic: &Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> ExtInt<I, C, FilteredAsync, AK, AS>
    where
        N: Counter,
    {
        eic.enable_filtering::<I::EINum>();
        eic.enable_async::<I::EINum>();

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
    AM: AnyMode<Mode = Filtered>,
    AK: AnyClock<Mode = WithClock<CS>>,
    AS: AnySenseMode,
{
    /// Disable filtering
    pub fn disable_filtering<N>(
        self,
        eic: &Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
    ) -> ExtInt<I, C, Normal, AK, AS>
    where
        N: Counter,
    {
        eic.disable_filtering::<I::EINum>();

        ExtInt {
            token: self.token,
            pin: self.pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}

impl<I, C, CS, AK, AS> ExtInt<I, C, FilteredAsync, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    AK: AnyClock<Mode = WithClock<CS>>,
    AS: AnySenseMode,
{
    /// Disable async filtering
    pub fn disable_filtering<N>(
        self,
        eic: &Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
    ) -> ExtInt<I, C, Normal, AK, AS>
    where
        N: Counter,
    {
        eic.disable_filtering::<I::EINum>();
        eic.disable_async::<I::EINum>();

        ExtInt {
            token: self.token,
            pin: self.pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}
