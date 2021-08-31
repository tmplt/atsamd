use core::marker::PhantomData;

use crate::clock::types::{Counter, Enabled};
use crate::gpio::v2::{Interrupt, InterruptConfig, Pin};

use crate::eic::v2::*;

pub mod extintsync;
pub mod extintasync;

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
