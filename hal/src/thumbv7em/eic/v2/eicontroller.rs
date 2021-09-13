use core::marker::PhantomData;

use typenum::U0;

use crate::clock::types::{Counter, Decrement, Enabled, Increment};
use crate::gpio::v2::{Interrupt, InterruptConfig, Pin};

use crate::eic::v2::*;

use super::extint::*;

//==============================================================================
// EIController
//==============================================================================

// Struct to represent the external interrupt controller
// You need exclusive access to this to set registers that
// share multiple pins, like the Sense configuration register
/// TODO
pub struct EIController<M: Clock>
where
    M: Clock,
{
    eic: crate::pac::EIC,
    #[allow(dead_code)]
    mode: M,
}

impl<K> EIController<WithClock<K>>
where
    K: EIClkSrc + Increment,
{
    /// Create an EIC Controller with a clock source
    ///
    /// This allows for full EIC functionality
    ///
    /// Safety
    ///
    /// Safe because you trade a singleton PAC struct for new singletons
    pub fn new(eic: crate::pac::EIC, clock: K) -> (Enabled<Self, U0>, Tokens, K::Inc) {
        // Software reset the EIC controller on creation
        eic.ctrla.modify(|_, w| w.swrst().set_bit());
        while eic.syncbusy.read().swrst().bit_is_set() {
            cortex_m::asm::nop();
        }

        // Set CKSEL to match the clock resource provided
        eic.ctrla.modify(|_, w| w.cksel().variant(K::CKSEL));

        unsafe {
            (
                Enabled::new(Self {
                    eic,
                    mode: WithClock { clock: PhantomData },
                }),
                Tokens::new(),
                clock.inc(),
            )
        }
    }
}

impl EIController<NoClock> {
    /// Create an EIC Controller without a clock source
    ///
    /// This limits the EIC functionality
    ///
    /// Safety
    ///
    /// Safe because you trade a singleton PAC struct for new singletons
    pub fn new_only_async(eic: crate::pac::EIC) -> (Enabled<Self, U0>, Tokens) {
        // Software reset the EIC controller on creation
        eic.ctrla.modify(|_, w| w.swrst().set_bit());
        while eic.syncbusy.read().swrst().bit_is_set() {
            cortex_m::asm::nop();
        }

        // Setup mode to async for all channels
        eic.asynch.write(|w| unsafe { w.bits(0xFFFF) });

        // Does not use or need any external clock, `CKSEL` is ignored

        unsafe {
            (
                Enabled::new(Self {
                    eic,
                    mode: NoClock {},
                }),
                Tokens::new(),
            )
        }
    }
}

impl<M> Enabled<EIController<M>, U0>
where
    M: Clock,
{
    /// Software reset needs to be synchronised
    fn syncbusy_swrst(&self) {
        while self.0.eic.syncbusy.read().swrst().bit_is_set() {
            cortex_m::asm::nop();
        }
    }
}

impl<M, N> Enabled<EIController<M>, N>
where
    M: Clock,
    N: Counter,
{
    pub(super) fn set_sense_mode<E: EINum>(&mut self, sense: Sense) {
        let index = match E::NUM {
            0..=7 => 0,
            _ => 1,
        };
        self.0.eic.config[index].write(|w| unsafe { w.bits(E::SENSE & sense as u32) });
    }

    /// Enabling the EIC controller needs to be synchronised
    fn syncbusy_finalize(&self) {
        while self.0.eic.syncbusy.read().enable().bit_is_set() {
            cortex_m::asm::nop();
        }
    }
    /// Start EIC controller by writing the enable bit
    pub fn finalize(&self) {
        self.0.eic.ctrla.modify(|_, w| w.enable().set_bit());
        self.syncbusy_finalize();
    }
}

impl<K> Enabled<EIController<WithClock<K>>, U0>
where
    K: EIClkSrc + Decrement,
{
    /// Disable and destroy the EIC controller
    pub fn destroy<S>(self, _tokens: Tokens, clock: K) -> (crate::pac::EIC, K::Dec)
    where
        S: EIClkSrc + Decrement,
    {
        (self.0.eic, clock.dec())
    }

    /// Softare reset the EIC controller
    pub fn swrst(&self) {
        self.0.eic.ctrla.modify(|_, w| w.swrst().set_bit());
        self.syncbusy_swrst();

        // Set CKSEL to match the clock resource provided
        self.0.eic.ctrla.modify(|_, w| w.cksel().variant(K::CKSEL));
    }
}

impl Enabled<EIController<NoClock>, U0> {
    /// Disable and destroy the EIC controller
    pub fn destroy(self, _tokens: Tokens) -> crate::pac::EIC {
        self.0.eic
    }

    /// Softare reset the EIC controller
    pub fn swrst(&self) {
        self.0.eic.ctrla.modify(|_, w| w.swrst().set_bit());
        self.syncbusy_swrst();

        // Setup mode to async for all channels
        self.0.eic.asynch.write(|w| unsafe { w.bits(0xFFFF) });
    }
}

impl<K, N> Enabled<EIController<WithClock<K>>, N>
where
    K: EIClkSrc,
    N: Counter,
{
    /// TODO
    pub fn new_sync<I, C>(
        &self,
        token: Token<I::EINum>,
        pin: Pin<I, Interrupt<C>>,
    ) -> ExtInt<I, C, WithClock<K>, SenseNone>
    where
        I: GetEINum,
        C: InterruptConfig,
    {
        ExtInt::new_sync(token, pin)
    }

    // Private function that should be accessed through the ExtInt
    // Could pass the MASK directly instead of making this function
    // generic over the EINum. Either way is fine.
    /// TODO
    pub(super) fn enable_debouncer<E: EINum>(&mut self) {
        self.0.eic.debouncen.modify(|r, w| unsafe {
            let bits = r.debouncen().bits();
            w.debouncen().bits(bits | E::MASK)
        });
    }

    pub(super) fn set_debouncer_settings<E: EINum>(&mut self, settings: &DebouncerSettings) {
        self.0.eic.dprescaler.write({
            |w| {
                w.tickon()
                    .variant(settings.tickon)
                    .prescaler0()
                    .variant(settings.prescaler0)
                    .states0()
                    .variant(settings.states0)
                    .prescaler1()
                    .variant(settings.prescaler1)
                    .states1()
                    .variant(settings.states1)
            }
        });
    }

    // Private function that should be accessed through the ExtInt
    /// TODO
    pub(super) fn enable_filtering<E: EINum>(&mut self) {
        let index = match E::NUM {
            0..=7 => 0,
            _ => 1,
        };
        self.0.eic.config[index].write(|w| unsafe { w.bits(E::FILTEN) });
    }
}

impl<M, N> Enabled<EIController<M>, N>
where
    M: Clock,
    N: Counter,
{
    /// TODO
    pub fn new_async<I, C>(
        &self,
        token: Token<I::EINum>,
        pin: Pin<I, Interrupt<C>>,
    ) -> ExtInt<I, C, NoClock, SenseNone>
    where
        I: GetEINum,
        C: InterruptConfig,
    {
        ExtInt::new_async(token, pin)
    }
}
