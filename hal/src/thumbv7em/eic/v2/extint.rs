use core::marker::PhantomData;

use crate::gpio::v2::{Interrupt, InterruptConfig, Pin};

use crate::typelevel::{Is, NoneT, Sealed};

use crate::eic::v2::*;

use core::mem::transmute;

pub mod extintasync;
pub mod extintsync;

pub use extintasync::*;
pub use extintsync::*;

//==============================================================================
// ExtInt
//==============================================================================

// The pin-level struct
// It must be generic over PinId, Interrupt PinMode configuration
// (i.e. Floating, PullUp, or PullDown)
/// TODO
pub struct ExtInt<I, C, F, B, S>
where
    I: GetEINum,
    C: InterruptConfig,
    F: FilteringT,
    B: DebouncingT,
    S: SenseModeT,
{
    regs: Registers<I::EINum>,
    #[allow(dead_code)]
    pin: Pin<I, Interrupt<C>>,
    filtering: PhantomData<F>,
    debouncing: PhantomData<B>,
    sensemode: PhantomData<S>,
}

impl<I, C, F, B, S> Sealed for ExtInt<I, C, F, B, S>
where
    I: GetEINum,
    C: InterruptConfig,
    F: FilteringT,
    B: DebouncingT,
    S: SenseModeT,
{
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

pub trait ExtIntT: AnyExtInt {
    // Do not need access to the EIController here
    /// Read the pin state of the ExtInt
    /// TODO
    fn pin_state(&self) -> bool;
}


/*
impl<I, C, F, B, S> ExtIntT for SyncExtInt<I, C, F, B, S>
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
    type Filtering: FilteringT;
    //type Mode: DetectionMode;
    /// TODO
    type Debouncing: DebouncingT;
    /// TODO
    type SenseMode: SenseModeT;
}

impl<I, C, F, B, S> AnyExtInt for ExtInt<I, C, F, B, S>
where
    I: EINum + GetEINum,
    C: InterruptConfig,
    F: FilteringT,
    B: DebouncingT,
    S: SenseModeT,
{
    /// TODO
    type Num = I;
    /// TODO
    type Pin = C;
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
