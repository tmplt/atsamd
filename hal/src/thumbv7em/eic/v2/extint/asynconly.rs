use crate::gpio::v2::InterruptConfig;

use crate::eic::v2::*;

use super::ExtInt;

pub struct AsyncExtInt<I, C, K, S>
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
            /// TODO Set AsyncExtInt Sense to [<$sense>]
            pub fn [<set_sense_$sense:lower>](self) -> AsyncExtInt<I, C, AK, [<Sense$sense>]>
            {
                self.extint.regs.set_sense_mode(Sense::$sense);

                AsyncExtInt {
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

impl<I, C, AK, S> AsyncExtInt<I, C, AK, S>
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
    set_sense! {self, None}
    set_sense! {self, High}
    set_sense! {self, Low}
    set_sense! {self, Both}
    set_sense! {self, Rise}
    set_sense! {self, Fall}

}
