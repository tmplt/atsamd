use core::marker::PhantomData;

use paste::paste;
use seq_macro::seq;

use crate::gpio::v2::{self as gpio, PinId};
use crate::pac::eic::{ctrla::CKSEL_A, dprescaler::*, RegisterBlock};
use crate::typelevel::{Is, NoneT, Sealed};

pub mod eicontroller;
pub mod extint;
pub mod types;

pub use crate::eic::v2::eicontroller::*;
pub use crate::eic::v2::extint::*;
pub use types::{Counter, Enabled};

//==============================================================================
// EINum
//==============================================================================

/// Type-level enum for the ExtInt number
/// Each PinId is mapped to one and only one
pub trait EINum: Sealed {
    /// Unique identifier
    const NUM: u8;
    /// Filter and sense configuration is split in two parts,
    /// `EIC->CONFIG0` and `EIC->CONFIG1`.
    ///
    /// OFFSET is used to index into the abstraction provided
    /// by svd2rust.
    ///
    /// OFFSET = 0 holds first 0..7 `FILTENx` and `SENSEx`
    ///
    /// OFFSET = 1 holds remaining 8..15 `FILTENx` and `SENSEx`
    const OFFSET: u8 = match Self::NUM {
        0..=7 => 0,
        8..=255 => 1,
    };
    /// Bitmask associated with NUM
    const MASK: u16 = 1 << Self::NUM;
    // Filten described by arithmetic series
    // 3+(n)*4
    /// Offset into `EIC->CONFIG` for `FILTEN`
    const FILTEN: u8 = 3 + Self::NUM * 4;
    // Start of Sense slice described by the series
    // (n)*4. Bitmask for Sense is 0b111,
    // meaning MSB of sense is 2 "steps" from LSB
    /// LSB of `SENSEx` for this `NUM`
    const SENSELSB: u8 = Self::NUM * 4;
    /// MSB of `SENSEx` for this `NUM`
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

/// Private struct that provides access to the EIC registers from
/// the ExtInt types. We must be careful about memory safety here
struct Registers<E: EINum> {
    ei_num: PhantomData<E>,
}

/// Private struct that provides access to the EIC registers from
/// the ExtIntNMI type
struct NmiRegisters {
    ei_nmi: PhantomData<NoneT>,
}

// Special for NMI
impl NmiRegisters {
    /// Create a new register associated with
    /// Non-Maskable External Interrupt (NmiExtInt)
    ///
    /// # Safety
    /// Unsafe because you must make there is only one copy
    /// of Registers for each unique E
    unsafe fn new() -> Self {
        NmiRegisters {
            ei_nmi: PhantomData,
        }
    }
    /// Direct reference to EIC [`RegisterBlock`]
    fn eic(&self) -> &RegisterBlock {
        unsafe { &*crate::pac::EIC::ptr() }
    }
    /// Enable or disable filtering mode
    ///
    /// Requires clocking via `GCLK_EIC` or `CLK_ULP32K`
    fn set_filter_mode(&self, usefilter: bool) {
        self.eic().nmictrl.write(|w| w.nmifilten().bit(usefilter));
    }
    /// Control `Async` mode of [`NmiExtInt`]
    ///
    /// ## Edge Detection mode
    ///
    /// ### Async mode on
    ///
    /// No external clock required, sets `NMIFLAG` directly
    ///
    /// Available in `Sleep` and `Standby` sleep modes
    ///
    /// ### Async mode off
    ///
    /// External clock required, sets `NMIFLAG` when the last
    /// sampled state differs from the previous state
    ///
    /// ##
    fn set_async_mode(&self, useasync: bool) {
        self.eic().nmictrl.write(|w| w.nmiasynch().bit(useasync));
    }
    /// Clear the interrupt
    fn clear_interrupt_status(&self) {
        self.eic().nmiflag.write(|w| w.nmi().set_bit());
    }
}

impl<E: EINum> Registers<E> {
    /// Create a new register associated with External Interrupt (ExtInt)
    ///
    /// # Safety
    /// Unsafe because you must make there is only one copy
    /// of Registers for each unique E
    unsafe fn new() -> Self {
        Registers {
            ei_num: PhantomData,
        }
    }

    /// Direct reference to EIC [`RegisterBlock`]
    fn eic(&self) -> &RegisterBlock {
        unsafe { &*crate::pac::EIC::ptr() }
    }

    /// EIC Pinstate return the state of the debounced external interrupt
    fn pin_state(&self) -> bool {
        let state = self.eic().pinstate.read().pinstate().bits();
        (state & E::MASK) != 0
    }

    /// Enable the interrupt of this ExtInt
    fn enable_interrupt(&self) {
        self.eic()
            .intenset
            .write(|w| unsafe { w.bits(E::MASK as u32) });
    }

    /// Disable the interrupt of this ExtInt
    fn disable_interrupt(&self) {
        self.eic()
            .intenclr
            .write(|w| unsafe { w.bits(E::MASK as u32) });
    }

    /// Read if interrupt has triggered
    ///
    /// Returning true indicates that ExtInt triggered with current settings
    /// If interrupts were enabled an interrupt would also have been triggered
    fn get_interrupt_status(&self) -> bool {
        let state = self.eic().intflag.read().extint().bits();
        (state & E::MASK) != 0
    }

    /// Clear the interrupt status for this ExtInt
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
    /// Register description for multiple EIC registers
    ///
    /// * EVCTRL
    /// * INTENCLR
    /// * INTENSET
    /// * INTFLAG
    /// * ASYNCH
    /// * DEBOUNCEN
    /// * PINSTATE
    ///
    /// Each bit from 0 to 16 corresponds directly to
    /// [`EINum::NUM`]
    pub struct EIReg(u16);
    impl Debug;
    u8;
    get_0, set_0: 0, 0;
    get_1, set_1: 1, 1;
    get_2, set_2: 2, 2;
    get_3, set_3: 3, 3;
    get_4, set_4: 4, 4;
    get_5, set_5: 5, 5;
    get_6, set_6: 6, 6;
    get_7, set_7: 7, 7;
    get_8, set_8: 8, 8;
    get_9, set_9: 9, 9;
    get_10, set_10: 10, 10;
    get_11, set_11: 11, 11;
    get_12, set_12: 12, 12;
    get_13, set_13: 13, 13;
    get_14, set_14: 14, 14;
    get_15, set_15: 15, 15;
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
                ) -> $extint<I, C, AM, AK2, [<Sense$sense>]>
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
            pub fn [<set_sense_$sense:lower>]<AK2, N>(
                self,
                // Used to enforce having access to EIController
                eic: &Enabled<EIController<AK2, Configurable>, N>,
                ) -> $extint<I, C, AsyncOnly, WithoutClock, [<Sense$sense>]>
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
}

//==============================================================================
// Token
//==============================================================================

// Singleton token structs
// We need to create exactly 16 of these at boot.
// A token will be consumed when creating an ExtInt.
// This will prevent multiple pins from using the same interrupt
/// Token struct, each token associated with [`EINum`] ensures
/// that only 16 ExtInts can be created
pub struct Token<E: EINum> {
    regs: Registers<E>,
}

impl<E: EINum> Token<E> {
    // Unsafe because you must make sure each Token is a singleton
    /// Create a new token
    ///
    /// # Safety
    ///
    /// Each Token must be a singleton
    unsafe fn new() -> Self {
        Token {
            regs: Registers::new(),
        }
    }
}

/// NmiToken struct, this token associated with the Non-Maskable Interrupt (NMI)
pub struct NmiToken {
    regs: NmiRegisters,
}

impl NmiToken {
    // Unsafe because you must make sure each NmiToken is a singleton
    /// Create a new token
    ///
    /// # Safety
    ///
    /// Each NmiToken must be a singleton
    unsafe fn new() -> Self {
        NmiToken {
            regs: NmiRegisters::new(),
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
            /// Create 16 tokens, one for each ExtInt and one for NmiExtInt.
            ///
            /// # Safety
            ///
            /// Each token must be a singleton
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
