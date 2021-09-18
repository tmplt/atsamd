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
    CS: EIClkSrc,
{
    /// Create initial synchronous ExtInt
    /// TODO
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
    /// TODO
    ///
    /// FIXME Requires extensive type annotations
    pub fn set_sense_mode<AK2, S2, N>(
        self,
        // Used to enforce having access to EIController
        eic: &Enabled<EIController<AK2, Configurable>, N>,
        sense: Sense,
    ) -> ExtInt<I, C, AM, AK, S2>
    where
        AK2: AnyClock,
        S2: AnySenseMode,
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(sense);

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

    /// Read the pin state of the ExtInt
    /// TODO
    pub fn pin_state(&self) -> bool {
        self.token.regs.pin_state()
    }

    /// TODO
    pub fn enable_interrupt(&self) {
        self.token.regs.enable_interrupt();
    }

    /// TODO
    pub fn disable_interrupt(&self) {
        self.token.regs.disable_interrupt();
    }

    /// TODO
    pub fn get_interrupt_status(&self) -> bool {
        self.token.regs.get_interrupt_status()
    }
    /// TODO
    pub fn clear_interrupt_status(&self) {
        self.token.regs.clear_interrupt_status();
    }
    /// TODO
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
    /// TODO
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
    I: GetEINum,
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
