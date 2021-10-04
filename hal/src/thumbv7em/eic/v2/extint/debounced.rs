use super::*;
use crate::gpio::v2::InterruptConfig;

impl<I, C, AM, CS, AK> ExtInt<I, C, AM, AK, SenseBoth>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrcMarker,
    AM: AnyMode<Mode = Normal>,
    AK: AnyClock<Mode = dyn WithClock<CS>>,
{
    /// Enable debouncing
    ///
    /// ExtInt sense mode must be either [`Sense::Rise`], [`Sense::Fall`]
    /// or [`Sense::Both`]
    pub fn enable_debouncing<N>(
        self,
        eic: &Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> ExtInt<I, C, Debounced, AK, SenseBoth>
    where
        N: Counter,
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

    /// Enable async debouncing
    ///
    /// ExtInt sense mode must be either [`Sense::Rise`], [`Sense::Fall`]
    /// or [`Sense::Both`]
    pub fn enable_debouncing_async<N>(
        self,
        eic: &Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> ExtInt<I, C, DebouncedAsync, AK, SenseBoth>
    where
        N: Counter,
    {
        eic.enable_debouncing::<I::EINum>();
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
impl<I, C, AM, CS, AK> ExtInt<I, C, AM, AK, SenseRise>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrcMarker,
    AM: AnyMode<Mode = Normal>,
    AK: AnyClock<Mode = WithClock<CS>>,
{
    /// Enable debouncing
    ///
    /// ExtInt sense mode must be either [`Sense::Rise`], [`Sense::Fall`]
    /// or [`Sense::Both`]
    pub fn enable_debouncing<N>(
        self,
        eic: &Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> ExtInt<I, C, Debounced, AK, SenseRise>
    where
        N: Counter,
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
    /// Enable async debouncing
    ///
    /// ExtInt sense mode must be either [`Sense::Rise`], [`Sense::Fall`]
    /// or [`Sense::Both`]
    pub fn enable_debouncing_async<N>(
        self,
        eic: &Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> ExtInt<I, C, DebouncedAsync, AK, SenseRise>
    where
        N: Counter,
    {
        eic.enable_debouncing::<I::EINum>();
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
impl<I, C, AM, CS, AK> ExtInt<I, C, AM, AK, SenseFall>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrcMarker,
    AM: AnyMode<Mode = Normal>,
    AK: AnyClock<Mode = WithClock<CS>>,
{
    /// Enable debouncing
    ///
    /// ExtInt sense mode must be either [`Sense::Rise`], [`Sense::Fall`]
    /// or [`Sense::Both`]
    pub fn enable_debouncing<N>(
        self,
        eic: &Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> ExtInt<I, C, Debounced, AK, SenseFall>
    where
        N: Counter,
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
    /// Enable async debouncing
    ///
    /// ExtInt sense mode must be either [`Sense::Rise`], [`Sense::Fall`]
    /// or [`Sense::Both`]
    pub fn enable_debouncing_async<N>(
        self,
        eic: &Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> ExtInt<I, C, DebouncedAsync, AK, SenseFall>
    where
        N: Counter,
    {
        eic.enable_debouncing::<I::EINum>();
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
    CS: EIClkSrcMarker,
    AM: AnyMode<Mode = Debounced>,
    AK: AnyClock<Mode = WithClock<CS>>,
    AS: AnySenseMode,
{
    /// Disable debouncing
    pub fn disable_debouncing<N>(
        self,
        eic: &Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
    ) -> ExtInt<I, C, Normal, AK, AS>
    where
        N: Counter,
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
}

impl<I, C, CS, AK, AS> ExtInt<I, C, DebouncedAsync, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrcMarker,
    AK: AnyClock<Mode = WithClock<CS>>,
    AS: AnySenseMode,
{
    /// Disable debouncing
    pub fn disable_debouncing<N>(
        self,
        eic: &Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
    ) -> ExtInt<I, C, Normal, AK, AS>
    where
        N: Counter,
    {
        eic.disable_debouncing::<I::EINum>();
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

impl<I, C, AK, AS> ExtInt<I, C, Debounced, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    AS: AnySenseMode,
{
    /// Read the debounced pin state of the ExtInt
    pub fn pin_state(&self) -> bool {
        self.token.regs.pin_state()
    }
}

impl<I, C, AK, AS> ExtInt<I, C, DebouncedAsync, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    AS: AnySenseMode,
{
    /// Read the debounced pin state of the ExtInt
    pub fn pin_state(&self) -> bool {
        self.token.regs.pin_state()
    }
}
