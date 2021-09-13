use crate::eic::v2::*;
use crate::gpio::v2::InterruptConfig;

use super::{AnyExtInt, ExtInt};

//pub struct DebouncedExtInt<E>
pub struct DebouncedExtInt<I, C, /*CS*/K, S>
where
    //E: AnyExtInt,
    I: GetEINum,
    C: InterruptConfig,
    //CS: EIClkSrc,
    K: Clock,
    S: SenseMode,
{
    //pub extint: E,
    pub extint: ExtInt<I, C, K/*WithClock<CS>*/, S>,
}

/*
impl<I, C, CS, S> DebouncedExtInt<I, C, CS, S>
where
    I: GetEINum,
    C: InterruptConfig,
    CS: EIClkSrc,
    S: SenseMode,
{
}
*/

//impl<E> DebouncedExtInt<E> where E: AnyExtInt<SenseMode = SenseRise> {}

//impl<E> DebouncedExtInt<E> where E: AnyExtInt {}
