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

    /// TODO
    pub fn set_sense_high<AK2, N>(
        self,
        eic: &mut Enabled<EIController<AK2>, N>,
    ) -> AsyncExtInt<I, C, AK, SenseHigh>
    where
        AK2: AnyClock,
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(Sense::High);

        AsyncExtInt {
            extint: ExtInt {
                regs: self.extint.regs,
                pin: self.extint.pin,
                clockmode: PhantomData,
                sensemode: PhantomData,
            },
        }
    }
    /// TODO
    pub fn set_sense_low<AK2, N>(
        self,
        eic: &mut Enabled<EIController<AK2>, N>,
    ) -> AsyncExtInt<I, C, AK, SenseLow>
    where
        AK2: AnyClock,
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(Sense::Low);

        AsyncExtInt {
            extint: ExtInt {
                regs: self.extint.regs,
                pin: self.extint.pin,
                clockmode: PhantomData,
                sensemode: PhantomData,
            },
        }
    }
    /// TODO
    pub fn set_sense_rise<AK2, N>(
        self,
        eic: &mut Enabled<EIController<AK2>, N>,
    ) -> AsyncExtInt<I, C, AK, SenseRise>
    where
        AK2: AnyClock,
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(Sense::Rise);

        AsyncExtInt {
            extint: ExtInt {
                regs: self.extint.regs,
                pin: self.extint.pin,
                clockmode: PhantomData,
                sensemode: PhantomData,
            },
        }
    }
    /// TODO
    pub fn set_sense_fall<AK2, N>(
        self,
        eic: &mut Enabled<EIController<AK2>, N>,
    ) -> AsyncExtInt<I, C, AK, SenseFall>
    where
        AK2: AnyClock,
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(Sense::Fall);

        AsyncExtInt {
            extint: ExtInt {
                regs: self.extint.regs,
                pin: self.extint.pin,
                clockmode: PhantomData,
                sensemode: PhantomData,
            },
        }
    }
    /// TODO
    pub fn set_sense_both<AK2, N>(
        self,
        eic: &mut Enabled<EIController<AK2>, N>,
    ) -> AsyncExtInt<I, C, AK, SenseBoth>
    where
        AK2: AnyClock,
        N: Counter,
    {
        eic.set_sense_mode::<I::EINum>(Sense::Both);

        AsyncExtInt {
            extint: ExtInt {
                regs: self.extint.regs,
                pin: self.extint.pin,
                clockmode: PhantomData,
                sensemode: PhantomData,
            },
        }
    }
}
