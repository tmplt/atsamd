use core::marker::PhantomData;

use crate::gpio::v2::{Interrupt, InterruptConfig, Pin};

use crate::typelevel::{Is, Sealed};

use crate::eic::v2::*;

use core::mem::transmute;

pub mod asynconly;
pub mod debounced;
pub mod filtered;

pub use asynconly::*;
pub use debounced::*;
pub use filtered::*;

//==============================================================================
// FilteredExtInt
//==============================================================================
/*
pub struct FilteredExtInt<E>
where
    E: AnyExtInt,
{
    extint: E,
}

impl<E> FilteredExtInt<E>
where
    E: AnyExtInt<Filtering = FilteringEnabled, Debouncing = DebouncingDisabled>,
{
    pub fn test(&self) {}
}

pub struct DebouncedExtInt<E>
where
    E: AnyExtInt,
{
    extint: ExtInt<
        <E as AnyExtInt>::Num,
        <E as AnyExtInt>::Pin,
        WithClock<<E as AnyExtInt>::Clock>,
        FilteringDisabled,
        DebouncingEnabled,
        <E as AnyExtInt>::SenseMode,
    >,
}
impl<E> DebouncedExtInt<E>
where
    E: AnyExtInt<Filtering = FilteringDisabled, Debouncing = DebouncingEnabled>,
{
    // Do not need access to the EIController here
    /// Read tf0138040.dnghe pin state of the ExtInt
    /// TODO
    //pub fn pin_state(&self) -> bool {
    //self.extint.regs.pin_state()
    //self.extint.set_sense(
    //}

    /// Set the sense mode
    /// TODOf0138040.dng
    pub fn set_sense<K, N>(&self, eic: &mut Enabled<EIController<WithClock<K>>, N>, sense: Sense)
    where
        K: EIClkSrc,
        N: Counter,
    {
        self.extint.set_sense(sense);
    }
}
*/

//==============================================================================
// ExtInt
//==============================================================================

// The pin-level struct
// It must be generic over PinId, Interrupt PinMode configuration
// (i.e. Floating, PullUp, or PullDown)
/// TODO
pub struct ExtInt<I, C, K, S>
where
    I: GetEINum,
    C: InterruptConfig,
    K: Clock,
    S: SenseMode,
{
    regs: Registers<I::EINum>,
    #[allow(dead_code)]
    pin: Pin<I, Interrupt<C>>,
    clockmode: PhantomData<K>,
    sensemode: PhantomData<S>,
}

// Sealed for ExtInt
impl<I, C, K, S> Sealed for ExtInt<I, C, K, S>
where
    I: GetEINum,
    C: InterruptConfig,
    K: Clock,
    S: SenseMode,
{
}

impl<I, C, CS> ExtInt<I, C, WithClock<CS>, SenseNone>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
{
    /// Create initial synchronous ExtInt
    /// TODO
    pub(crate) fn new_sync(token: Token<I::EINum>, pin: Pin<I, Interrupt<C>>) -> Self {
        ExtInt {
            regs: token.regs,
            pin,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}

impl<I, C> ExtInt<I, C, NoClock, SenseNone>
where
    I: GetEINum,
    C: InterruptConfig,
{
    /// Create initial asynchronous ExtInt
    /// TODO
    pub(crate) fn new_async(token: Token<I::EINum>, pin: Pin<I, Interrupt<C>>) -> Self {
        // #TODO
        // Configure the AsyncExtInt (e.g. set the Asynchronous Mode register)
        ExtInt {
            regs: token.regs,
            pin,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}

// Methods for any state of ExtInt
impl<I, C, K, S> ExtInt<I, C, K, S>
where
    I: GetEINum,
    C: InterruptConfig,
    K: Clock,
    S: SenseMode,
{
    // Must have access to the EIController here
    /// TODO
    pub fn set_sense_none<N>(
        self,
        eic: &mut Enabled<EIController<K>, N>,
    ) -> ExtInt<I, C, K, SenseNone>
    where
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(Sense::None);

        ExtInt {
            regs: self.regs,
            pin: self.pin,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
    /// TODO
    pub fn set_sense_high<N>(
        self,
        eic: &mut Enabled<EIController<K>, N>,
    ) -> ExtInt<I, C, K, SenseHigh>
    where
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(Sense::High);

        ExtInt {
            regs: self.regs,
            pin: self.pin,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
    /// TODO
    pub fn set_sense_low<N>(
        self,
        eic: &mut Enabled<EIController<K>, N>,
    ) -> ExtInt<I, C, K, SenseLow>
    where
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(Sense::Low);

        ExtInt {
            regs: self.regs,
            pin: self.pin,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
    /// TODO
    pub fn set_sense_rise<N>(
        self,
        eic: &mut Enabled<EIController<K>, N>,
    ) -> ExtInt<I, C, K, SenseRise>
    where
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(Sense::Rise);

        ExtInt {
            regs: self.regs,
            pin: self.pin,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
    /// TODO
    pub fn set_sense_fall<N>(
        self,
        eic: &mut Enabled<EIController<K>, N>,
    ) -> ExtInt<I, C, K, SenseFall>
    where
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(Sense::Fall);

        ExtInt {
            regs: self.regs,
            pin: self.pin,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
    /// TODO
    pub fn set_sense_both<N>(
        self,
        eic: &mut Enabled<EIController<K>, N>,
    ) -> ExtInt<I, C, K, SenseBoth>
    where
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(Sense::Both);

        ExtInt {
            regs: self.regs,
            pin: self.pin,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }

    // Do not need access to the EIController here
    /// Read the pin state of the ExtInt
    /// TODO
    pub fn pin_state(&self) -> bool {
        self.regs.pin_state()
    }
}

// Methods for any state of ExtInt
impl<I, C, K> ExtInt<I, C, K, SenseNone>
where
    I: GetEINum,
    C: InterruptConfig,
    K: Clock,
{
}

impl<I, C, CS, S> ExtInt<I, C, WithClock<CS>, S>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    S: DebounceMode,
{
    // Methods related to debouncing go here since they require a clock
    // and that SenseMode are one of: Rise, Fall or Both

    /// TODO
    pub fn enable_debouncing<N>(
        self,
        eic: &mut Enabled<EIController<WithClock<CS>>, N>,
    ) -> DebouncedExtInt<I, C, CS, S>
    where
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.enable_debouncing::<I::EINum>();
        DebouncedExtInt { extint: self }
    }
}

impl<I, C, CS, S> ExtInt<I, C, WithClock<CS>, S>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    S: SenseMode,
{
    // Methods related to filtering and debouncing go here,
    // since they require a clock

    // Must have access to the EIController here
    /// TODO
    pub fn enable_filtering<N>(
        self,
        eic: &mut Enabled<EIController<WithClock<CS>>, N>,
    ) -> FilteredExtInt<I, C, WithClock<CS>, S>
    where
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.enable_filtering::<I::EINum>();
        FilteredExtInt { extint: self }
    }
}

impl<I, C, K, S> ExtInt<I, C, WithClock<K>, S>
where
    I: GetEINum,
    C: InterruptConfig,
    K: EIClkSrc,
    S: SenseMode,
{
}

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
    //type Filtering: Filtering;
    /// TODO
    //type Debouncing: Debouncing;
    /// TODO
    type SenseMode: SenseMode;
}

impl<I, C, K, S> AnyExtInt for ExtInt<I, C, K, S>
where
    I: EINum + GetEINum,
    C: InterruptConfig,
    K: Clock,
    S: SenseMode,
{
    /// TODO
    type Num = I;
    /// TODO
    type Pin = C;
    /// TODO
    type Clock = K;
    /// TODO
    //type Filtering = F;
    /// TODO
    //type Debouncing = B;
    /// TODO
    type SenseMode = S;
}
pub type SpecificExtInt<E> = ExtInt<
    <E as AnyExtInt>::Num,
    <E as AnyExtInt>::Pin,
    <E as AnyExtInt>::Clock,
    //<E as AnyExtInt>::Filtering,
    //<E as AnyExtInt>::Debouncing,
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
