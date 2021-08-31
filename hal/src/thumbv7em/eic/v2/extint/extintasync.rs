use core::marker::PhantomData;

use crate::clock::types::{Counter, Enabled};
use crate::gpio::v2::{Interrupt, InterruptConfig, Pin};

use crate::eic::v2::*;

pub struct AsyncExtInt<I, C, F, B, S>
where
    I: GetEINum,
    C: InterruptConfig,
    F: Filtering,
    B: Debouncing,
    S: SenseMode,
{
    #[allow(dead_code)]
    regs: Registers<I::EINum>,
    #[allow(dead_code)]
    pin: Pin<I, Interrupt<C>>,
    filtering: PhantomData<F>,
    debouncing: PhantomData<B>,
    sensemode: PhantomData<S>,
}

impl<I, C> AsyncExtInt<I, C, FilteringDisabled, DebouncingDisabled, SenseNone>
where
    I: GetEINum,
    C: InterruptConfig,
{
    /// TODO
    pub fn new_async(token: Token<I::EINum>, pin: Pin<I, Interrupt<C>>) -> Self {
        // Configure the AsyncExtInt (e.g. set the Asynchronous Mode register)
        AsyncExtInt {
            regs: token.regs,
            pin,
            filtering: PhantomData,
            debouncing: PhantomData,
            sensemode: PhantomData,
        }
    }
}

impl<I, C, S> AsyncExtInt<I, C, FilteringDisabled, DebouncingDisabled, S>
where
    I: GetEINum,
    C: InterruptConfig,
    S: SenseMode,
{
    /// TODO
    pub fn set_sense<K, N>(
        &self,
        eic: &mut Enabled<EIController<NoClockOnlyAsync>, N>,
        sense: Sense,
    ) where
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(sense);
    }
}
