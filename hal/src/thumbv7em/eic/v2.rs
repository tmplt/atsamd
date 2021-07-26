use core::marker::PhantomData;

use seq_macro::seq;

use crate::clock::v2::osculp32k::OscUlp32k;
use crate::clock::v2::pclk::{Eic, Pclk, PclkSourceMarker};
use crate::clock::v2::rtc::{Active1k, Active32k, Output1k, Output32k};
use crate::gpio::v2::{self as gpio, Alternate, AnyPin, Interrupt, InterruptConfig, Pin, PinId};
use crate::pac::eic::{ctrla::CKSEL_A, RegisterBlock};
use crate::typelevel::{NoneT, Sealed};

//==============================================================================
// Sense
//==============================================================================

// Need a custom type, because the PAC has 8 identical copies
// of the same enum. There's probably a way to patch the PAC
pub enum Sense {
    None = 0,
    Rise,
    Fall,
    Both,
    High,
    Low,
}

//==============================================================================
// EINum
//==============================================================================

// Type-level enum for the ExtInt number
// Each PinId is mapped to one and only one
pub trait EINum: Sealed {
    const NUM: u8;
    const MASK: u16 = 1 << Self::NUM;
    // Possibly other constants
}

seq!(N in 00..16 {
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
struct Registers<E: EINum> {
    ei_num: PhantomData<E>,
}

impl<E: EINum> Registers<E> {
    // Unsafe because you must make there is only one copy
    // of Registers for each unique E
    unsafe fn new() -> Self {
        Registers {
            ei_num: PhantomData,
        }
    }

    fn eic(&self) -> &RegisterBlock {
        unsafe { &*crate::pac::EIC::ptr() }
    }

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
pub struct Token<E: EINum> {
    regs: Registers<E>,
}

impl<E: EINum> Token<E> {
    // Unsafe because you must make sure each Token is a singleton
    unsafe fn new() -> Self {
        Token {
            regs: Registers::new(),
        }
    }
}

seq!(N in 00..16 {
    pub struct Tokens {
        #(
            #[allow(dead_code)]
            pub ext_int_#N: Token<EI#N>,
        )*
    }

    impl Tokens {
        // Unsafe because you must make sure each Token is a singleton
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
// DetectionMode
//==============================================================================

// Synchronous vs. asynchronous detection
pub trait DetectionMode: Sealed {}

pub struct AsyncMode;
impl Sealed for AsyncMode {}
impl DetectionMode for AsyncMode {}

// When in SyncMode, we have to store a clock resource
pub struct SyncMode<C: EIClkSrc> {
    #[allow(dead_code)]
    clock: C,
}
impl<C: EIClkSrc> Sealed for SyncMode<C> {}
impl<C: EIClkSrc> DetectionMode for SyncMode<C> {}

// EI clock source for synchronous detection modes
// TODO should this need Sealed?
pub trait EIClkSrc {
    const CKSEL: CKSEL_A;
}

// Peripheral channel clock, routed from a GCLK
impl<T: PclkSourceMarker> EIClkSrc for Pclk<Eic, T> {
    const CKSEL: CKSEL_A = CKSEL_A::CLK_GCLK;
}

// Ultra-low power oscillator can be used instead
impl<Y: Output1k> EIClkSrc for OscUlp32k<Active32k, Y> {
    const CKSEL: CKSEL_A = CKSEL_A::CLK_ULP32K;
}

//==============================================================================
// ExtInt
//==============================================================================

// The pin-level struct
// It must be generic over PinId, Interrupt PinMode configuration
// (i.e. Floating, PullUp, or PullDown), and DetectionMode
pub struct ExtInt<I, C, M>
where
    I: GetEINum,
    C: InterruptConfig,
    M: DetectionMode,
{
    regs: Registers<I::EINum>,
    pin: Pin<I, Interrupt<C>>,
    mode: M,
}

impl<I, C> ExtInt<I, C, AsyncMode>
where
    I: GetEINum,
    C: InterruptConfig,
{
    pub fn new_async(token: Token<I::EINum>, pin: Pin<I, Interrupt<C>>) -> Self {
        // Configure the ExtInt (e.g. set the Asynchronous Mode register)
        ExtInt {
            regs: token.regs,
            pin: pin.into(),
            mode: AsyncMode,
        }
    }

    // Do not need access to the EIController here
    pub fn pin_state(&self) -> bool {
        self.regs.pin_state()
    }
}

impl<I, C, K> ExtInt<I, C, SyncMode<K>>
where
    I: GetEINum,
    C: InterruptConfig,
    K: EIClkSrc,
{
    pub fn new_sync(token: Token<I::EINum>, pin: Pin<I, Interrupt<C>>, clock: K) -> Self {
        // Configure the ExtInt (e.g. set the Asynchronous Mode register)
        ExtInt {
            regs: token.regs,
            pin: pin.into(),
            mode: SyncMode { clock },
        }
    }

    // Methods related to filtering and debouncing go here,
    // since they require a clock

    // Must have access to the EIController here
    pub fn enable_debouncer(&mut self, eic: &mut EIController<K>) {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.enable_debouncer::<I::EINum>();
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

//==============================================================================
// OptionalEIClock
//==============================================================================

/// Type-level equivalent of `Option<EIClock>`
///
/// See the [`OptionalKind`] documentation for more details on the pattern.
///
/// [`OptionalKind`]: crate::typelevel#optionalkind-trait-pattern
/// TODO Sealed?
pub trait OptionalEIClock {
    type EIClock: OptionalEIClock;
}

impl OptionalEIClock for NoneT {
    type EIClock = NoneT;
}

impl<C: EIClkSrc> OptionalEIClock for C {
    type EIClock = C;
}

//==============================================================================
// EIController
//==============================================================================

// Struct to represent the external interrupt controller
// You need exclusive access to this to set registers that
// share multiple pins, like the Sense configuration register
pub struct EIController<C: OptionalEIClock> {
    eic: crate::pac::EIC,
    optional_clk: C,
}

impl<C> EIController<C>
where
    C: OptionalEIClock,
{
    // Safe because you trade a singleton PAC struct for new singletons
    pub fn new(eic: crate::pac::EIC, optional_clk: C) -> (Self, Tokens) {
        unsafe { (Self { eic, optional_clk}, Tokens::new()) }
    }

    // Private function that should be accessed through the ExtInt
    // Could pass the MASK directly instead of making this function
    // generic over the EINum. Either way is fine.
    fn enable_debouncer<E: EINum>(&mut self) {
        self.eic.debouncen.modify(|r, w| unsafe {
            let bits = r.debouncen().bits();
            w.debouncen().bits(bits | E::MASK)
        });
    }
}

//==============================================================================
// GetEINum
//==============================================================================

// Type-level function to get the EINum from a PinId
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
