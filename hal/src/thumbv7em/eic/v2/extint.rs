use core::marker::PhantomData;
use core::mem::transmute;

use crate::eic::v2::*;
use crate::gpio::v2::{Interrupt, InterruptConfig, Pin};
use crate::typelevel::{Is, Sealed};

pub mod asynconly;
pub mod debounced;
pub mod filtered;
pub mod nmi;

pub use asynconly::*;
pub use debounced::*;
pub use filtered::*;
pub use nmi::*;

// Macro for setting sense
use crate::set_sense_ext;

//==============================================================================
// Mode
//==============================================================================

/// ExtInt Detection Mode
pub enum EIMode {
    Normal = 0,
    AsyncOnly,
    Filtered,
    FilteredAsync,
    Debounced,
    DebouncedAsync,
}

///  ExtInt Detection Mode
pub trait Mode: Sealed {
    const MODE: EIMode;
}

/// Normal Detection mode
pub enum Normal {}
/// Asynchronous Detection Mode
///
/// Level detection and operating without
/// external clock
pub enum AsyncOnly {}
/// Filtered mode
pub enum Filtered {}
/// Filtered + Async flag set
pub enum FilteredAsync {}
/// Debounced mode
pub enum Debounced {}
/// Debounced mode + Async flag set
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
pub trait AnySenseMode: Sealed + Is<Type = SpecificSenseMode<Self>> {
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
// ExtInt
//==============================================================================

// The pin-level struct
// It must be generic over PinId, Interrupt PinMode configuration
// (i.e. Floating, PullUp, or PullDown)
/// External Interrupt
///
/// Each of the 16 EIC Pins are represented by this struct.
///
/// The exception is the NMI interrupt, see [`NmiExtInt`].
///
///
pub struct ExtInt<I, C, AM, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    AM: AnyMode,
    AK: AnyClock,
    AS: AnySenseMode,
{
    pub(super) token: Token<I::EINum>,
    pub(super) pin: Pin<I, Interrupt<C>>,
    mode: PhantomData<AM>,
    clockmode: PhantomData<AK>,
    sensemode: PhantomData<AS>,
}

// Sealed for ExtInt
impl<I, C, AM, AK, AS> Sealed for ExtInt<I, C, AM, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    AM: AnyMode,
    AK: AnyClock,
    AS: AnySenseMode,
{
}

impl<I, C, AM, CS> ExtInt<I, C, AM, WithClock<CS>, SenseNone>
where
    I: GetEINum,
    C: InterruptConfig,
    AM: AnyMode,
    CS: ClkSrc,
{
    /// Create initial synchronous ExtInt
    pub(crate) fn new_sync(token: Token<I::EINum>, pin: Pin<I, Interrupt<C>>) -> Self {
        ExtInt {
            token,
            pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}

// Methods for any state of ExtInt
impl<I, C, AM, AK, AS> ExtInt<I, C, AM, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    AM: AnyMode,
    AK: AnyClock,
    AS: AnySenseMode,
{
    /// Change sense mode
    ///
    /// Requires full type annotations
    /// But if using this as a RTIC resource
    /// that is already a requirement
    pub fn set_sense_mode<AK2, S, S2, N>(
        self,
        // Used to enforce having access to EIController
        eic: &Enabled<EIController<AK2, Configurable>, N>,
    ) -> ExtInt<I, C, AM, AK, S2>
    where
        AK2: AnyClock,
        S: SenseMode,
        S2: AnySenseMode<Mode = S>,
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(S::SENSE);

        ExtInt {
            token: self.token,
            pin: self.pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
    set_sense_ext! {self, I, ExtInt, None}
    set_sense_ext! {self, I, ExtInt, High}
    set_sense_ext! {self, I, ExtInt, Low}
    set_sense_ext! {self, I, ExtInt, Both}
    set_sense_ext! {self, I, ExtInt, Rise}
    set_sense_ext! {self, I, ExtInt, Fall}

    /// Enable Interrupt
    pub fn enable_interrupt(&self) {
        self.token.regs.enable_interrupt();
    }

    /// Disable Interrupt
    pub fn disable_interrupt(&self) {
        self.token.regs.disable_interrupt();
    }

    /// Check if interrupt has triggered
    pub fn get_interrupt_status(&self) -> bool {
        self.token.regs.get_interrupt_status()
    }
    /// Clear interrupt status
    pub fn clear_interrupt_status(&self) {
        self.token.regs.clear_interrupt_status();
    }
    /// Enable Event System Output (EVSYS)
    ///
    /// Note: This is not tracked in typestate
    pub fn enable_event_output<AK2, N>(
        &self,
        // Used to enforce having access to EIController
        eic: &Enabled<EIController<AK2, Configurable>, N>,
    ) where
        AK2: AnyClock,
        N: Counter,
    {
        eic.set_event_output::<I::EINum>(true);
    }
    /// Disable Event System Output (EVSYS)
    ///
    /// Note: This is not tracked in typestate
    pub fn disable_event_output<AK2, N>(
        &self,
        // Used to enforce having access to EIController
        eic: &Enabled<EIController<AK2, Configurable>, N>,
    ) where
        AK2: AnyClock,
        N: Counter,
    {
        eic.set_event_output::<I::EINum>(false);
    }
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
    type Num: GetEINum;
    /// Configuration of GPIO [`Pin`]
    type Pin: InterruptConfig;
    /// ExtInt operation mode [`Mode`]
    type Mode: AnyMode;
    /// Clock source [`Clock`]
    type Clock: AnyClock;
    /// ExtInt SenseMode [`Sense`]
    type SenseMode: AnySenseMode;
}

impl<I, C, AM, AK, AS> AnyExtInt for ExtInt<I, C, AM, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    AM: AnyMode,
    AK: AnyClock,
    AS: AnySenseMode,
{
    type Num = I;
    type Pin = C;
    type Mode = AM;
    type Clock = AK;
    type SenseMode = AS;
}

pub type SpecificExtInt<E> = ExtInt<
    <E as AnyExtInt>::Num,
    <E as AnyExtInt>::Pin,
    <E as AnyExtInt>::Mode,
    <E as AnyExtInt>::Clock,
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
