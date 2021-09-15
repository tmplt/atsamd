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
pub struct EIController<AK>
where
    AK: AnyClock,
{
    eic: crate::pac::EIC,
    // Config consists of two 32-bit registers with the same layout
    // config.0 covers [`EInum`] 0 to 7, config.1 [`EInum`] 8 to 15
    config: (EIConfigReg, EIConfigReg),
    _clockmode: PhantomData<AK>,
}

impl<CS> EIController<WithClock<CS>>
where
    CS: EIClkSrc + Increment,
{
    /// Create an EIC Controller with a clock source
    ///
    /// This allows for full EIC functionality
    ///
    /// Safety
    ///
    /// Safe because you trade a singleton PAC struct for new singletons
    pub fn new(
        eic: crate::pac::EIC,
        clock: CS,
    ) -> (Enabled<EIController<WithClock<CS>>, U0>, Tokens, CS::Inc) {
        // Software reset the EIC controller on creation
        eic.ctrla.modify(|_, w| w.swrst().set_bit());
        while eic.syncbusy.read().swrst().bit_is_set() {
            cortex_m::asm::nop();
        }

        // Set CKSEL to match the clock resource provided
        eic.ctrla.modify(|_, w| w.cksel().variant(CS::CKSEL));

        // Create the config registers, matching reset state
        let config0 = EIConfigReg(0);
        let config1 = EIConfigReg(0);
        unsafe {
            (
                Enabled::new(Self {
                    eic,
                    config: (config0, config1),
                    _clockmode: PhantomData,
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
    pub fn new_only_async(eic: crate::pac::EIC) -> (Enabled<EIController<NoClock>, U0>, Tokens) {
        // Software reset the EIC controller on creation
        eic.ctrla.modify(|_, w| w.swrst().set_bit());
        while eic.syncbusy.read().swrst().bit_is_set() {
            cortex_m::asm::nop();
        }

        // Setup mode to async for all channels
        eic.asynch.write(|w| unsafe { w.bits(0xFFFF) });

        // Does not use or need any external clock, `CKSEL` is ignored
        // Create the config registers, matching reset state
        let config0 = EIConfigReg(0);
        let config1 = EIConfigReg(0);

        unsafe {
            (
                Enabled::new(Self {
                    eic,
                    config: (config0, config1),
                    _clockmode: PhantomData,
                }),
                Tokens::new(),
            )
        }
    }
}

impl<K> Enabled<EIController<K>, U0>
where
    K: AnyClock,
{
    /// Software reset needs to be synchronised
    fn syncbusy_swrst(&self) {
        while self.0.eic.syncbusy.read().swrst().bit_is_set() {
            cortex_m::asm::nop();
        }
    }
}

impl<K, N> Enabled<EIController<K>, N>
where
    K: AnyClock,
    N: Counter,
{
    pub(super) fn set_sense_mode<E: EINum>(&mut self, sense: Sense) {
        let index = match E::NUM {
            0..=7 => 0,
            // Requires rust 1.55, otherwise use _
            8.. => 1,
            //_ => 1,
        };
        //self.0.eic.config[index].write(|w| unsafe { w.bits(E::SENSE & sense as u32)
        // });
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

/*
 * Currently broken
 *
impl<CS> Enabled<EIController<WithClock<CS>>, U0>
where
    CS: EIClkSrc,
{
    /// Softare reset the EIC controller
    ///
    /// Will clear all registers and leave the controller disabled
    /// Return the same kind that was configured previously
    /// #TODO, not verified, broken, disable for now
    pub fn swrst(self) -> Enabled<EIController<WithClock<CS>>, U0> {
        let (controller, _, _) = EIController::new(self.0.eic, self.0._clockmode);
        controller
    }
}

impl Enabled<EIController<NoClock>, U0> {
    /// Softare reset the EIC controller
    ///
    /// Will clear all registers and leave the controller disabled
    /// Return the same kind that was configured previously
    /// #TODO, not verified, broken, disable for now
    pub fn swrst(self) -> Enabled<EIController<NoClock>, U0> {
        let (controller, _) = EIController::new_only_async(self.0.eic);
        controller
    }
}
*/

impl<CS> Enabled<EIController<WithClock<CS>>, U0>
where
    CS: EIClkSrc + Decrement,
{
    /// Disable and destroy the EIC controller
    pub fn destroy<S>(self, _tokens: Tokens, clock: CS) -> (crate::pac::EIC, CS::Dec)
    where
        S: EIClkSrc + Decrement,
    {
        (self.0.eic, clock.dec())
    }
}

impl Enabled<EIController<NoClock>, U0> {
    /// Disable and destroy the EIC controller
    pub fn destroy(self, _tokens: Tokens) -> crate::pac::EIC {
        self.0.eic
    }
}

macro_rules! set_filten {
    ($self:ident, $index:expr, $number:expr) => {
        paste! {
            $self.0.eic.config[1].write(|w| w.[<filten $number>]().bit(
                ($self.0.config.$index).[<get_filten $number>]() != 0
                    ))
        }
    };
}

impl<CS, N> Enabled<EIController<WithClock<CS>>, N>
where
    CS: EIClkSrc,
    N: Counter,
{
    /// TODO
    pub fn new_sync<I, C>(
        &self,
        token: Token<I::EINum>,
        pin: Pin<I, Interrupt<C>>,
    ) -> ExtInt<I, C, WithClock<CS>, SenseNone>
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
    pub(super) fn enable_debouncing<E: EINum>(&mut self) {
        self.0.eic.debouncen.modify(|r, w| unsafe {
            let bits = r.debouncen().bits();
            w.debouncen().bits(bits | E::MASK)
        });
    }

    /// TODO
    pub(super) fn disable_debouncing<E: EINum>(&mut self) {
        self.0.eic.debouncen.modify(|r, w| unsafe {
            let bits = r.debouncen().bits();
            // Cler specific bit
            w.debouncen().bits(bits & 0 << E::NUM)
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
        // Set the FILTEN bit in the configuration state

        // Write the configuration state to hardware
        match E::NUM {
            0 =>{set_filten!(self, 0, 0)},
            1 =>{set_filten!(self, 0, 1)},
            2 =>{set_filten!(self, 0, 2)},
            3 =>{set_filten!(self, 0, 3)},
            4 =>{set_filten!(self, 0, 4)},
            5 =>{set_filten!(self, 0, 5)},
            6 =>{set_filten!(self, 0, 6)},
            7 =>{set_filten!(self, 0, 7)},
            8 =>{set_filten!(self, 1, 0)},
            9 =>{set_filten!(self, 1, 1)},
            10 => {set_filten!(self, 1, 2)},
            11 => {set_filten!(self, 1, 3)},
            12 => {set_filten!(self, 1, 4)},
            13 => {set_filten!(self, 1, 5)},
            14 => {set_filten!(self, 1, 6)},
            15 => {set_filten!(self, 1, 7)},
            _ => unimplemented!(),
        }
    }

    /// TODO
    pub(super) fn disable_filtering<E: EINum>(&mut self) {
        let index = match E::NUM {
            0..=7 => 0,
            _ => 1,
        };
        // Clear the FILTEN bit
        self.0.eic.config[index].write(|w| unsafe { w.bits(0 << E::FILTEN) });
    }
}

impl<K, N> Enabled<EIController<K>, N>
where
    K: AnyClock,
    N: Counter,
{
    /// TODO
    pub fn new_async<I, C>(
        &self,
        token: Token<I::EINum>,
        pin: Pin<I, Interrupt<C>>,
    ) -> AsyncExtInt<I, C, NoClock, SenseNone>
    where
        I: GetEINum,
        C: InterruptConfig,
    {
        ExtInt::new_async(token, pin)
    }
}
