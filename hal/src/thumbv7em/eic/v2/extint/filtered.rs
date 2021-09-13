use crate::gpio::v2::InterruptConfig;

use crate::eic::v2::*;

use super::ExtInt;

pub struct FilteredExtInt<I, C, K, S>
where
    I: GetEINum,
    C: InterruptConfig,
    K: EIClkSrc,
    S: SenseMode,
{
    pub extint: ExtInt<I, C, WithClock<K>, S>,
}

impl<I, C, K, S> FilteredExtInt<I, C, K, S>
where
    I: GetEINum,
    C: InterruptConfig,
    K: EIClkSrc,
    S: SenseMode,
{
}
