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
    /// TODO
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
    ///
    pub fn enable_filtering<AM2, N>(
        self,
        eic: &mut Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> ExtInt<I, C, AM2, AK, AS>
    where
        N: Counter,
        AM2: AnyMode<Mode = Filtered>,
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
    /// TODO
    pub fn disable_filtering<AM2, N>(
        self,
        eic: &mut Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
    ) -> ExtInt<I, C, AM2, AK, AS>
    where
        N: Counter,
        AM2: AnyMode<Mode = Normal>,
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
