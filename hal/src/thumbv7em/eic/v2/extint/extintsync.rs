use core::marker::PhantomData;

use crate::clock::types::{Counter, Enabled};
use crate::gpio::v2::{Interrupt, InterruptConfig, Pin};

use crate::eic::v2::*;

impl<I, C> ExtInt<I, C, SyncMode, FilteringDisabled, DebouncingDisabled, SenseNone>
where
    I: GetEINum,
    C: InterruptConfig,
{
    /// TODO
    pub fn new_sync(token: Token<I::EINum>, pin: Pin<I, Interrupt<C>>) -> Self {
        // Configure the ExtInt (e.g. set the Asynchronous Mode register)
        ExtInt {
            regs: token.regs,
            pin,
            mode: SyncMode,
            filtering: PhantomData,
            debouncing: PhantomData,
            sensemode: PhantomData,
        }
    }
}

impl<I, C, S> ExtInt<I, C, SyncMode, FilteringDisabled, DebouncingDisabled, S>
where
    I: GetEINum,
    C: InterruptConfig,
    S: SenseMode,
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
    ) -> ExtInt<I, C, SyncMode, FilteringDisabled, DebouncingEnabled, S>
    where
        K: EIClkSrc,
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.enable_debouncer::<I::EINum>();
        ExtInt {
            regs: self.regs,
            pin: self.pin,
            mode: self.mode,
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
    ) -> ExtInt<I, C, SyncMode, FilteringEnabled, DebouncingDisabled, S>
    where
        K: EIClkSrc,
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.enable_filtering::<I::EINum>();
        ExtInt {
            regs: self.regs,
            pin: self.pin,
            mode: self.mode,
            filtering: PhantomData::<FilteringEnabled>,
            debouncing: self.debouncing,
            sensemode: self.sensemode,
        }
    }
}

impl<I, C, S> ExtInt<I, C, SyncMode, FilteringDisabled, DebouncingEnabled, S>
where
    I: GetEINum,
    C: InterruptConfig,
    S: SenseMode,
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
