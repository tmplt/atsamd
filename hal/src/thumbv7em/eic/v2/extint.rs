use core::marker::PhantomData;

use crate::gpio::v2::{Interrupt, InterruptConfig, Pin};

use crate::typelevel::{Is, Sealed};

use crate::eic::v2::*;

use core::mem::transmute;

pub mod extintasync;
pub mod extintsync;

pub use extintasync::*;
pub use extintsync::*;

//==============================================================================
// ExternalInterrupt
//==============================================================================
pub struct ExternalInterrupt<E>
where
    E: AnyExtInt,
{
    regs: Registers<E::Num>,
}

impl<E> ExternalInterrupt<E>
where
    E: AnyExtInt<
        Clock = NoClock,
        Filtering = FilteringDisabled,
        Debouncing = DebouncingDisabled,
    >,
{
    pub fn test(&self) {}
}

impl<E> ExternalInterrupt<E>
where
    E: AnyExtInt,
{
    // Do not need access to the EIController here
    /// Read the pin state of the ExtInt
    /// TODO
    pub fn pin_state(&self) -> bool {
        self.regs.pin_state()
    }
}

//==============================================================================
// ExtInt
//==============================================================================

// The pin-level struct
// It must be generic over PinId, Interrupt PinMode configuration
// (i.e. Floating, PullUp, or PullDown)
/// TODO
pub struct ExtInt<I, C, M, F, B, S>
where
    I: GetEINum,
    C: InterruptConfig,
    M: Clock,
    F: FilteringT,
    B: DebouncingT,
    S: SenseModeT,
{
    regs: Registers<I::EINum>,
    #[allow(dead_code)]
    pin: Pin<I, Interrupt<C>>,
    clockmode: PhantomData<M>,
    filtering: PhantomData<F>,
    debouncing: PhantomData<B>,
    sensemode: PhantomData<S>,
}

impl<I, C, M, F, B, S> Sealed for ExtInt<I, C, M, F, B, S>
where
    I: GetEINum,
    C: InterruptConfig,
    M: Clock,
    F: FilteringT,
    B: DebouncingT,
    S: SenseModeT,
{
}

impl<I, C, K> ExtInt<I, C, WithClock<K>, FilteringDisabled, DebouncingDisabled, SenseNone>
where
    I: GetEINum,
    C: InterruptConfig,
    K: EIClkSrc,
{
    /// TODO
    pub(crate) fn new_sync(token: Token<I::EINum>, pin: Pin<I, Interrupt<C>>) -> Self {
        // Configure the ExtInt (e.g. set the Asynchronous Mode register)
        ExtInt {
            regs: token.regs,
            pin,
            clockmode: PhantomData,
            filtering: PhantomData,
            debouncing: PhantomData,
            sensemode: PhantomData,
        }
    }
}

impl<I, C> ExtInt<I, C, NoClock, FilteringDisabled, DebouncingDisabled, SenseNone>
where
    I: GetEINum,
    C: InterruptConfig,
{
    /// TODO
    pub(crate) fn new_async(token: Token<I::EINum>, pin: Pin<I, Interrupt<C>>) -> Self {
        // Configure the AsyncExtInt (e.g. set the Asynchronous Mode register)
        ExtInt {
            regs: token.regs,
            pin,
            clockmode: PhantomData,
            filtering: PhantomData,
            debouncing: PhantomData,
            sensemode: PhantomData,
        }
    }
}

impl<I, C, S> ExtInt<I, C, NoClock, FilteringDisabled, DebouncingDisabled, S>
where
    I: GetEINum,
    C: InterruptConfig,
    S: SenseModeT,
{
    /// TODO
    pub fn set_sense<K, N>(&self, eic: &mut Enabled<EIController<NoClock>, N>, sense: Sense)
    where
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(sense);
    }
}

impl<I, C, K, S> ExtInt<I, C, WithClock<K>, FilteringDisabled, DebouncingDisabled, S>
where
    I: GetEINum,
    C: InterruptConfig,
    K: EIClkSrc,
    S: SenseModeT,
{
    // Methods related to filtering and debouncing go here,
    // since they require a clock

    // Must have access to the EIController here
    /// TODO
    pub fn set_sense<N>(&self, eic: &mut Enabled<EIController<WithClock<K>>, N>, sense: Sense)
    where
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(sense);
    }

    /// TODO
    pub fn enable_debouncer<N>(
        self,
        eic: &mut Enabled<EIController<WithClock<K>>, N>,
    ) -> ExtInt<I, C, WithClock<K>, FilteringDisabled, DebouncingEnabled, S>
    where
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.enable_debouncer::<I::EINum>();
        ExtInt {
            regs: self.regs,
            pin: self.pin,
            clockmode: self.clockmode,
            filtering: self.filtering,
            debouncing: PhantomData::<DebouncingEnabled>,
            sensemode: self.sensemode,
        }
    }

    // Must have access to the EIController here
    /// TODO
    pub fn enable_filtering<N>(
        self,
        eic: &mut Enabled<EIController<WithClock<K>>, N>,
    ) -> ExtInt<I, C, WithClock<K>, FilteringEnabled, DebouncingDisabled, S>
    where
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.enable_filtering::<I::EINum>();
        ExtInt {
            regs: self.regs,
            pin: self.pin,
            clockmode: self.clockmode,
            filtering: PhantomData::<FilteringEnabled>,
            debouncing: self.debouncing,
            sensemode: self.sensemode,
        }
    }
}

impl<I, C, K, S> ExtInt<I, C, WithClock<K>, FilteringDisabled, DebouncingEnabled, S>
where
    I: GetEINum,
    C: InterruptConfig,
    K: EIClkSrc,
    S: SenseModeT,
{
    /// TODO
    pub fn set_debouncer_settings<N>(
        self,
        eic: &mut Enabled<EIController<WithClock<K>>, N>,
        settings: &DebouncerSettings,
    ) where
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.set_debouncer_settings::<I::EINum>(settings);
    }
}

/*
impl<I, C, F, B, S> ExtInt<I, C, F, B, S>
where
    I: GetEINum,
    C: InterruptConfig,
    F: FilteringT,
    B: DebouncingT,
    S: SenseModeT,
{
    // Do not need access to the EIController here
    /// Read the pin state of the ExtInt
    /// TODO
    pub fn pin_state(&self) -> bool {
        self.regs.pin_state()
    }
}

impl<I, C, F, S> ExtInt<I, C, F, DebouncingDisabled, S>
where
    I: GetEINum,
    C: InterruptConfig,
    F: FilteringT,
    S: SenseModeT,
{
}
*/

/*
impl<I, C, F, B, S> ExtIntT for ExtInt<I, C, F, B, S>
where
    I: GetEINum,
    C: InterruptConfig,
    F: FilteringT,
    B: DebouncingT,
    S: SenseModeT,
{}
*/

//==============================================================================
// AnyExtInt
//==============================================================================

// It probably makes sense to implement the `AnyKind` pattern for ExtInt
pub trait AnyExtInt: Is<Type = SpecificExtInt<Self>>
where
    Self: Sealed,
    Self: From<SpecificExtInt<Self>>,
    Self: Into<SpecificExtInt<Self>>,
    Self: AsRef<SpecificExtInt<Self>>,
    Self: AsMut<SpecificExtInt<Self>>,
{
    /// Associated type representing the ExtInt number [`EINum`]
    type Num: EINum + GetEINum;
    /// TODO
    type Pin: InterruptConfig;
    /// TODO
    type Clock: Clock;
    /// TODO
    type Filtering: FilteringT;
    /// TODO
    type Debouncing: DebouncingT;
    /// TODO
    type SenseMode: SenseModeT;
}

impl<I, C, M, F, B, S> AnyExtInt for ExtInt<I, C, M, F, B, S>
where
    I: EINum + GetEINum,
    C: InterruptConfig,
    M: Clock,
    F: FilteringT,
    B: DebouncingT,
    S: SenseModeT,
{
    /// TODO
    type Num = I;
    /// TODO
    type Pin = C;
    /// TODO
    type Clock = M;
    /// TODO
    type Filtering = F;
    /// TODO
    type Debouncing = B;
    /// TODO
    type SenseMode = S;
}
pub type SpecificExtInt<E> = ExtInt<
    <E as AnyExtInt>::Num,
    <E as AnyExtInt>::Pin,
    <E as AnyExtInt>::Clock,
    <E as AnyExtInt>::Filtering,
    <E as AnyExtInt>::Debouncing,
    <E as AnyExtInt>::SenseMode,
>;

impl<E: AnyExtInt> AsRef<E> for SpecificExtInt<E> {
    #[inline]
    fn as_ref(&self) -> &E {
        unsafe { transmute(self) }
    }
}

impl<E: AnyExtInt> AsMut<E> for SpecificExtInt<E> {
    #[inline]
    fn as_mut(&mut self) -> &mut E {
        unsafe { transmute(self) }
    }
}

/*
impl<E: AnyExtInt> From<E> for SpecificExtInt<E> {
    #[inline]
    fn from(&self) -> Self {
        SpecificExtInt {
            regs: Registers::<self::Num>,
            pin: self::Pin,
            filtering: self::Filtering,
            debouncing: self::Debouncing,
            sensemode: self::SenseMode,
        }
    }
}
*/
