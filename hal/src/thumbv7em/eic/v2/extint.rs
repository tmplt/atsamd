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
// ExtInt
//==============================================================================

// The pin-level struct
// It must be generic over PinId, Interrupt PinMode configuration
// (i.e. Floating, PullUp, or PullDown)
/// TODO
pub struct ExtInt<I, C, AK, S>
where
    I: GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    S: SenseMode,
{
    regs: Registers<I::EINum>,
    #[allow(dead_code)]
    pin: Pin<I, Interrupt<C>>,
    clockmode: PhantomData<AK>,
    sensemode: PhantomData<S>,
}

// Sealed for ExtInt
impl<I, C, AK, S> Sealed for ExtInt<I, C, AK, S>
where
    I: GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
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
    pub(crate) fn new_async(
        token: Token<I::EINum>,
        pin: Pin<I, Interrupt<C>>,
    ) -> AsyncExtInt<I, C, NoClock, SenseNone> {
        // #TODO
        // Configure the AsyncExtInt (e.g. set the Asynchronous Mode register)
        AsyncExtInt {
            extint: ExtInt {
                regs: token.regs,
                pin,
                clockmode: PhantomData,
                sensemode: PhantomData,
            },
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
                ) -> ExtInt<I, C, AK, [<Sense$sense>]>
                where
                    AK2: AnyClock,
                    N: Counter,
            {
                self.regs.set_sense_mode(Sense::$sense);

                ExtInt {
                    regs: self.regs,
                    pin: self.pin,
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
                ) -> [<$kind Int>]<I, C, AK, [<Sense$sense>]>
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
impl<I, C, AK, S> ExtInt<I, C, AK, S>
where
    I: GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    S: SenseMode,
{
    // Must have access to the EIController here

    // Do not need access to the EIController here

    /// TODO
    pub fn set_sense_mode<S2>(self, sense: Sense) -> ExtInt<I, C, AK, S2>
    where
        S2: SenseMode,
    {
        self.regs.set_sense_mode(sense);

        ExtInt {
            regs: self.regs,
            pin: self.pin,
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

impl<I, C, CS, AK, S> ExtInt<I, C, AK, S>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    AK: AnyClock<Mode = WithClock<CS>>,
    S: DebounceMode,
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
    ) -> DebouncedExtInt<I, C, AK, S>
    where
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.enable_debouncing::<I::EINum>();
        DebouncedExtInt { extint: self }
    }
}

impl<I, C, CS, AK, S> ExtInt<I, C, AK, S>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    AK: AnyClock<Mode = WithClock<CS>>,
    S: SenseMode,
{
    // Methods related to filtering and debouncing go here,
    // since they require a clock

    // Must have access to the EIController here
    /// TODO
    pub fn enable_filtering<N>(
        self,
        eic: &mut Enabled<EIController<WithClock<CS>, Configurable>, N>,
    ) -> FilteredExtInt<I, C, AK, S>
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
    type Clock: AnyClock;
    /// TODO
    type SenseMode: SenseMode;
}

impl<I, C, K, S> AnyExtInt for ExtInt<I, C, K, S>
where
    I: EINum + GetEINum,
    C: InterruptConfig,
    K: AnyClock,
    S: SenseMode,
{
    /// TODO
    type Num = I;
    /// TODO
    type Pin = C;
    /// TODO
    type Clock = K;
    /// TODO
    type SenseMode = S;
}
pub type SpecificExtInt<E> = ExtInt<
    <E as AnyExtInt>::Num,
    <E as AnyExtInt>::Pin,
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
