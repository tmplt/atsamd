use super::*;
use crate::gpio::v2::InterruptConfig;
// Macro for setting sense
use crate::set_sense_ext_nmi;

//==============================================================================
// NmiExtInt
//==============================================================================

/// Non-Maskable Interrupt External Interrupt struct
///
/// ## Setting sense mode for [`NmiExtInt`]
///
/// Enabled directly when [`Sense`] changes from `Sense::None`
///
/// Different from the non-`NMI` interrupts which require
/// calling [`Enabled<EIController::finalize()>`] to enable/activate
/// interrupts.
///
/// ## Filtering
///
/// By enabling filtering a majority vote filter identical
/// to that of regular [`ExtInt::enable_filtering()`]
/// is available
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
    CS: EIClkSrcMarker,
{
    /// Create initial synchronous NmiExtInt
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
    ///
    /// All level detection ([`SenseHigh`], [`SenseLow`])
    /// is done asynchronously since it does not require
    /// any external clocking
    pub(crate) fn new_async(
        token: NmiToken,
        pin: Pin<I, Interrupt<C>>,
    ) -> NmiExtInt<I, C, AM, WithoutClock, SenseNone> {
        // Set async flag
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

impl<I, C, AM, AK, AS> NmiExtInt<I, C, AM, AK, AS>
where
    I: NmiEI,
    C: InterruptConfig,
    AM: AnyMode<Mode = Normal>,
    AK: AnyClock,
    AS: AnySenseMode,
{
    /// When EIController has access to a clock source both
    /// async and synchronous modes are available
    ///
    /// Thus it is possible to change to `OnlyAsync` mode,
    /// which has all `LevelDetectMode`
    ///
    /// Force SenseNone
    pub fn enable_async<N>(
        self,
        // Used to enforce WithClock
        _eic: &Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
    ) -> NmiExtInt<I, C, AsyncOnly, WithoutClock, SenseNone>
    where
        N: Counter,
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
    /// Enable filtering of NmiExtInt
    pub fn enable_filtering<AM2, CS, N>(
        self,
        _eic: &Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> NmiExtInt<I, C, AM, AK, AS>
    where
        N: Counter,
        CS: EIClkSrcMarker,
        AM2: AnyMode<Mode = Filtered>,
    {
        self.token.regs.set_filter_mode(true);
        // Async is never true when filtering
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
    CS: EIClkSrcMarker,
    AM: AnyMode<Mode = Filtered>,
    AK: AnyClock<Mode = WithClock<CS>>,
    AS: AnySenseMode,
{
    /// Disable filtering of NmiExtInt
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
    /// Clear the ExtIntNMI interrupt
    pub fn clear_interrupt_status(&self) {
        self.token.regs.clear_interrupt_status();
    }
}

impl<I, C> NmiExtInt<I, C, AsyncOnly, WithoutClock, SenseNone>
where
    I: NmiEI,
    C: InterruptConfig,
{
    // Allow switching to SenseLow or SenseHigh from SenseNone
    set_sense_ext_nmi! {Async self, NmiExtInt, High}
    set_sense_ext_nmi! {Async self, NmiExtInt, Low}
}
impl<I, C> NmiExtInt<I, C, AsyncOnly, WithoutClock, SenseLow>
where
    I: NmiEI,
    C: InterruptConfig,
{
    // Allow switching to SenseHigh from SenseLow
    set_sense_ext_nmi! {Async self, NmiExtInt, High}
    // And back to SenseNone
    set_sense_ext_nmi! {Async self, NmiExtInt, None}
}
impl<I, C> NmiExtInt<I, C, AsyncOnly, WithoutClock, SenseHigh>
where
    I: NmiEI,
    C: InterruptConfig,
{
    // Allow switching to SenseLow from SenseHigh
    set_sense_ext_nmi! {Async self, NmiExtInt, Low}
    // And back to SenseNone
    set_sense_ext_nmi! {Async self, NmiExtInt, None}
}
impl<I, C, CS, AM> NmiExtInt<I, C, AM, WithClock<CS>, SenseLow>
where
    I: NmiEI,
    C: InterruptConfig,
    AM: AnyMode,
    CS: EIClkSrcMarker,
{
    // Allow switching to SenseLow from SenseHigh
    set_sense_ext_nmi! {self, NmiExtInt, High}
}
impl<I, C, CS, AM> NmiExtInt<I, C, AM, WithClock<CS>, SenseHigh>
where
    I: NmiEI,
    C: InterruptConfig,
    AM: AnyMode,
    CS: EIClkSrcMarker,
{
    // Allow switching to SenseHigh from SenseLow
    set_sense_ext_nmi! {self, NmiExtInt, Low}
}

impl<I, C, CS, AM, AS> NmiExtInt<I, C, AM, WithClock<CS>, AS>
where
    I: NmiEI,
    C: InterruptConfig,
    AM: AnyMode,
    AS: AnySenseMode,
    CS: EIClkSrcMarker,
{
    // A clock source is present, all EdgeDetect senses are available
    set_sense_ext_nmi! {self, NmiExtInt, Both}
    set_sense_ext_nmi! {self, NmiExtInt, Rise}
    set_sense_ext_nmi! {self, NmiExtInt, Fall}
    // And back to SenseNone
    set_sense_ext_nmi! {self, NmiExtInt, None}
}
