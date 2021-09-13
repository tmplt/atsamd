use crate::gpio::v2::InterruptConfig;

use crate::eic::v2::*;

use super::ExtInt;

pub struct FilteredExtInt<I, C, CS, S>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    S: SenseMode,
{
    pub extint: ExtInt<I, C, WithClock<CS>, S>,
}

impl<I, C, CS, S> FilteredExtInt<I, C, CS, S>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    S: SenseMode,
{
}
