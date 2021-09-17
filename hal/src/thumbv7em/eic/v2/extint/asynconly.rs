use super::*;
use crate::gpio::v2::InterruptConfig;

impl<I, AM, C> ExtInt<I, C, AM, NoClock, SenseNone>
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
    ) -> ExtInt<I, C, AM, NoClock, SenseNone> {
        // #TODO
        // Read the current asynch register
        let val = token.regs.eic().asynch.read().bits() as u16;
        // Set the bit corresponding to the EINum to one
        token
            .regs
            .eic()
            .asynch
            .write(|w| unsafe { w.asynch().bits(val & (1 << <I as GetEINum>::EINum::NUM)) });
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
    AM: AnyMode<Mode = AsyncOnly>,
    AK: AnyClock,
    AS: AnySenseMode,
{
    /// TODO
    /// Only possible to deactivate AnyMode<Mode = AsyncOnly>
    /// when EIController has access to a clock source.
    /// FIXME
    pub fn disable_asynconly<AM2, N>(
        self,
        eic: &mut Enabled<EIController<WithClock<AK::ClockSource>, Configurable>, N>,
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
