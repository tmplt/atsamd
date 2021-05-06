use core::marker::PhantomData;

use crate::pac::oscctrl::xoscctrl::{CFDPRESC_A, STARTUP_A};
use crate::pac::oscctrl::{RegisterBlock, XOSCCTRL};

use crate::gpio::v2::{AnyPin, FloatingDisabled, OptionalPin, Pin, PinId, PA14, PA15, PB22, PB23};
use crate::time::Hertz;
use crate::typelevel::{Count, Decrement, Increment, Lockable, NoneT, Sealed, Unlockable, Zero};

use super::super::gclk::{GclkSource, GclkSourceEnum, GclkSourceType, GenNum};
use super::dpll::{DpllSource, DpllSourceType, DpllSrc};

//==============================================================================
// XOscNum
//==============================================================================

/// TODO
pub trait XOscNum: Sealed {
    const NUM: usize;
    const DPLL_SRC: DpllSrc;
    type XIn: PinId;
    type XOut: PinId;
}

/// TODO
pub enum Osc0 {}

impl Sealed for Osc0 {}

impl XOscNum for Osc0 {
    const NUM: usize = 0;
    const DPLL_SRC: DpllSrc = DpllSrc::XOSC0;
    type XIn = PA14;
    type XOut = PA15;
}

/// TODO
pub enum Osc1 {}

impl Sealed for Osc1 {}

impl XOscNum for Osc1 {
    const NUM: usize = 1;
    const DPLL_SRC: DpllSrc = DpllSrc::XOSC1;
    type XIn = PB22;
    type XOut = PB23;
}

#[derive(Debug, Clone, PartialEq)]
/// Current mutliplier/reference pair
pub enum CrystalCurrent {
    /// 8MHz
    BaseFreq,
    /// 8 to 16MHz
    LowFreq,
    /// 16 to 24MHz
    MedFreq,
    /// 244 to 48MHz
    HighFreq,
}

impl CrystalCurrent {
    /// Get the current multiplier
    pub fn imult(&self) -> u8 {
        match &self {
            Self::BaseFreq => 3,
            Self::LowFreq => 4,
            Self::MedFreq => 5,
            Self::HighFreq => 6,
        }
    }

    /// Get the current reference
    pub fn iptat(&self) -> u8 {
        match &self {
            Self::BaseFreq => 2,
            Self::LowFreq => 3,
            Self::MedFreq => 3,
            Self::HighFreq => 3,
        }
    }
}

//==============================================================================
// Registers
//==============================================================================

pub type XOscToken<X> = Registers<X>;

pub struct Registers<X: XOscNum> {
    osc: PhantomData<X>,
}

impl<X: XOscNum> Registers<X> {
    /// TODO
    #[inline]
    pub(super) unsafe fn new() -> Self {
        Self { osc: PhantomData }
    }

    #[inline]
    fn oscctrl(&self) -> &RegisterBlock {
        unsafe { &*crate::pac::OSCCTRL::ptr() }
    }

    #[inline]
    fn xoscctrl(&self) -> &XOSCCTRL {
        &self.oscctrl().xoscctrl[X::NUM]
    }

    #[inline]
    fn set_start_up(&mut self, start_up: StartUp) {
        self.xoscctrl().modify(|_, w| w.startup().variant(start_up));
    }

    #[inline]
    fn set_on_demand(&mut self, on_demand: bool) {
        self.xoscctrl().modify(|_, w| w.ondemand().bit(on_demand));
    }

    #[inline]
    fn set_run_standby(&mut self, run_standby: bool) {
        self.xoscctrl().modify(|_, w| w.runstdby().bit(run_standby));
    }

    #[inline]
    fn from_clock(&mut self) {
        self.xoscctrl().modify(|_, w| w.xtalen().bit(false));
    }

    #[inline]
    fn from_crystal(&mut self) {
        self.xoscctrl().modify(|_, w| w.xtalen().bit(true));
    }

    #[inline]
    fn enable(&mut self) {
        self.xoscctrl().modify(|_, w| w.enable().bit(true));
    }

    #[inline]
    fn disable(&mut self) {
        self.xoscctrl().modify(|_, w| w.enable().bit(false));
    }

    #[inline]
    fn wait_ready(&self) {
        let mask = 1 << X::NUM;
        while self.oscctrl().status.read().bits() & mask == 0 {}
    }

    #[inline]
    fn set_clock_failure_detector_prescaler(&mut self, prescale: CFDPRESC_A) {
        self.xoscctrl()
            .modify(|_, w| w.cfdpresc().variant(prescale));
    }
    #[inline]
    fn set_crystal_current(&mut self, cc: CrystalCurrent) {
        self.xoscctrl().modify(|_, w| unsafe {
            w.imult().bits(cc.imult());
            w.iptat().bits(cc.iptat())
        });
    }
    #[inline]
    fn set_clock_switch(&mut self, swben: bool) {
        self.xoscctrl().modify(|_, w| w.swben().bit(swben));
    }

    #[inline]
    fn set_low_buf_gain(&mut self, lowbufgain: bool) {
        self.xoscctrl()
            .modify(|_, w| w.lowbufgain().bit(lowbufgain));
    }
}

//==============================================================================
// Aliases
//==============================================================================

/// TODO
pub type StartUp = STARTUP_A;

/// TODO
pub type XIn<X> = Pin<<X as XOscNum>::XIn, FloatingDisabled>;

/// TODO
pub type XOut<X> = Pin<<X as XOscNum>::XOut, FloatingDisabled>;

//==============================================================================
// XOscConfig
//==============================================================================

pub struct XOscConfig<X, P = NoneT>
where
    X: XOscNum,
    P: OptionalPin,
{
    token: XOscToken<X>,
    xin: XIn<X>,
    xout: P,
    freq: Hertz,
}

impl<X: XOscNum> XOscConfig<X> {
    /// TODO
    #[inline]
    pub fn from_clock(
        mut token: XOscToken<X>,
        xin: impl AnyPin<Id = X::XIn>,
        freq: impl Into<Hertz>,
    ) -> Self {
        let xin = xin.into().into_floating_disabled();
        token.from_clock();
        // TODO
        Self {
            token,
            xin,
            xout: NoneT,
            freq: freq.into(),
        }
    }

    /// TODO
    #[inline]
    pub fn free(self) -> (XOscToken<X>, XIn<X>) {
        (self.token, self.xin)
    }
}

impl<X: XOscNum> XOscConfig<X, XOut<X>> {
    /// TODO
    #[inline]
    pub fn from_crystal(
        mut token: XOscToken<X>,
        xin: impl AnyPin<Id = X::XIn>,
        xout: impl AnyPin<Id = X::XOut>,
        freq: impl Into<Hertz>,
    ) -> Self {
        let xin = xin.into().into_floating_disabled();
        let xout = xout.into().into_floating_disabled();
        // TODO
        token.from_crystal();
        Self {
            token,
            xin,
            xout,
            freq: freq.into(),
        }
    }

    /// TODO
    #[inline]
    pub fn free(self) -> (XOscToken<X>, XIn<X>, XOut<X>) {
        (self.token, self.xin, self.xout)
    }
}

impl<X, P> XOscConfig<X, P>
where
    X: XOscNum,
    P: OptionalPin,
{
    /// TODO
    #[inline]
    pub fn freq(&self) -> Hertz {
        self.freq
    }

    /// TODO
    #[inline]
    pub fn set_start_up(mut self, start_up: StartUp) -> Self {
        self.token.set_start_up(start_up);
        self
    }

    /// TODO
    #[inline]
    pub fn set_on_demand(mut self, on_demand: bool) -> Self {
        self.token.set_on_demand(on_demand);
        self
    }

    /// TODO
    #[inline]
    pub fn set_run_standby(mut self, run_standby: bool) -> Self {
        self.token.set_run_standby(run_standby);
        self
    }

    /// TODO
    #[inline]
    pub fn set_clock_failure_detector_prescaler(mut self, prescale: CFDPRESC_A) -> Self {
        self.token.set_clock_failure_detector_prescaler(prescale);
        self
    }

    /// TODO
    #[inline]
    pub fn set_crystal_current(mut self, crystal_current: CrystalCurrent) -> Self {
        self.token.set_crystal_current(crystal_current);
        self
    }

    /// TODO
    #[inline]
    pub fn enable(mut self) -> XOsc<X, P> {
        self.token.enable();
        XOsc::new(self)
    }
}

//==============================================================================
// XOsc
//==============================================================================

pub struct XOsc<X, P = NoneT, N = Zero>
where
    X: XOscNum,
    P: OptionalPin,
    N: Count,
{
    config: XOscConfig<X, P>,
    count: N,
}
///
/// TODO
pub type XOsc0<P = NoneT> = XOsc<Osc0, P>;

/// TODO
pub type XOsc1<P = NoneT> = XOsc<Osc1, P>;

impl<X, P, N> Sealed for XOsc<X, P, N>
where
    X: XOscNum,
    P: OptionalPin,
    N: Count,
{
}

impl<X, P> XOsc<X, P>
where
    X: XOscNum,
    P: OptionalPin,
{
    /// TODO
    #[inline]
    fn new(config: XOscConfig<X, P>) -> Self {
        let count = Zero::new();
        XOsc { config, count }
    }

    /// TODO
    #[inline]
    pub fn disable(mut self) -> XOscConfig<X, P> {
        self.config.token.disable();
        self.config
    }
}

impl<X, P, N> XOsc<X, P, N>
where
    X: XOscNum,
    P: OptionalPin,
    N: Count,
{
    /// TODO
    #[inline]
    fn create(config: XOscConfig<X, P>, count: N) -> Self {
        XOsc { config, count }
    }

    /// TODO
    #[inline]
    pub fn wait_ready(&self) {
        self.config.token.wait_ready();
    }

    /// TODO
    #[inline]
    pub fn freq(&self) -> Hertz {
        self.config.freq()
    }
}

//==============================================================================
// Lockable
//==============================================================================

impl<X, P, N> Lockable for XOsc<X, P, N>
where
    X: XOscNum,
    P: OptionalPin,
    N: Increment,
{
    type Locked = XOsc<X, P, N::Inc>;
    fn lock(self) -> Self::Locked {
        XOsc::create(self.config, self.count.inc())
    }
}

//==============================================================================
// Unlockable
//==============================================================================

impl<X, P, N> Unlockable for XOsc<X, P, N>
where
    X: XOscNum,
    P: OptionalPin,
    N: Decrement,
{
    type Unlocked = XOsc<X, P, N::Dec>;
    fn unlock(self) -> Self::Unlocked {
        XOsc::create(self.config, self.count.dec())
    }
}

//==============================================================================
// GclkSource
//==============================================================================

impl GclkSourceType for Osc0 {
    const GCLK_SRC: GclkSourceEnum = GclkSourceEnum::XOSC0;
}

impl GclkSourceType for Osc1 {
    const GCLK_SRC: GclkSourceEnum = GclkSourceEnum::XOSC0;
}

impl<G, X, P, N> GclkSource<G> for XOsc<X, P, N>
where
    G: GenNum,
    X: XOscNum + GclkSourceType,
    P: OptionalPin,
    N: Count,
{
    type Type = X;

    #[inline]
    fn freq(&self) -> Hertz {
        self.config.freq
    }
}

//==============================================================================
// DpllSource
//==============================================================================

impl DpllSourceType for Osc0 {
    const DPLL_SRC: DpllSrc = DpllSrc::XOSC0;
}

impl DpllSourceType for Osc1 {
    const DPLL_SRC: DpllSrc = DpllSrc::XOSC1;
}

impl<X, P, N> DpllSource for XOsc<X, P, N>
where
    X: XOscNum + DpllSourceType,
    P: OptionalPin,
    N: Count,
{
    type Type = X;

    #[inline]
    fn freq(&self) -> Hertz {
        self.config.freq
    }
}
