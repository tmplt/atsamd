use core::marker::PhantomData;

use crate::clock::types::{Counter, Enabled};
use crate::gpio::v2::{Interrupt, InterruptConfig, Pin};

use crate::eic::v2::*;

//use super::AnyExtInt;
/*

pub struct SyncExtInt<I, C, F, B, S>
where
    I: GetEINum,
    C: InterruptConfig,
    F: FilteringT,
    B: DebouncingT,
    S: SenseModeT,
{
    regs: Registers<I::EINum>,
    #[allow(dead_code)]
    pin: Pin<I, Interrupt<C>>,
    filtering: PhantomData<F>,
    debouncing: PhantomData<B>,
    sensemode: PhantomData<S>,
}

impl<I, C> SyncExtInt<I, C, FilteringDisabled, DebouncingDisabled, SenseNone>
where
    I: GetEINum,
    C: InterruptConfig,
{
    /// TODO
    pub(crate) fn new_sync(token: Token<I::EINum>, pin: Pin<I, Interrupt<C>>) -> Self {
        // Configure the SyncExtInt (e.g. set the Asynchronous Mode register)
        SyncExtInt {
            regs: token.regs,
            pin,
            filtering: PhantomData,
            debouncing: PhantomData,
            sensemode: PhantomData,
        }
    }
}
*/

/*
pub trait SyncExtInt {
    /// TODO
    fn new_sync<I, C>(token: Token<I::EINum>, pin: Pin<I, Interrupt<C>>) -> Self {
        // Configure the SyncExtInt (e.g. set the Asynchronous Mode register)
        SyncExtInt {
            regs: token.regs,
            pin,
            filtering: PhantomData,
            debouncing: PhantomData,
            sensemode: PhantomData,
        }
    }
}
*/

//impl<E> SyncExtInt
//where
//E: AnyExtInt<Filtering = FilteringDisabled, Debouncing = DebouncingDisabled, SenseMode = SenseNone>,
//{
//}

/*
impl<I, C, S> SyncExtInt<I, C, FilteringDisabled, DebouncingDisabled, S>
where
    I: GetEINum,
    C: InterruptConfig,
    S: SenseModeT,
{
    // Methods related to filtering and debouncing go here,
    // since they require a clock

    // Must have access to the EIController here
    /// TODO
    pub fn set_sense<K, N>(&self, eic: &mut Enabled<EIController<WithClock<K>>, N>, sense: Sense)
    where
        K: EIClkSrc,
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(sense);
    }

    /// TODO
    pub fn enable_debouncer<K, N>(
        self,
        eic: &mut Enabled<EIController<WithClock<K>>, N>,
    ) -> SyncExtInt<I, C, FilteringDisabled, DebouncingEnabled, S>
    where
        K: EIClkSrc,
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.enable_debouncer::<I::EINum>();
        SyncExtInt {
            regs: self.regs,
            pin: self.pin,
            filtering: self.filtering,
            debouncing: PhantomData::<DebouncingEnabled>,
            sensemode: self.sensemode,
        }
    }

    // Must have access to the EIController here
    /// TODO
    pub fn enable_filtering<K, N>(
        self,
        eic: &mut Enabled<EIController<WithClock<K>>, N>,
    ) -> SyncExtInt<I, C, FilteringEnabled, DebouncingDisabled, S>
    where
        K: EIClkSrc,
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.enable_filtering::<I::EINum>();
        SyncExtInt {
            regs: self.regs,
            pin: self.pin,
            filtering: PhantomData::<FilteringEnabled>,
            debouncing: self.debouncing,
            sensemode: self.sensemode,
        }
    }
}

impl<I, C, S> SyncExtInt<I, C, FilteringDisabled, DebouncingEnabled, S>
where
    I: GetEINum,
    C: InterruptConfig,
    S: SenseModeT,
{
    /// TODO
    pub fn set_debouncer_settings<K, N>(
        self,
        eic: &mut Enabled<EIController<WithClock<K>>, N>,
        settings: &DebouncerSettings,
    ) where
        K: EIClkSrc,
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.set_debouncer_settings::<I::EINum>(settings);
    }
}

*/
