use core::marker::PhantomData;

use crate::clock::types::{Counter, Enabled};
use crate::gpio::v2::{Interrupt, InterruptConfig, Pin};

use crate::eic::v2::*;

impl<I, C> ExtInt<I, C, AsyncMode, FilteringDisabled, DebouncingDisabled, SenseNone>
where
    I: GetEINum,
    C: InterruptConfig,
{
    /// TODO
    pub fn new_async(token: Token<I::EINum>, pin: Pin<I, Interrupt<C>>) -> Self {
        // Configure the ExtInt (e.g. set the Asynchronous Mode register)
        ExtInt {
            regs: token.regs,
            pin,
            mode: AsyncMode,
            filtering: PhantomData,
            debouncing: PhantomData,
            sensemode: PhantomData,
        }
    }
}

impl<I, C, S> ExtInt<I, C, AsyncMode, FilteringDisabled, DebouncingDisabled, S>
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
