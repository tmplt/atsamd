use core::marker::PhantomData;

use crate::clock::types::{Counter, Enabled};
use crate::gpio::v2::{Interrupt, InterruptConfig, Pin};

use crate::eic::v2::*;

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
    M: DetectionMode,
    F: Filtering,
    B: Debouncing,
    S: SenseMode,
{
    regs: Registers<I::EINum>,
    #[allow(dead_code)]
    pin: Pin<I, Interrupt<C>>,
    #[allow(dead_code)]
    mode: M,
    filtering: PhantomData<F>,
    debouncing: PhantomData<B>,
    sensemode: PhantomData<S>,
}
impl<I, C, M, F, B, S> ExtInt<I, C, M, F, B, S>
where
    I: GetEINum,
    C: InterruptConfig,
    M: DetectionMode,
    F: Filtering,
    B: Debouncing,
    S: SenseMode,
{
    // Do not need access to the EIController here
    /// Read the pin state of the ExtInt
    /// TODO
    pub fn pin_state(&self) -> bool {
        self.regs.pin_state()
    }
}
impl<I, C, M, F, S> ExtInt<I, C, M, F, DebouncingDisabled, S>
where
    I: GetEINum,
    C: InterruptConfig,
    M: DetectionMode,
    F: Filtering,
    S: SenseMode,
{
}

impl<I, C, S> ExtInt<I, C, AsyncMode, FilteringDisabled, DebouncingDisabled, S>
where
    I: GetEINum,
    C: InterruptConfig,
    S: SenseMode,
{
    /// TODO
    pub fn new_async(token: Token<I::EINum>, pin: Pin<I, Interrupt<C>>) -> Self {
        // Configure the ExtInt (e.g. set the Asynchronous Mode register)
        ExtInt {
            regs: token.regs,
            pin,
            mode: AsyncMode,
            filtering: PhantomData,
            debouncing: PhantomData,
            sensemode: PhantomData,
        }
    }
    /// TODO
    pub fn set_sense<K, N>(
        &self,
        eic: &mut Enabled<EIController<NoClockOnlyAsync>, N>,
        sense: Sense,
    ) where
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(sense);
    }
}

impl<I, C, S> ExtInt<I, C, SyncMode, FilteringDisabled, DebouncingDisabled, S>
where
    I: GetEINum,
    C: InterruptConfig,
    S: SenseMode,
{
    /// TODO
    pub fn new_sync(token: Token<I::EINum>, pin: Pin<I, Interrupt<C>>) -> Self {
        // Configure the ExtInt (e.g. set the Asynchronous Mode register)
        ExtInt {
            regs: token.regs,
            pin,
            mode: SyncMode,
            filtering: PhantomData,
            debouncing: PhantomData,
            sensemode: PhantomData,
        }
    }

    // Methods related to filtering and debouncing go here,
    // since they require a clock

    // Must have access to the EIController here
    /// TODO
    pub fn set_sense<K, N>(&self, eic: &mut Enabled<EIController<WithClock<K>>, N>, sense: Sense)
    where
        K: EIClkSrc,
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(sense);
    }

    /// TODO
    pub fn enable_debouncer<K, N>(
        self,
        eic: &mut Enabled<EIController<WithClock<K>>, N>,
    ) -> ExtInt<I, C, SyncMode, FilteringDisabled, DebouncingEnabled, S>
    where
        K: EIClkSrc,
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.enable_debouncer::<I::EINum>();
        ExtInt {
            regs: self.regs,
            pin: self.pin,
            mode: self.mode,
            filtering: self.filtering,
            debouncing: PhantomData::<DebouncingEnabled>,
            sensemode: self.sensemode,
        }
    }

    // Must have access to the EIController here
    /// TODO
    pub fn enable_filtering<K, N>(
        self,
        eic: &mut Enabled<EIController<WithClock<K>>, N>,
    ) -> ExtInt<I, C, SyncMode, FilteringEnabled, DebouncingDisabled, S>
    where
        K: EIClkSrc,
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.enable_filtering::<I::EINum>();
        ExtInt {
            regs: self.regs,
            pin: self.pin,
            mode: self.mode,
            filtering: PhantomData::<FilteringEnabled>,
            debouncing: self.debouncing,
            sensemode: self.sensemode,
        }
    }
}

impl<I, C, S> ExtInt<I, C, SyncMode, FilteringDisabled, DebouncingEnabled, S>
where
    I: GetEINum,
    C: InterruptConfig,
    S: SenseMode,
{
    /// TODO
    pub fn set_debouncer_settings<K, N>(
        self,
        eic: &mut Enabled<EIController<WithClock<K>>, N>,
        settings: &DebouncerSettings,
    ) where
        K: EIClkSrc,
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.set_debouncer_settings::<I::EINum>(settings);
    }
}

//==============================================================================
// AnyExtInt
//==============================================================================

// It probably makes sense to implement the `AnyKind` pattern for ExtInt
//pub trait AnyExtInt
//where
//Self: Sealed,
//Self: From<SpecificExtInt<Self>>,
//Self: Into<SpecificExtInt<Self>>,
//Self: AsRef<SpecificExtInt<Self>>,
//Self: AsMut<SpecificExtInt<Self>>,
//{
///// TODO
//type Num: EINum;
///// TODO
//type Pin: InterruptConfig;
///// TODO
//type Mode: DetectionMode;
//}

//pub type SpecificExtInt<E> =
//ExtInt<<E as AnyExtInt>::Num, <E as AnyExtInt>::Pin, <E as AnyExtInt>::Mode>;

//impl<E: AnyExtInt> From<E> for SpecificExtInt<E> {
//#[inline]
//fn from(&self) -> Self {
//SpecificExtInt {
//regs: Registers<self::Num>,
//pin: self::Pin,
//mode: self::DetectionMode,
//}
//}
//}
