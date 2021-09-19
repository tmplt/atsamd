use core::marker::PhantomData;

use paste::paste;
use seq_macro::seq;

use crate::clock::types::{Counter, Enabled};
use crate::clock::v2::osculp32k::OscUlp32k;
use crate::clock::v2::pclk::{Eic, Pclk, PclkSourceMarker};
use crate::clock::v2::rtc::{Active32k, Output1k};
use crate::gpio::v2::{self as gpio, PinId};
use crate::pac::eic::{ctrla::CKSEL_A, dprescaler::*, RegisterBlock};
use crate::typelevel::{Is, NoneT, Sealed};

pub mod eicontroller;
pub mod extint;

pub use crate::eic::v2::eicontroller::*;

//==============================================================================
// Mode
//==============================================================================

/// Detection Mode
/// TODO
pub enum EIMode {
    Normal = 0,
    AsyncOnly,
    Filtered,
    FilteredAsync,
    Debounced,
    DebouncedAsync,
}

/// TODO
pub trait Mode: Sealed {
    const MODE: EIMode;
}

/// TODO
pub enum Normal {}
/// TODO
pub enum AsyncOnly {}
/// TODO
pub enum Filtered {}
/// TODO
pub enum FilteredAsync {}
/// TODO
pub enum Debounced {}
/// TODO
pub enum DebouncedAsync {}

impl Sealed for Normal {}
impl Sealed for AsyncOnly {}
impl Sealed for Filtered {}
impl Sealed for FilteredAsync {}
impl Sealed for Debounced {}
impl Sealed for DebouncedAsync {}

impl Mode for Normal {
    const MODE: EIMode = EIMode::Normal;
}
impl Mode for AsyncOnly {
    const MODE: EIMode = EIMode::AsyncOnly;
}
impl Mode for Filtered {
    const MODE: EIMode = EIMode::Filtered;
}
impl Mode for FilteredAsync {
    const MODE: EIMode = EIMode::FilteredAsync;
}
impl Mode for Debounced {
    const MODE: EIMode = EIMode::Debounced;
}
impl Mode for DebouncedAsync {
    const MODE: EIMode = EIMode::DebouncedAsync;
}

//==============================================================================
// AnyMode
//==============================================================================
/// Type class for all possible [`Mode`] types
///
/// This trait uses the [`AnyKind`] trait pattern to create a [type class] for
/// [`Mode`] types. See the `AnyKind` documentation for more details on the
/// pattern.
///
/// [`AnyKind`]: crate::typelevel#anykind-trait-pattern
/// [type class]: crate::typelevel#type-classes
pub trait AnyMode: Sealed + Is<Type = SpecificMode<Self>> {
    type Mode: Mode;
}

pub type SpecificMode<S> = <S as AnyMode>::Mode;

macro_rules! any_mode {
    ($name:ident) => {
        paste! {
        impl AnyMode for [<$name>]
        {
            type Mode = [<$name>];
        }

        impl AsRef<Self> for [<$name>] {
            #[inline]
            fn as_ref(&self) -> &Self {
                self
            }
        }
        impl AsMut<Self> for [<$name>] {
            #[inline]
            fn as_mut(&mut self) -> &mut Self {
                self
            }
        }

                }
    };
}

any_mode!(Normal);
any_mode!(AsyncOnly);
any_mode!(Filtered);
any_mode!(FilteredAsync);
any_mode!(Debounced);
any_mode!(DebouncedAsync);

//==============================================================================
// Sense
//==============================================================================

// Need a custom type, because the PAC has 8 identical copies
// of the same enum. There's probably a way to patch the PAC

/// Detection Mode
///
/// Defines what triggers an interrupt and/or event
pub enum Sense {
    /// No detection
    None = 0,
    /// Rising-edge detection
    Rise,
    /// Falling-edge detection
    Fall,
    /// Both-edge detection
    Both,
    /// High-level detection
    High,
    /// Low-level detection
    Low,
}

/// Trait for all input [`Sense`] modes
pub trait SenseMode: Sealed {
    const SENSE: Sense;
}

/// No detection
pub enum SenseNone {}
/// Rising-edge detection
pub enum SenseRise {}
/// Falling-edge detection
pub enum SenseFall {}
/// Both-edge detection
pub enum SenseBoth {}
/// High-level detection
pub enum SenseHigh {}
/// Low-level detection
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
pub trait LevelDetectMode: SenseMode {}
impl LevelDetectMode for SenseHigh {}
impl LevelDetectMode for SenseLow {}

/// Valid SenseModes for Edge Detection
pub trait EdgeDetectMode: SenseMode {}
impl EdgeDetectMode for SenseRise {}
impl EdgeDetectMode for SenseFall {}
impl EdgeDetectMode for SenseBoth {}

/// Valid SenseModes with Debouncing active
pub trait DebounceMode: EdgeDetectMode {}
impl DebounceMode for SenseRise {}
impl DebounceMode for SenseFall {}
impl DebounceMode for SenseBoth {}

//==============================================================================
// AnySenseMode
//==============================================================================
/// Type class for all possible [`SenseMode`] types
///
/// This trait uses the [`AnyKind`] trait pattern to create a [type class] for
/// [`Sense`] types. See the `AnyKind` documentation for more details on the
/// pattern.
///
/// [`AnyKind`]: crate::typelevel#anykind-trait-pattern
/// [type class]: crate::typelevel#type-classes
pub trait AnySenseMode: SenseMode + Sealed + Is<Type = SpecificSenseMode<Self>> {
    type Mode: SenseMode;
}

pub type SpecificSenseMode<S> = <S as AnySenseMode>::Mode;

macro_rules! any_sense {
    ($name:ident) => {
        paste! {
        impl AnySenseMode for [<$name>]
        {
            type Mode = [<$name>];
        }

        impl AsRef<Self> for [<$name>] {
            #[inline]
            fn as_ref(&self) -> &Self {
                self
            }
        }
        impl AsMut<Self> for [<$name>] {
            #[inline]
            fn as_mut(&mut self) -> &mut Self {
                self
            }
        }

                }
    };
}

any_sense!(SenseNone);
any_sense!(SenseRise);
any_sense!(SenseFall);
any_sense!(SenseBoth);
any_sense!(SenseHigh);
any_sense!(SenseLow);

//==============================================================================
// Debouncer settings
//==============================================================================

/// TODO
pub struct DebouncerSettings {
    pub tickon: TICKON_A,
    pub prescaler0: PRESCALER0_A,
    pub states0: STATES0_A,
    pub prescaler1: PRESCALER1_A,
    pub states1: STATES1_A,
}

// EINum
//==============================================================================

// Type-level enum for the ExtInt number
// Each PinId is mapped to one and only one
/// TODO
pub trait EINum: Sealed {
    const NUM: u8;
    const OFFSET: u8 = match Self::NUM {
        0..=7 => 0,
        8.. => 1,
    };
    const MASK: u16 = 1 << Self::NUM;
    // Filten described by arithmetic series
    // 3+(n)*4
    const FILTEN: u8 = 3 + Self::NUM * 4;
    // Start of Sense slice described by the series
    // (n)*4. Bitmask for Sense is 0b111,
    // meaning MSB of sense is 2 "steps" from LSB
    const SENSELSB: u8 = Self::NUM * 4;
    const SENSEMSB: u8 = Self::NUM * 4 + 2;
    // Possibly other constants
}

seq!(N in 00..16 {
    paste! {
        #[doc = "Token type for ExtInt" N]
        pub enum EI#N {}
        impl Sealed for EI#N {}
        impl EINum for EI#N {
            const NUM: u8 = N;
        }
    }
});

#[doc = "Token type for ExtIntNMI"]
pub enum EINMI {}
impl Sealed for EINMI {}

//==============================================================================
// Registers
//==============================================================================

// Private struct that provides access to the EIC registers from
// the ExtInt types. We must be careful about memory safety here
/// TODO
struct Registers<E: EINum> {
    ei_num: PhantomData<E>,
}

struct NmiRegisters {
    ei_num: PhantomData<NoneT>,
}

// Special for NMI
impl NmiRegisters {
    /// TODO
    unsafe fn new() -> Self {
        NmiRegisters {
            ei_num: PhantomData,
        }
    }
    /// TODO
    fn eic(&self) -> &RegisterBlock {
        unsafe { &*crate::pac::EIC::ptr() }
    }
    /// TODO
    fn set_filter_mode(&self, usefilter: bool) {
        self.eic().nmictrl.write(|w| w.nmifilten().bit(usefilter));
    }
    /// TODO
    fn set_async_mode(&self, useasync: bool) {
        self.eic().nmictrl.write(|w| w.nmiasynch().bit(useasync));
    }
    /// TODO
    fn clear_interrupt_status(&self) {
        self.eic().nmiflag.write(|w| w.nmi().set_bit());
    }
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

    /// TODO
    fn enable_interrupt(&self) {
        self.eic()
            .intenset
            .write(|w| unsafe { w.bits(E::MASK as u32) });
    }

    /// TODO
    fn disable_interrupt(&self) {
        self.eic()
            .intenclr
            .write(|w| unsafe { w.bits(E::MASK as u32) });
    }

    /// TODO
    fn get_interrupt_status(&self) -> bool {
        let state = self.eic().intflag.read().extint().bits();
        (state & E::MASK) != 0
    }
    fn clear_interrupt_status(&self) {
        self.eic()
            .intflag
            .write(|w| unsafe { w.bits(E::MASK as u32) });
    }

    // Can't add methods that access registers that share state
    // between different ExtInt. Those most be added to EIController
}

//==============================================================================
// Register representations
//==============================================================================

bitfield::bitfield! {
    /// Register description for EIC Control register
    ///
    /// Control consists of two registers, part 1 and 2
    /// both sharing the same layout.
    pub struct EIConfigReg(u32);
    impl Debug;
    u8;
    get_sense0, set_sense0: 2, 0;
    get_filten0, set_filten0: 3, 3;
    get_sense1, set_sense1: 6, 4;
    get_filten1, set_filten1: 7, 7;
    get_sense2, set_sense2: 10, 8;
    get_filten2, set_filten2: 11, 11;
    get_sense3, set_sense3: 15, 12;
    get_filten3, set_filten3: 15, 15;
    get_sense4, set_sense4: 18, 16;
    get_filten4, set_filten4: 19, 19;
    get_sense5, set_sense5: 22, 20;
    get_filten5, set_filten5: 23, 23;
    get_sense6, set_sense6: 26, 24;
    get_filten6, set_filten6: 27, 27;
    get_sense7, set_sense7: 30, 28;
    get_filten7, set_filten7: 31, 31;
}

bitfield::bitfield! {
    /// Register description for EIC Control register
    ///
    /// Control consists of two registers, part 1 and 2
    /// both sharing the same layout.
    pub struct EIAsyncReg(u16);
    impl Debug;
    u8;
    get_asynch0, set_asynch0: 0, 0;
    get_asynch1, set_asynch1: 1, 1;
    get_asynch2, set_asynch2: 2, 2;
    get_asynch3, set_asynch3: 3, 3;
    get_asynch4, set_asynch4: 4, 4;
    get_asynch5, set_asynch5: 5, 5;
    get_asynch6, set_asynch6: 6, 6;
    get_asynch7, set_asynch7: 7, 7;
    get_asynch8, set_asynch8: 8, 8;
    get_asynch9, set_asynch9: 9, 9;
    get_asynch10, set_asynch10: 10, 10;
    get_asynch11, set_asynch11: 11, 11;
    get_asynch12, set_asynch12: 12, 12;
    get_asynch13, set_asynch13: 13, 13;
    get_asynch14, set_asynch14: 14, 14;
    get_asynch15, set_asynch15: 15, 15;
}

//==============================================================================
// Set sense mode helper macro
//==============================================================================
#[macro_export]
macro_rules! set_sense_ext {
    // For all regular ExtInt
    ($self:ident, $I:ident, $extint:ident, $sense:ident) => {
        paste! {
            #[doc = "Set Input [`Sense`] to "$sense]
            pub fn [<set_sense_$sense:lower>]<AK2, N>(
                self,
                // Used to enforce having access to EIController
                eic: &Enabled<EIController<AK2, Configurable>, N>,
                ) -> $extint<I, C, AM, AK, [<Sense$sense>]>
                where
                    AK2: AnyClock,
                    N: Counter,
            {
                eic.set_sense_mode::<I::EINum>(Sense::$sense);

                $extint {
                    token: self.token,
                    pin: self.pin,
                    mode: PhantomData,
                    clockmode: PhantomData,
                    sensemode: PhantomData,
                }
            }
        }
    };
}
#[macro_export]
macro_rules! set_sense_ext_nmi {
    // For NMI case
    ($self:ident, $extint:ident, $sense:ident) => {
        paste! {
            #[doc = "Set Input [`Sense`] to "$sense]
            pub fn [<set_sense_$sense:lower>]<AK2, N>(
                self,
                // Used to enforce having access to EIController
                eic: &Enabled<EIController<AK2, Configurable>, N>,
                ) -> $extint<I, C, AM, AK, [<Sense$sense>]>
                where
                    AK2: AnyClock,
                    N: Counter,
            {
                eic.set_sense_mode_nmi(Sense::$sense);

                $extint {
                    token: self.token,
                    pin: self.pin,
                    mode: PhantomData,
                    clockmode: PhantomData,
                    sensemode: PhantomData,
                }
            }
        }
    };
    // For NMI Async case
    (Async $self:ident, $extint:ident, $sense:ident) => {
        paste! {
            #[doc = "Set Input [`Sense`] to "$sense]
            pub fn [<set_sense_$sense:lower>]<AK, N>(
                self,
                // Used to enforce having access to EIController
                eic: &Enabled<EIController<AK, Configurable>, N>,
                ) -> $extint<I, C, AsyncOnly, WithoutClock, [<Sense$sense>]>
                where
                    AK: AnyClock,
                    N: Counter,
            {
                eic.set_sense_mode_nmi(Sense::$sense);

                $extint {
                    token: self.token,
                    pin: self.pin,
                    mode: PhantomData,
                    clockmode: PhantomData,
                    sensemode: PhantomData,
                }
            }
        }
    };
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
pub struct NmiToken {
    regs: NmiRegisters,
}

impl NmiToken {
    // Unsafe because you must make sure each NmiToken is a singleton
    /// TODO
    unsafe fn new() -> Self {
        NmiToken {
            regs: NmiRegisters::new(),
        }
    }
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
    paste!{
        /// Tokens for each External Interrupt
        pub struct Tokens {
            #(
                #[allow(dead_code)]
                #[doc = "Token for EI" N]
                pub ext_int_#N: Token<EI#N>,
            )*
            #[allow(dead_code)]
            #[doc = "Token for EINMI"]
            pub ext_int_nmi: NmiToken,
        }

        impl Tokens {
            // Unsafe because you must make sure each Token is a singleton
            /// TODO
            unsafe fn new() -> Self {
                Tokens {
                    #(
                        ext_int_#N: Token::new(),
                    )*
                    ext_int_nmi: NmiToken::new(),
                }
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
pub struct WithoutClock {}
impl Sealed for WithoutClock {}
impl Clock for WithoutClock {}

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

/// Type class for all possible [`Clock`] types
///
/// This trait uses the [`AnyKind`] trait pattern to create a [type class] for
/// [`Clock`] types. See the `AnyKind` documentation for more details on the
/// pattern.
///
/// [`AnyKind`]: crate::typelevel#anykind-trait-pattern
/// [type class]: crate::typelevel#type-classes
pub trait AnyClock: Sealed + Is<Type = SpecificClock<Self>> {
    type Mode: Clock;
    type ClockSource: EIClkSrc;
}

/// TODO
pub type SpecificClock<K> = <K as AnyClock>::Mode;

impl AnyClock for WithoutClock {
    /// TODO
    type Mode = WithoutClock;
    type ClockSource = NoneT;
}

impl<CS> AnyClock for WithClock<CS>
where
    CS: EIClkSrc,
{
    type Mode = WithClock<CS>;
    type ClockSource = CS;
}

impl AsRef<Self> for WithoutClock {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<Self> for WithoutClock {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<CS: EIClkSrc> AsRef<Self> for WithClock<CS> {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<CS: EIClkSrc> AsMut<Self> for WithClock<CS> {
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

impl EIClkSrc for NoneT {
    /// TODO
    /// This is the default value at reset
    /// This is a workaround to be able to extract ClockSource
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

// ExtInt Non-Maskable-Interrupt (NMI)
pub trait NmiEI: PinId {}
impl NmiEI for gpio::PA08 {}

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
