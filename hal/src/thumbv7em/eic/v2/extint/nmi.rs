use super::*;
use crate::gpio::v2::InterruptConfig;
// Macro for setting sense
use crate::set_sense_ext_nmi;

//==============================================================================
// NmiExtInt
//==============================================================================

/// TODO
pub struct NmiExtInt<I, C, AM, AK, AS>
where
    I: NmiEI,
    C: InterruptConfig,
    AM: AnyMode,
    AK: AnyClock,
    AS: AnySenseMode,
{
    pub(in crate::thumbv7em::eic::v2) token: NmiToken,
    pub(in crate::thumbv7em::eic::v2) pin: Pin<I, Interrupt<C>>,
    mode: PhantomData<AM>,
    clockmode: PhantomData<AK>,
    sensemode: PhantomData<AS>,
}

// Sealed for NmiExtInt
impl<I, C, AM, AK, AS> Sealed for NmiExtInt<I, C, AM, AK, AS>
where
    I: NmiEI,
    C: InterruptConfig,
    AM: AnyMode,
    AK: AnyClock,
    AS: AnySenseMode,
{
}

impl<I, C, AM, CS> NmiExtInt<I, C, AM, WithClock<CS>, SenseNone>
where
    I: NmiEI,
    C: InterruptConfig,
    AM: AnyMode<Mode = Normal>,
    CS: EIClkSrc,
{
    /// Create initial synchronous NmiExtInt
    /// TODO
    pub(crate) fn new_sync(token: NmiToken, pin: Pin<I, Interrupt<C>>) -> Self {
        NmiExtInt {
            token,
            pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}

impl<I, AM, C> NmiExtInt<I, C, AM, WithoutClock, SenseNone>
where
    I: NmiEI,
    C: InterruptConfig,
    AM: AnyMode<Mode = AsyncOnly>,
{
    /// Create initial asynchronous NmiExtInt
    /// TODO
    pub(crate) fn new_async(
        token: NmiToken,
        pin: Pin<I, Interrupt<C>>,
    ) -> NmiExtInt<I, C, AM, WithoutClock, SenseNone> {
        // #TODO

        // Need to set async flag
        token.regs.set_async_mode(true);
        NmiExtInt {
            token,
            pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}

impl<I, C, AM, AK> NmiExtInt<I, C, AM, AK, SenseLow>
where
    I: NmiEI,
    C: InterruptConfig,
    AM: AnyMode<Mode = Normal>,
    AK: AnyClock,
{
    /// TODO
    /// When EIController has access to a clock source both
    /// async and synchronous modes are available
    ///
    /// Repurpose the `OnlyAsync` for this mode
    pub fn enable_async<AM2, AS, N>(
        self,
        // Used to enforce WithClock
        _eic: &Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
    ) -> NmiExtInt<I, C, AM2, AK, AS>
    where
        N: Counter,
        AS: AnySenseMode<Mode = SenseLow>,
        AM2: AnyMode<Mode = AsyncOnly>,
    {
        self.token.regs.set_async_mode(true);

        NmiExtInt {
            token: self.token,
            pin: self.pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}
impl<I, C, AM, AK> NmiExtInt<I, C, AM, AK, SenseHigh>
where
    I: NmiEI,
    C: InterruptConfig,
    AM: AnyMode<Mode = Normal>,
    AK: AnyClock,
{
    /// TODO
    /// When EIController has access to a clock source both
    /// async and synchronous modes are available
    ///
    /// Repurpose the `OnlyAsync` for this mode
    pub fn enable_async<AM2, AS, N>(
        self,
        // Used to enforce WithClock
        _eic: &Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
    ) -> NmiExtInt<I, C, AM2, AK, AS>
    where
        N: Counter,
        AS: AnySenseMode<Mode = SenseHigh>,
        AM2: AnyMode<Mode = AsyncOnly>,
    {
        self.token.regs.set_async_mode(true);

        NmiExtInt {
            token: self.token,
            pin: self.pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}
impl<I, C, AM, AK, AS> NmiExtInt<I, C, AM, AK, AS>
where
    I: NmiEI,
    C: InterruptConfig,
    AM: AnyMode<Mode = AsyncOnly>,
    AK: AnyClock,
    AS: AnySenseMode,
{
    /// TODO
    /// Only possible to deactivate AnyMode<Mode = AsyncOnly>
    /// when EIController has access to a clock source.
    /// FIXME
    pub fn disable_async<AM2, N>(
        self,
        // Used to enforce WithClock
        _eic: &Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
    ) -> NmiExtInt<I, C, AM2, AK, AS>
    where
        N: Counter,
        AM2: AnyMode<Mode = Normal>,
    {
        self.token.regs.set_async_mode(false);

        NmiExtInt {
            token: self.token,
            pin: self.pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}

impl<I, C, CS, AM, AK, AS> NmiExtInt<I, C, AM, AK, AS>
where
    I: NmiEI,
    C: InterruptConfig,
    CS: EIClkSrc,
    AM: AnyMode<Mode = Normal>,
    AK: AnyClock<Mode = WithClock<CS>>,
    AS: AnySenseMode,
{
    /// TODO
    pub fn enable_filtering<AM2, N>(
        self,
        _eic: &Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> NmiExtInt<I, C, AM, AK, AS>
    where
        N: Counter,
        AM2: AnyMode<Mode = Filtered>,
    {
        self.token.regs.set_filter_mode(true);

        NmiExtInt {
            token: self.token,
            pin: self.pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}
impl<I, C, CS, AM, AK, AS> NmiExtInt<I, C, AM, AK, AS>
where
    I: NmiEI,
    C: InterruptConfig,
    CS: EIClkSrc,
    AM: AnyMode<Mode = Filtered>,
    AK: AnyClock<Mode = WithClock<CS>>,
    AS: AnySenseMode,
{
    /// TODO
    pub fn disable_filtering<AM2, N>(
        self,
        _eic: &Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> NmiExtInt<I, C, AM, AK, AS>
    where
        N: Counter,
        AM2: AnyMode<Mode = Normal>,
    {
        self.token.regs.set_filter_mode(false);

        NmiExtInt {
            token: self.token,
            pin: self.pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}

// Methods for any state of NmiExtInt
impl<I, C, AM, AK, AS> NmiExtInt<I, C, AM, AK, AS>
where
    I: NmiEI,
    C: InterruptConfig,
    AM: AnyMode,
    AK: AnyClock,
    AS: AnySenseMode,
{
    /// Set sense mode for [`NmiExtInt`]
    ///
    /// Enabled directly when [`Sense`] changes from `Sense::None`
    ///
    /// Different from the non-`NMI` interrupts which require
    /// calling [`Enabled<EIController::finalize()>`] to enable/activate
    /// interrupts.
    ///
    /// TODO
    pub fn set_sense_mode<AK2, S2, N>(
        self,
        // Used to enforce having access to EIController
        eic: &Enabled<EIController<AK2, Configurable>, N>,
        sense: Sense,
    ) -> NmiExtInt<I, C, AM, AK, S2>
    where
        AK2: AnyClock,
        S2: AnySenseMode,
        N: Counter,
    {
        eic.set_sense_mode_nmi(sense);

        NmiExtInt {
            token: self.token,
            pin: self.pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
    set_sense_ext_nmi! {self, NmiExtInt, None}
    set_sense_ext_nmi! {self, NmiExtInt, High}
    //set_sense_ext_nmi! {self, NmiExtInt, Low}
    set_sense_ext_nmi! {self, NmiExtInt, Both}
    set_sense_ext_nmi! {self, NmiExtInt, Rise}
    set_sense_ext_nmi! {self, NmiExtInt, Fall}

    /// TODO
    pub fn clear_interrupt_status(&self) {
        self.token.regs.clear_interrupt_status();
    }
}

impl<I, C> NmiExtInt<I, C, AsyncOnly, WithoutClock, SenseLow>
where
    I: NmiEI,
    C: InterruptConfig,
{
    // The only SenseModes supported in OnlyAsync
    // are of the kind "LevelDetectMode"
    //set_sense_ext_nmi! {self, NmiExtInt, None Async}
    //set_sense_ext_nmi! {self, NmiExtInt, Hig Async}
    set_sense_ext_nmi! {Async self, NmiExtInt, Low}
}
