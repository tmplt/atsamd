use crate::gpio::v2::InterruptConfig;

use crate::eic::v2::*;

use super::ExtInt;

pub struct DebouncedExtInt<I, C, K, S>
where
    I: GetEINum,
    C: InterruptConfig,
    K: EIClkSrc,
    S: SenseMode,
{
    pub extint: ExtInt<I, C, WithClock<K>, S>,
}

impl<I, C, K, S> DebouncedExtInt<I, C, K, S>
where
    I: GetEINum,
    C: InterruptConfig,
    K: EIClkSrc,
    S: SenseMode,
{
}

impl<I, C, K, S> DebouncedExtInt<I, C, K, S>
where
    I: GetEINum,
    C: InterruptConfig,
    K: EIClkSrc,
    S: SenseMode,
{
}
