//! TODO

use core::marker::PhantomData;

use crate::pac::oscctrl::dpll::{dpllstatus, dpllsyncbusy, DPLLCTRLA, DPLLCTRLB, DPLLRATIO};
use crate::pac::oscctrl::DPLL;

pub use crate::pac::oscctrl::dpll::dpllctrlb::REFCLK_A as DpllSrc;

use crate::time::Hertz;
use crate::typelevel::counted::Counted;
use crate::typelevel::{Decrement, Increment, PrivateDecrement, PrivateIncrement, Sealed, Zero};

use super::super::gclk::{GclkSource, GclkSourceEnum, GclkSourceType, GenNum};
use super::super::pclk::{Pclk, PclkSourceType, PclkType};

//==============================================================================
// DpllNum
//==============================================================================

pub trait DpllNum: Sealed {
    const NUM: usize;
}

pub enum Pll0 {}

impl Sealed for Pll0 {}

impl DpllNum for Pll0 {
    const NUM: usize = 0;
}

pub enum Pll1 {}

impl Sealed for Pll1 {}

impl DpllNum for Pll1 {
    const NUM: usize = 1;
}

//==============================================================================
// DpllSource
//==============================================================================

/// TODO
pub trait DpllSourceType: Sealed {
    const DPLL_SRC: DpllSrc;
}

impl<D, T> DpllSourceType for Pclk<D, T>
where
    D: DpllNum + PclkType,
    T: PclkSourceType,
{
    const DPLL_SRC: DpllSrc = DpllSrc::GCLK;
}

/// TODO
pub trait DpllSource: Sealed {
    type Type: DpllSourceType;
    fn freq(&self) -> Hertz;
}

//==============================================================================
// Registers
//==============================================================================

pub type DpllToken<D> = Registers<D>;

pub struct Registers<D: DpllNum> {
    dpll: PhantomData<D>,
}

impl<D: DpllNum> Registers<D> {
    /// TODO
    #[inline]
    pub(super) unsafe fn new() -> Self {
        Self { dpll: PhantomData }
    }

    #[inline]
    fn dpll(&self) -> &DPLL {
        // TODO
        unsafe { &(*crate::pac::OSCCTRL::ptr()).dpll[D::NUM] }
    }

    #[inline]
    fn ctrla(&self) -> &DPLLCTRLA {
        &self.dpll().dpllctrla
    }

    #[inline]
    fn ctrlb(&self) -> &DPLLCTRLB {
        &self.dpll().dpllctrlb
    }

    #[inline]
    fn ratio(&self) -> &DPLLRATIO {
        &self.dpll().dpllratio
    }

    #[inline]
    fn syncbusy(&self) -> dpllsyncbusy::R {
        self.dpll().dpllsyncbusy.read()
    }

    #[inline]
    fn status(&self) -> dpllstatus::R {
        self.dpll().dpllstatus.read()
    }

    // TODO
    #[inline]
    fn set_loop_div(&mut self, int: u16, frac: u8) {
        // TODO
        self.ratio().write(|w| unsafe {
            w.ldr().bits((int - 1) & 0x1FFF);
            w.ldrfrac().bits(frac & 0x1F)
        });
        while self.syncbusy().dpllratio().bit_is_set() {}
    }

    // TODO
    #[inline]
    fn set_source_clock(&mut self, variant: DpllSrc) {
        self.ctrlb().modify(|_, w| w.refclk().variant(variant));
    }

    // TODO
    #[inline]
    fn set_source_div(&mut self, div: u16) {
        // TODO
        self.ctrlb()
            .modify(|_, w| unsafe { w.div().bits(div & 0x7FF) });
    }

    // TODO
    #[inline]
    fn set_lock_bypass(&mut self, bypass: bool) {
        self.ctrlb().modify(|_, w| w.lbypass().bit(bypass));
    }

    // TODO
    #[inline]
    fn set_wake_up_fast(&mut self, wuf: bool) {
        self.ctrlb().modify(|_, w| w.wuf().bit(wuf));
    }

    // TODO
    #[inline]
    fn wait_until_ready(&self) {
        while self.status().clkrdy().bit_is_clear() {}
    }

    // TODO
    #[inline]
    fn wait_until_locked(&self) {
        while self.status().lock().bit_is_clear() {}
    }

    /// TODO
    #[inline]
    fn wait_until_enable_synced(&self) {
        // TODO
        while self.syncbusy().enable().bit_is_set() {}
    }

    // TODO
    #[inline]
    fn enable(&mut self) {
        self.ctrla().modify(|_, w| w.enable().set_bit());
        self.wait_until_enable_synced();
    }

    // TODO
    #[inline]
    fn disable(&mut self) {
        self.ctrla().modify(|_, w| w.enable().clear_bit());
        self.wait_until_enable_synced();
    }
}

//==============================================================================
// DpllConfig
//==============================================================================

/// TODO
pub struct DpllConfig<D, T>
where
    D: DpllNum,
    T: DpllSourceType,
{
    token: DpllToken<D>,
    src: PhantomData<T>,
    freq: Hertz,
    mult: u16,
    frac: u8,
    div: u16,
}

impl<D, T> DpllConfig<D, Pclk<D, T>>
where
    D: DpllNum + PclkType,
    T: PclkSourceType,
    Pclk<D, T>: DpllSourceType,
{
    /// TODO
    #[inline]
    pub fn from_pclk(mut token: DpllToken<D>, pclk: Pclk<D, T>) -> Self {
        let freq = pclk.freq();
        assert!(freq.0 >= 32_000);
        assert!(freq.0 <= 3_200_000);
        let (mult, frac, div) = (1, 0, 1);
        token.set_source_clock(Pclk::<D, T>::DPLL_SRC);
        DpllConfig {
            token,
            src: PhantomData,
            freq,
            mult,
            frac,
            div,
        }
        // If the DpllSource is a Pclk, we would prefer to store it and return
        // it when the Dpll is dropped. However, if the DpllSource is an XOsc
        // source, we can't store it. The easy solution is to drop the Pclk here
        // and recreate it later.
    }

    /// TODO
    #[inline]
    pub fn free_pclk(self) -> (DpllToken<D>, Pclk<D, T>) {
        // If the DpllSource is a Pclk, we would prefer to store it and return
        // it when the Dpll is dropped. However, if the DpllSource is an
        // instance of AnySource, we can't store it. The easy solution is to
        // drop the Pclk and recreate it here.
        let pclk = unsafe { Pclk::create(self.freq) };
        (self.token, pclk)
    }
}

impl<D, T> DpllConfig<D, T>
where
    D: DpllNum,
    T: DpllSourceType,
{
    /// TODO
    #[inline]
    pub fn from_xosc<S, N>(
        mut token: DpllToken<D>,
        source: Counted<S, N>,
    ) -> (DpllConfig<D, T>, Counted<S, N::Inc>)
    where
        S: DpllSource<Type = T>,
        N: Increment,
    {
        let freq = source.freq();
        assert!(freq.0 >= 32_000);
        assert!(freq.0 <= 3_200_000);
        let (mult, frac, div) = (1, 0, 1);
        token.set_source_clock(T::DPLL_SRC);
        let dpll = DpllConfig {
            token,
            src: PhantomData,
            freq,
            mult,
            frac,
            div,
        };
        // TODO
        (dpll, source.inc())
    }
}

impl<D, T> DpllConfig<D, T>
where
    D: DpllNum,
    T: DpllSourceType,
{
    /// TODO
    #[inline]
    pub fn free_xosc<S, N>(self, source: Counted<S, N>) -> (DpllToken<D>, Counted<S, N::Dec>)
    where
        S: DpllSource<Type = T>,
        N: Decrement,
    {
        (self.token, source.dec())
    }
}

impl<D, T> DpllConfig<D, T>
where
    D: DpllNum,
    T: DpllSourceType,
{
    // TODO
    #[inline]
    pub fn set_source_div(mut self, div: u16) -> Self {
        self.token.set_source_div(div);
        self.div = div;
        self
    }

    // TODO
    #[inline]
    pub fn set_loop_div(mut self, int: u16, frac: u8) -> Self {
        self.token.set_loop_div(int, frac);
        self.mult = int;
        self.frac = frac;
        self
    }

    // TODO
    #[inline]
    pub fn set_lock_bypass(mut self, bypass: bool) -> Self {
        self.token.set_lock_bypass(bypass);
        self
    }

    // TODO
    #[inline]
    pub fn set_wake_up_fast(mut self, wuf: bool) -> Self {
        self.token.set_wake_up_fast(wuf);
        self
    }

    /// TODO
    #[inline]
    pub fn freq(&self) -> Hertz {
        Hertz(self.freq.0 / self.div as u32 * (self.mult as u32 + 1 + self.frac as u32 / 32))
    }

    /// TODO
    #[inline]
    pub fn enable(mut self) -> Counted<Dpll<D, T>, Zero> {
        assert!(self.freq().0 >= 96_000_000);
        assert!(self.freq().0 <= 200_000_000);
        self.token.enable();
        Counted::new(Dpll::new(self))
    }
}

//==============================================================================
// Dpll
//==============================================================================

/// TODO
pub struct Dpll<D, T>
where
    D: DpllNum,
    T: DpllSourceType,
{
    config: DpllConfig<D, T>,
}

/// TODO
pub type Dpll0<T> = Dpll<Pll0, T>;

/// TODO
pub type Dpll1<T> = Dpll<Pll1, T>;

impl<D, T> Sealed for Dpll<D, T>
where
    D: DpllNum,
    T: DpllSourceType,
{
}

impl<D, T> Dpll<D, T>
where
    D: DpllNum,
    T: DpllSourceType,
{
    /// TODO
    #[inline]
    fn new(config: DpllConfig<D, T>) -> Self {
        Dpll { config }
    }

    /// TODO
    #[inline]
    pub fn disable(mut self) -> DpllConfig<D, T> {
        self.config.token.disable();
        self.config
    }

    /// TODO
    #[inline]
    pub fn wait_until_ready(&self) {
        self.config.token.wait_until_ready();
    }

    /// TODO
    #[inline]
    pub fn wait_until_locked(&self) {
        self.config.token.wait_until_locked();
    }

    /// TODO
    #[inline]
    pub fn freq(&self) -> Hertz {
        self.config.freq()
    }
}

impl<D, T> Counted<Dpll<D, T>, Zero>
where
    D: DpllNum,
    T: DpllSourceType,
{
    #[inline]
    pub fn disable(self) -> DpllConfig<D, T> {
        self.0.disable()
    }
}

//==============================================================================
// GclkSource
//==============================================================================

impl GclkSourceType for Pll0 {
    const GCLK_SRC: GclkSourceEnum = GclkSourceEnum::DPLL0;
}

impl GclkSourceType for Pll1 {
    const GCLK_SRC: GclkSourceEnum = GclkSourceEnum::DPLL1;
}

impl<G, D, T> GclkSource<G> for Dpll<D, T>
where
    G: GenNum,
    D: DpllNum + GclkSourceType,
    T: DpllSourceType,
{
    type Type = D;

    #[inline]
    fn freq(&self) -> Hertz {
        self.config.freq
    }
}
