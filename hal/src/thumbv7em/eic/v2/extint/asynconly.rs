use super::*;
use crate::gpio::v2::InterruptConfig;
use bitfield::{Bit, BitRange};

impl<I, AM, C> ExtInt<I, C, AM, WithoutClock, SenseNone>
where
    I: GetEINum,
    C: InterruptConfig,
    AM: AnyMode<Mode = AsyncOnly>,
{
    /// Create initial asynchronous ExtInt
    /// TODO
    pub(crate) fn new_async(
        token: Token<I::EINum>,
        pin: Pin<I, Interrupt<C>>,
    ) -> ExtInt<I, C, AM, WithoutClock, SenseNone> {
        let bitnum: usize = I::EINum::NUM.into();

        // Read the current asynch register
        let mut asynch_reg = EIReg(token.regs.eic().asynch.read().bits() as u16);
        // Enable the asynch-bit
        asynch_reg.set_bit(bitnum, true);

        // Set the bit corresponding to the EINum
        token
            .regs
            .eic()
            .asynch
            .write(|w| unsafe { w.asynch().bits(asynch_reg.bit_range(15, 0)) });

        ExtInt {
            token,
            pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}
impl<I, C, AM, AK, AS> ExtInt<I, C, AM, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    AM: AnyMode<Mode = Normal>,
    AK: AnyClock,
    AS: AnySenseMode,
{
    /// TODO
    /// Only possible to change to AnyMode<Mode = AsyncOnly>
    /// when EIController has access to a clock source because
    /// otherwise locked in AsyncOnly
    pub fn enable_async<AM2, N>(
        self,
        eic: &Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
    ) -> ExtInt<I, C, AM2, AK, AS>
    where
        N: Counter,
        AM2: AnyMode<Mode = AsyncOnly>,
    {
        eic.enable_async::<I::EINum>();

        ExtInt {
            token: self.token,
            pin: self.pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}
impl<I, C, AM, AK, AS> ExtInt<I, C, AM, AK, AS>
where
    I: GetEINum,
    C: InterruptConfig,
    AM: AnyMode<Mode = AsyncOnly>,
    AK: AnyClock,
    AS: AnySenseMode,
{
    /// TODO
    /// Only possible to deactivate AnyMode<Mode = AsyncOnly>
    /// when EIController has access to a clock source.
    ///
    /// Ensuring that if EIController was created without
    /// a clocksource it is the only available mode
    pub fn disable_async<AM2, N>(
        self,
        // EIContrtoller<WithClock<...>> ensures EIController has a clocksource
        eic: &Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
    ) -> ExtInt<I, C, AM2, AK, AS>
    where
        N: Counter,
        AM2: AnyMode<Mode = Normal>,
    {
        eic.disable_async::<I::EINum>();

        ExtInt {
            token: self.token,
            pin: self.pin,
            mode: PhantomData,
            clockmode: PhantomData,
            sensemode: PhantomData,
        }
    }
}
