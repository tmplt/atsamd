use core::marker::PhantomData;

use crate::gpio::v2::{Interrupt, InterruptConfig, Pin};

use crate::typelevel::{Is, Sealed};

use crate::eic::v2::*;

use core::mem::transmute;

/*
pub mod asynconly;
pub mod debounced;
pub mod filtered;

pub use asynconly::*;
pub use debounced::*;
pub use filtered::*;
*/

//==============================================================================
// Mode
//==============================================================================

/// Detection Mode
/// TODO
pub enum ExtMode {
    Normal = 0,
    AsyncOnly,
    Filtered,
    Debounced,
}

/// TODO
pub trait Mode: Sealed {
    const MODE: ExtMode;
}

/// TODO
pub enum Normal {}
/// TODO
pub enum AsyncOnly {}
/// TODO
pub enum Filtered {}
/// TODO
pub enum Debounced {}

impl Sealed for Normal {}
impl Sealed for AsyncOnly {}
impl Sealed for Filtered {}
impl Sealed for Debounced {}

impl Mode for Normal {
    const MODE: ExtMode = ExtMode::Normal;
}
impl Mode for AsyncOnly {
    const MODE: ExtMode = ExtMode::AsyncOnly;
}
impl Mode for Filtered {
    const MODE: ExtMode = ExtMode::Filtered;
}
impl Mode for Debounced {
    const MODE: ExtMode = ExtMode::Debounced;
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
any_mode!(Debounced);

//==============================================================================
// ExtInt
//==============================================================================

// The pin-level struct
// It must be generic over PinId, Interrupt PinMode configuration
// (i.e. Floating, PullUp, or PullDown)
/// TODO
pub struct ExtInt<I, C, AM, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    AM: AnyMode,
    AK: AnyClock,
    AS: AnySenseMode,
{
    regs: Registers<I::EINum>,
    #[allow(dead_code)]
    pin: Pin<I, Interrupt<C>>,
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
    CS: EIClkSrc,
{
    /// Create initial synchronous ExtInt
    /// TODO
    pub(crate) fn new_sync(token: Token<I::EINum>, pin: Pin<I, Interrupt<C>>) -> Self {
        ExtInt {
            regs: token.regs,
            pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}

impl<I, AM, C> ExtInt<I, C, AM, NoClock, SenseNone>
where
    I: GetEINum,
    C: InterruptConfig,
    AM: AnyMode<Mode = AsyncOnly>,
{
    /// Create initial asynchronous ExtInt
    /// TODO
    pub(crate) fn new_async(
        token: Token<I::EINum>,
        pin: Pin<I, Interrupt<C>>,
    ) -> ExtInt<I, C, AM, NoClock, SenseNone> {
        // #TODO
        // Configure the AsyncExtInt (e.g. set the Asynchronous Mode register)
        ExtInt {
                regs: token.regs,
                pin,
                mode: PhantomData,
                clockmode: PhantomData,
                sensemode: PhantomData,
        }
    }
}

macro_rules! set_sense_ext {
    ($self:ident, $sense:ident) => {
        paste! {
            /// TODO Set ExtInt Sense to [<$sense>]
            pub fn [<set_sense_$sense:lower>]<AK2, N>(
                self,
                // Used to enforce having access to EIController
                _eic: &mut Enabled<EIController<AK2, Configurable>, N>,
                ) -> ExtInt<I, C, AM, AK, [<Sense$sense>]>
                where
                    AK2: AnyClock,
                    N: Counter,
            {
                self.regs.set_sense_mode(Sense::$sense);

                ExtInt {
                    regs: self.regs,
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
macro_rules! set_sense_anyextint {
    ($self:ident, $kind:literal, $sense:ident) => {
        paste! {
            /// TODO Set [<$kind>]ExtInt Sense to [<$sense>]
            pub fn [<set_sense_$sense:lower>]<AK2, N>(
                self,
                // Used to enforce having access to EIController
                _eic: &mut Enabled<EIController<AK2, Configurable>, N>,
                ) -> [<$kind Int>]<I, C, AM, AK, [<Sense$sense>]>
                where
                    AK2: AnyClock,
                    N: Counter,
            {
                self.extint.regs.set_sense_mode(Sense::$sense);

                [<$kind Int>] {
                    extint: ExtInt {
                        regs: self.extint.regs,
                        pin: self.extint.pin,
                        clockmode: PhantomData,
                        sensemode: PhantomData,
                    }
                }
            }
        }
    };
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
    // Must have access to the EIController here

    // Do not need access to the EIController here

    /// TODO
    pub fn set_sense_mode<AK2, S2, N>(
        self,
        // Used to enforce having access to EIController
        _eic: &mut Enabled<EIController<AK2, Configurable>, N>,
        sense: Sense,
    ) -> ExtInt<I, C, AM, AK, S2>
    where
        AK2: AnyClock,
        S2: AnySenseMode,
        N: Counter,
    {
        self.regs.set_sense_mode(sense);

        ExtInt {
            regs: self.regs,
            pin: self.pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
    set_sense_ext! {self, None}
    set_sense_ext! {self, High}
    set_sense_ext! {self, Low}
    set_sense_ext! {self, Both}
    set_sense_ext! {self, Rise}
    set_sense_ext! {self, Fall}

    /// Read the pin state of the ExtInt
    /// TODO
    pub fn pin_state(&self) -> bool {
        self.regs.pin_state()
    }

    /// TODO
    pub fn enable_interrupt(&self) {
        self.regs.enable_interrupt();
    }

    /// TODO
    pub fn disable_interrupt(&self) {
        self.regs.disable_interrupt();
    }

    /// TODO
    pub fn get_interrupt_status(&self) -> bool {
        self.regs.get_interrupt_status()
    }
    /// TODO
    pub fn clear_interrupt_status(&self) {
        self.regs.clear_interrupt_status();
    }
}

impl<I, C, AM, CS, AK, S> ExtInt<I, C, AM, AK, S>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    AM: AnyMode<Mode = Debounced>,
    AK: AnyClock<Mode = WithClock<CS>>,
    S: DebounceMode + AnySenseMode,
{
    // Methods related to debouncing go here since they require a clock
    // and that SenseMode are one of: Rise, Fall or Both

    /// TODO
    ///
    /// ExtInt sense mode must be either [`Sense::Rise`], [`Sense::Fall`]
    /// or [`Sense::Both`]
    pub fn enable_debouncing<N>(
        self,
        eic: &mut Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> ExtInt<I, C, AM, AK, S>
    where
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.enable_debouncing::<I::EINum>();
        self
    }
}

impl<I, C, AM, CS, AK, AS> ExtInt<I, C, AM, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    AM: AnyMode<Mode = Filtered>,
    AK: AnyClock<Mode = WithClock<CS>>,
    AS: AnySenseMode,
{
    // Methods related to filtering and debouncing go here,
    // since they require a clock

    // Must have access to the EIController here
    /// TODO
    pub fn enable_filtering<N>(
        self,
        eic: &mut Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> ExtInt<I, C, AM, AK, AS>
    where
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.enable_filtering::<I::EINum>();
        self
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
    type Num: EINum + GetEINum;
    /// TODO
    type Pin: InterruptConfig;
    /// TODO
    type Mode: AnyMode;
    /// TODO
    type Clock: AnyClock;
    /// TODO
    type SenseMode: AnySenseMode;
}

impl<I, C, AM, AK, AS> AnyExtInt for ExtInt<I, C, AM, AK, AS>
where
    I: EINum + GetEINum,
    C: InterruptConfig,
    AM: AnyMode,
    AK: AnyClock,
    AS: AnySenseMode,
{
    /// TODO
    type Num = I;
    /// TODO
    type Pin = C;
    /// TODO
    type Mode = AM;
    /// TODO
    type Clock = AK;
    /// TODO
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

// AsyncExtInt

/*
impl<I, C, AM, AK, S> AnyExtInt for AsyncExtInt<I, C, AM, AK, S>
where
    I: EINum + GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    S: SenseMode,
{
    /// TODO
    type Num = I;
    /// TODO
    type Pin = C;
    /// TODO
    type Clock = AK;
    /// TODO
    type SenseMode = S;
}
*/

/*
impl<I, C, AM, AK, S> AsRef<AsyncExtInt<I, C, AM, AK, S>> for AsyncExtInt<I, C, AM, AK, S>
where
    I: EINum + GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    S: SenseMode,
{
    #[inline]
    fn as_ref(&self) -> &ExtInt<I, C, AM, AK, S> {
        unsafe { transmute(self) }
    }
}

impl<I, C, AM, AK, S> AsMut<AsyncExtInt<I, C, AM, AK, S>> for AsyncExtInt<I, C, AM, AK, S>
where
    I: EINum + GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    S: SenseMode,
{
    #[inline]
    fn as_mut(&mut self) -> &mut ExtInt<I, C, AM, AK, S> {
        unsafe { transmute(self) }
    }
}
impl<I, C, AM, AK, S> AsRef<ExtInt<I, C, AM, AK, S>> for AsyncExtInt<I, C, AM, AK, S>
where
    I: EINum + GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    S: SenseMode,
{
    #[inline]
    fn as_ref(&self) -> &ExtInt<I, C, AM, AK, S> {
        unsafe { transmute(self) }
    }
}

impl<I, C, AM, AK, S> AsMut<ExtInt<I, C, AM, AK, S>> for AsyncExtInt<I, C, AM, AK, S>
where
    I: EINum + GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    S: SenseMode,
{
    #[inline]
    fn as_mut(&mut self) -> &mut ExtInt<I, C, AM, AK, S> {
        unsafe { transmute(self) }
    }
}

impl<I, C, AM, AK, S> From<ExtInt<I, C, AM, AK, S>> for AsyncExtInt<I, C, AM, AK, S>
where
    I: EINum + GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    S: SenseMode,
{
    fn from(self) -> Self {
        ExtInt {
            regs: self.extint.regs,
            pin: self.extint.pin,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}
impl<I, C, AM, AK, S> From<AsyncExtInt<I, C, AM, AK, S>> for ExtInt<I, C, AM, AK, S>
where
    I: EINum + GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    S: SenseMode,
{
    fn from(self) -> Self {
        ExtInt {
            regs: self.extint.regs,
            pin: self.extint.pin,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}
*/
