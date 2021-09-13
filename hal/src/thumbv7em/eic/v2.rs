use core::marker::PhantomData;

use seq_macro::seq;

use crate::clock::types::{Counter, Enabled};
use crate::clock::v2::osculp32k::OscUlp32k;
use crate::clock::v2::pclk::{Eic, Pclk, PclkSourceMarker};
use crate::clock::v2::rtc::{Active32k, Output1k};
use crate::gpio::v2::{self as gpio, PinId};
use crate::pac::eic::{ctrla::CKSEL_A, dprescaler::*, RegisterBlock};
use crate::typelevel::{Is, Sealed};

pub mod eicontroller;
pub mod extint;

pub use crate::eic::v2::eicontroller::*;
pub use crate::eic::v2::extint::{asynconly::*, debounced::*, filtered::*};

//==============================================================================
// Sense
//==============================================================================

/// Type class for all possible [`SenseMode`] types
///
/// This trait uses the [`AnyKind`] trait pattern to create a [type class] for
/// [`Config`] types. See the `AnyKind` documentation for more details on the
/// pattern.
///
/// [`AnyKind`]: crate::typelevel#anykind-trait-pattern
/// [type class]: crate::typelevel#type-classes
pub trait AnySenseMode: Sealed + Is<Type = SpecificSenseMode<Self>> {
    type Mode: SenseMode;
}

pub type SpecificSenseMode<S> = <S as AnySenseMode>::Mode;

/*
impl<S> AnySenseMode for S
where
    S: SenseMode,
{
    type Mode = S;
}

impl<S: SenseMode> AsRef<Self> for S {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}
*/

/*
impl AsMut<Self> for Sense {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
*/

// Need a custom type, because the PAC has 8 identical copies
// of the same enum. There's probably a way to patch the PAC
/// Detection Mode
/// TODO
pub enum Sense {
    None = 0,
    Rise,
    Fall,
    Both,
    High,
    Low,
}

/// TODO
pub trait SenseMode: Sealed {
    const SENSE: Sense;
}

/// TODO
pub enum SenseNone {}
/// TODO
pub enum SenseRise {}
/// TODO
pub enum SenseFall {}
/// TODO
pub enum SenseBoth {}
/// TODO
pub enum SenseHigh {}
/// TODO
pub enum SenseLow {}

impl Sealed for SenseNone {}
impl Sealed for SenseRise {}
impl Sealed for SenseFall {}
impl Sealed for SenseBoth {}
impl Sealed for SenseHigh {}
impl Sealed for SenseLow {}

impl SenseMode for SenseNone {
    const SENSE: Sense = Sense::None;
}
impl SenseMode for SenseRise {
    const SENSE: Sense = Sense::Rise;
}
impl SenseMode for SenseFall {
    const SENSE: Sense = Sense::Fall;
}
impl SenseMode for SenseBoth {
    const SENSE: Sense = Sense::Both;
}
impl SenseMode for SenseHigh {
    const SENSE: Sense = Sense::High;
}
impl SenseMode for SenseLow {
    const SENSE: Sense = Sense::Low;
}

/// Valid SenseModes for Level Detection
/// TODO
pub trait LevelDetectMode: SenseMode {}
impl LevelDetectMode for SenseHigh {}
impl LevelDetectMode for SenseLow {}

/// Valid SenseModes for Edge Detection
/// TODO
pub trait EdgeDetectMode: SenseMode {}
impl EdgeDetectMode for SenseRise {}
impl EdgeDetectMode for SenseFall {}
impl EdgeDetectMode for SenseBoth {}

/// Valid SenseModes with Debouncing active
/// TODO
pub trait DebounceMode: EdgeDetectMode {}
impl DebounceMode for SenseRise {}
impl DebounceMode for SenseFall {}
impl DebounceMode for SenseBoth {}

//==============================================================================
// Debouncer
//==============================================================================

/*
/// TODO
pub trait Debouncing: Sealed {}

/// Debouncing is enabled
pub enum DebouncingEnabled {}
impl Sealed for DebouncingEnabled {}
impl Debouncing for DebouncingEnabled {}

/// Debouncing is disabled
pub enum DebouncingDisabled {}
impl Sealed for DebouncingDisabled {}
impl Debouncing for DebouncingDisabled {}
*/

/// TODO
pub struct DebouncerSettings {
    pub tickon: TICKON_A,
    pub prescaler0: PRESCALER0_A,
    pub states0: STATES0_A,
    pub prescaler1: PRESCALER1_A,
    pub states1: STATES1_A,
}

//==============================================================================
// Filtering
//==============================================================================

/*
/// TODO
pub trait Filtering: Sealed {}

/// Filtering is enabled
pub enum FilteringEnabled {}
impl Sealed for FilteringEnabled {}
impl Filtering for FilteringEnabled {}
//impl AnyFilterMode for FilteringEnabled {}

/// Filtering is disabled
pub enum FilteringDisabled {}
impl Sealed for FilteringDisabled {}
impl Filtering for FilteringDisabled {}
//impl AnyFilterMode for FilteringDisabled {}

pub trait AnyFilterMode: Sealed + Is<Type = SpecificFilterMode<Self>> {
    type Mode: Filtering;
}

pub type SpecificFilterMode<F> = <F as AnyFilterMode>::Mode;
*/

//==============================================================================
// EINum
//==============================================================================

// Type-level enum for the ExtInt number
// Each PinId is mapped to one and only one
/// TODO
pub trait EINum: Sealed {
    const NUM: u8;
    const MASK: u16 = 1 << Self::NUM;
    // Filten described by arithmetic series
    // 7+(n-1)*4
    const FILTEN: u32 = 1 << (7 + (Self::NUM - 1) * 4);
    // Sense described by the arithmetic series
    // 2+(n-1)*4
    const SENSE: u32 = 111 << (2 + (Self::NUM - 1) * 4);
    // Possibly other constants
}

seq!(N in 00..16 {
    /// TODO
    pub enum EI#N {}
    impl Sealed for EI#N {}
    impl EINum for EI#N {
        const NUM: u8 = N;
    }
});

//==============================================================================
// Registers
//==============================================================================

// Private struct that provides access to the EIC registers from
// the ExtInt types. We must be careful about memory safety here
/// TODO
struct Registers<E: EINum> {
    ei_num: PhantomData<E>,
}

impl<E: EINum> Registers<E> {
    // Unsafe because you must make there is only one copy
    // of Registers for each unique E
    /// TODO
    unsafe fn new() -> Self {
        Registers {
            ei_num: PhantomData,
        }
    }

    /// TODO
    fn eic(&self) -> &RegisterBlock {
        unsafe { &*crate::pac::EIC::ptr() }
    }

    /// TODO
    fn pin_state(&self) -> bool {
        let state = self.eic().pinstate.read().pinstate().bits();
        (state & E::MASK) != 0
    }

    // Can't add methods that access registers that share state
    // between different ExtInt. Those most be added to EIController
}

//==============================================================================
// Token
//==============================================================================

// Singleton token structs
// We need to create exactly 16 of these at boot.
// A token will be consumed when creating an ExtInt.
// This will prevent multiple pins from using the same interrupt
/// TODO
pub struct Token<E: EINum> {
    regs: Registers<E>,
}

impl<E: EINum> Token<E> {
    // Unsafe because you must make sure each Token is a singleton
    /// TODO
    unsafe fn new() -> Self {
        Token {
            regs: Registers::new(),
        }
    }
}

seq!(N in 00..16 {
    /// TODO
    pub struct Tokens {
        #(
            #[allow(dead_code)]
            pub ext_int_#N: Token<EI#N>,
        )*
    }

    impl Tokens {
        // Unsafe because you must make sure each Token is a singleton
        /// TODO
        unsafe fn new() -> Self {
            Tokens {
                #(
                    ext_int_#N: Token::new(),
                )*
            }
        }
    }
});

//==============================================================================
// Clock
//==============================================================================

// Synchronous vs. asynchronous detection
/// TODO
pub trait Clock: Sealed {}

/// AsyncMode only allows asynchronous detection
pub struct NoClock {}
impl Sealed for NoClock {}
impl Clock for NoClock {}

// When in WithClock, we have to store a clock resource
/// SyncMode allows full EIC functionality
///
/// Required if:
/// * The NMI Using edge detection or filtering
/// * One EXTINT uses filtering
/// * One EXTINT uses synchronous edge detection
/// * One EXTINT uses debouncing
pub struct WithClock<C: EIClkSrc> {
    /// Clock resource
    _clock: PhantomData<C>,
}
impl<C: EIClkSrc> Sealed for WithClock<C> {}
impl<C: EIClkSrc> Clock for WithClock<C> {}

pub trait AnyClock: Sealed + Is<Type = SpecificClock<Self>> {
    type Mode: Clock;
}

/// TODO
pub type SpecificClock<K> = <K as AnyClock>::Mode;

impl AnyClock for NoClock {
    type Mode = NoClock;
}

impl<CS> AnyClock for WithClock<CS>
where
    CS: EIClkSrc,
{
    type Mode = WithClock<CS>;
}

impl AsRef<Self> for NoClock {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<Self> for NoClock {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<CS> AsRef<Self> for WithClock<CS>
where
    CS: EIClkSrc,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<CS> AsMut<Self> for WithClock<CS>
where
    CS: EIClkSrc,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

// EI clock source for synchronous detection modes
/// TODO
pub trait EIClkSrc: Sealed {
    const CKSEL: CKSEL_A;
}

// Peripheral channel clock, routed from a GCLK
impl<T: PclkSourceMarker> EIClkSrc for Pclk<Eic, T> {
    /// TODO
    const CKSEL: CKSEL_A = CKSEL_A::CLK_GCLK;
}

// Ultra-low power oscillator can be used instead
impl<Y: Output1k, N: Counter> EIClkSrc for Enabled<OscUlp32k<Active32k, Y>, N> {
    /// TODO
    const CKSEL: CKSEL_A = CKSEL_A::CLK_ULP32K;
}

//==============================================================================
// GetEINum
//==============================================================================

/// Type-level function to get the EINum from a PinId
pub trait GetEINum: PinId {
    type EINum: EINum;
}

macro_rules! impl_get_ei_num (
    ($PinId:ident, $EINum:ident, $NUM:literal) => {
        impl GetEINum for gpio::$PinId {
            type EINum = $EINum;
        }
    }
);

// Need many more of these. But be careful, because the pin number
// doesn't always match the EINum
// impl_get_ei_num!(PA00, EI00, 0);
//
// See bottom of file for full list

// ExtInt 0
impl_get_ei_num!(PA00, EI00, 0);
impl_get_ei_num!(PA16, EI00, 0);
#[cfg(feature = "min-samd51j")]
impl_get_ei_num!(PB00, EI00, 0);
#[cfg(feature = "min-samd51j")]
impl_get_ei_num!(PB16, EI00, 0);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC00, EI00, 0);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC16, EI00, 0);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PD00, EI00, 0);

// ExtInt 1
impl_get_ei_num!(PA01, EI01, 1);
impl_get_ei_num!(PA17, EI01, 1);
#[cfg(feature = "min-samd51j")]
impl_get_ei_num!(PB01, EI01, 1);
#[cfg(feature = "min-samd51j")]
impl_get_ei_num!(PB17, EI01, 1);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC01, EI01, 1);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC17, EI01, 1);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PD01, EI01, 1);

// ExtInt 2
impl_get_ei_num!(PA02, EI02, 2);
impl_get_ei_num!(PA18, EI02, 2);
impl_get_ei_num!(PB02, EI02, 2);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PB18, EI02, 2);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC02, EI02, 2);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC18, EI02, 2);

// ExtInt 3
impl_get_ei_num!(PA03, EI03, 3);
impl_get_ei_num!(PA19, EI03, 3);
impl_get_ei_num!(PB03, EI03, 3);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PB19, EI03, 3);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC03, EI03, 3);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC19, EI03, 3);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PD08, EI03, 3);

// ExtInt 4
impl_get_ei_num!(PA04, EI04, 4);
impl_get_ei_num!(PA20, EI04, 4);
#[cfg(feature = "min-samd51j")]
impl_get_ei_num!(PB04, EI04, 4);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PB20, EI04, 4);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PC04, EI04, 4);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC20, EI04, 4);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PD9, EI04, 4);

// ExtInt 5
impl_get_ei_num!(PA05, EI05, 5);
impl_get_ei_num!(PA21, EI05, 5);
#[cfg(feature = "min-samd51j")]
impl_get_ei_num!(PB05, EI05, 5);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PB21, EI05, 5);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC05, EI05, 5);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC21, EI05, 5);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PD10, EI05, 5);

// ExtInt 6
impl_get_ei_num!(PA06, EI06, 6);
impl_get_ei_num!(PA22, EI06, 6);
#[cfg(feature = "min-samd51j")]
impl_get_ei_num!(PB06, EI06, 6);
impl_get_ei_num!(PB22, EI06, 6);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC06, EI06, 6);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PC22, EI06, 6);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PD11, EI06, 6);

// ExtInt 7
impl_get_ei_num!(PA07, EI07, 7);
impl_get_ei_num!(PA23, EI07, 7);
#[cfg(feature = "min-samd51j")]
impl_get_ei_num!(PB07, EI07, 7);
impl_get_ei_num!(PB23, EI07, 7);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PC23, EI07, 7);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PD12, EI07, 7);

// ExtInt 8
impl_get_ei_num!(PA24, EI08, 8);
impl_get_ei_num!(PB08, EI08, 8);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PB24, EI08, 8);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC24, EI08, 8);

// ExtInt 9
impl_get_ei_num!(PA09, EI08, 7);
impl_get_ei_num!(PA25, EI08, 7);
impl_get_ei_num!(PB09, EI08, 7);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PB25, EI08, 7);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC07, EI07, 7);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC25, EI08, 7);

// ExtInt 10
impl_get_ei_num!(PA10, EI10, 10);
impl_get_ei_num!(PB10, EI10, 10);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC10, EI10, 10);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC26, EI10, 10);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PD20, EI10, 10);

// ExtInt 11
impl_get_ei_num!(PA11, EI11, 11);
impl_get_ei_num!(PA27, EI11, 11);
impl_get_ei_num!(PB11, EI11, 11);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC11, EI11, 11);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC27, EI11, 11);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PD21, EI11, 11);

// ExtInt 12
impl_get_ei_num!(PA12, EI12, 12);
#[cfg(feature = "min-samd51j")]
impl_get_ei_num!(PB12, EI12, 12);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PB26, EI12, 12);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC12, EI12, 12);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC28, EI12, 12);

// ExtInt 13
impl_get_ei_num!(PA13, EI13, 13);
#[cfg(feature = "min-samd51j")]
impl_get_ei_num!(PB13, EI13, 13);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PB27, EI13, 13);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC13, EI13, 13);

// ExtInt 14
impl_get_ei_num!(PA14, EI14, 14);
impl_get_ei_num!(PA30, EI14, 14);
#[cfg(feature = "min-samd51j")]
impl_get_ei_num!(PB14, EI14, 14);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PB28, EI14, 14);
#[cfg(feature = "min-samd51j")]
impl_get_ei_num!(PB30, EI14, 14);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC14, EI14, 14);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PC30, EI14, 14);

// ExtInt 15
impl_get_ei_num!(PA15, EI15, 15);
impl_get_ei_num!(PA31, EI15, 15);
#[cfg(feature = "min-samd51j")]
impl_get_ei_num!(PB15, EI15, 15);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PB29, EI15, 15);
#[cfg(feature = "min-samd51j")]
impl_get_ei_num!(PB31, EI15, 15);
#[cfg(feature = "min-samd51n")]
impl_get_ei_num!(PC15, EI15, 15);
#[cfg(feature = "min-samd51p")]
impl_get_ei_num!(PC31, EI15, 15);
