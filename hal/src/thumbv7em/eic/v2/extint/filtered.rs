use crate::gpio::v2::InterruptConfig;

use crate::eic::v2::*;

use super::ExtInt;

pub struct FilteredExtInt<I, C, K, S>
where
    I: GetEINum,
    C: InterruptConfig,
    K: AnyClock,
    S: SenseMode,
{
    pub extint: ExtInt<I, C, K, S>,
}

macro_rules! set_sense {
    ($self:ident, $sense:ident) => {
        paste! {
            /// TODO Set FilteredExtInt Sense to [<$sense>]
            pub fn [<set_sense_$sense:lower>](self) -> FilteredExtInt<I, C, AK, [<Sense$sense>]>
            {
                self.extint.regs.set_sense_mode(Sense::$sense);

                FilteredExtInt {
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

impl<I, C, AK, S> FilteredExtInt<I, C, AK, S>
where
    I: GetEINum,
    C: InterruptConfig,
    AK: AnyClock,
    S: SenseMode,
{
    // Do not need access to the EIController here
    /// Read the pin state of the ExtInt
    /// TODO
    pub fn pin_state(&self) -> bool {
        self.extint.pin_state()
    }

    /// TODO
    pub fn disable_filtering<N>(
        self,
        eic: &mut Enabled<EIController<WithClock<AK::ClockSource>>, N>,
    ) -> ExtInt<I, C, AK, S>
    where
        N: Counter,
    {
        // Could pass the MASK directly instead of making this function
        // generic over the EINum. Either way is fine.
        eic.disable_filtering::<I::EINum>();
        // Return the inner ExtInt<...>
        self.extint
    }

    set_sense! {self, None}
    set_sense! {self, High}
    set_sense! {self, Low}
    set_sense! {self, Both}
    set_sense! {self, Rise}
    set_sense! {self, Fall}

}
