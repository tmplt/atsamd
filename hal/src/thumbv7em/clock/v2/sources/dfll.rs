use crate::time::{Hertz, U32Ext};
use crate::typelevel::counted::Counted;
use crate::typelevel::{Count, Sealed, Zero};

use super::super::gclk::{GclkSource, GclkSourceEnum, GclkSourceType, GenNum};
use super::super::pclk::{Dfll48, Pclk, PclkSourceType};

/// TODO
pub type DfllToken = Registers;

pub struct Registers {
    __: (),
}

/// TODO
impl Registers {
    /// TODO
    #[inline]
    unsafe fn new() -> Self {
        Self { __: () }
    }

    #[inline]
    fn oscctrl(&self) -> &crate::pac::oscctrl::RegisterBlock {
        unsafe { &*crate::pac::OSCCTRL::ptr() }
    }

    #[allow(dead_code)]
    #[inline]
    fn dfllctrla(&self) -> &crate::pac::oscctrl::DFLLCTRLA {
        &self.oscctrl().dfllctrla
    }
    #[allow(dead_code)]
    #[inline]
    fn dfllctrlb(&self) -> &crate::pac::oscctrl::DFLLCTRLB {
        &self.oscctrl().dfllctrlb
    }
    #[allow(dead_code)]
    #[inline]
    fn dfllval(&self) -> &crate::pac::oscctrl::DFLLVAL {
        &self.oscctrl().dfllval
    }
    #[allow(dead_code)]
    #[inline]
    fn dfllmul(&self) -> &crate::pac::oscctrl::DFLLMUL {
        &self.oscctrl().dfllmul
    }
    #[allow(dead_code)]
    #[inline]
    fn dfllsync(&self) -> &crate::pac::oscctrl::DFLLSYNC {
        &self.oscctrl().dfllsync
    }
    #[allow(dead_code)]
    #[inline]
    fn wait_sync_enable(&self) {
        while self.dfllsync().read().enable().bit() == true {}
    }
    #[allow(dead_code)]
    #[inline]
    fn wait_sync_dfllmul(&self) {
        while self.dfllsync().read().dfllmul().bit() == true {}
    }
    #[allow(dead_code)]
    #[inline]
    fn wait_sync_dfllval(&self) {
        while self.dfllsync().read().dfllval().bit() == true {}
    }
    #[allow(dead_code)]
    #[inline]
    fn wait_sync_dfllctrlb(&self) {
        while self.dfllsync().read().dfllctrlb().bit() == true {}
    }
    #[allow(dead_code)]
    #[inline]
    fn enable(&mut self) {
        self.dfllctrla().modify(|_, w| w.enable().set_bit());
        self.wait_sync_enable();
    }
    #[allow(dead_code)]
    #[inline]
    fn set_open_mode(&mut self) {
        self.dfllctrlb().modify(|_, w| w.mode().clear_bit());
        self.wait_sync_enable();
    }
    #[allow(dead_code)]
    #[inline]
    fn set_closed_mode(&mut self) {
        self.dfllctrlb().modify(|_, w| w.mode().set_bit());
        self.wait_sync_enable();
    }
    #[allow(dead_code)]
    #[inline]
    fn set_fine_maximum_step(&mut self, value: u8) {
        self.dfllmul()
            .modify(|_, w| unsafe { w.fstep().bits(value) });
        self.wait_sync_dfllmul();
    }
    #[allow(dead_code)]
    #[inline]
    fn set_coarse_maximum_step(&mut self, value: u8) {
        self.dfllmul()
            .modify(|_, w| unsafe { w.cstep().bits(value) });
        self.wait_sync_dfllmul();
    }
    #[allow(dead_code)]
    #[inline]
    fn set_multiplication_factor(&mut self, value: u16) {
        self.dfllmul().modify(|_, w| unsafe { w.mul().bits(value) });
        self.wait_sync_dfllmul();
    }
}

type MultiplicationFactor = u16;
type CoarseMaximumStep = u8;
type FineMaximumStep = u8;
type Fine = u8;
type Coarse = u8;

pub trait LoopMode: Sealed {}

pub struct OpenLoop {
    // TODO: Add support for custom fine and coarse? Otherwise remove it.
    #[allow(dead_code)]
    fine: Option<Fine>,
    #[allow(dead_code)]
    coarse: Option<Coarse>,
}
impl LoopMode for OpenLoop {}
impl Sealed for OpenLoop {}
pub struct ClosedLoop<T: PclkSourceType> {
    reference_clk: Pclk<Dfll48, T>,
    coarse_maximum_step: CoarseMaximumStep,
    fine_maximum_step: FineMaximumStep,
}
impl<T: PclkSourceType> Sealed for ClosedLoop<T> {}
impl<T: PclkSourceType> LoopMode for ClosedLoop<T> {}

pub struct DfllConfig<TMode: LoopMode> {
    token: DfllToken,
    freq: Hertz,
    mode: TMode,
    multiplication_factor: MultiplicationFactor,
    // TODO: Add support for standby and on-demand mode.
    #[allow(dead_code)]
    standby_sleep_mode: bool,
    #[allow(dead_code)]
    on_demand_mode: bool,
}

impl<TMode: LoopMode> DfllConfig<TMode> {
    pub fn freq(&self) -> Hertz {
        Hertz(self.freq.0 * self.multiplication_factor as u32)
    }
    pub fn set_standby_sleep_mode(&mut self, value: bool) {
        self.standby_sleep_mode = value;
    }
    pub fn set_on_demand_mode(&mut self, value: bool) {
        self.on_demand_mode = value;
    }
}

impl DfllConfig<OpenLoop> {
    pub fn in_open_mode(token: DfllToken) -> DfllConfig<OpenLoop> {
        Self {
            token,
            freq: 48.mhz().into(),
            mode: OpenLoop {
                fine: None,
                coarse: None,
            },
            multiplication_factor: 1_u16,
            standby_sleep_mode: false,
            on_demand_mode: false,
        }
    }
    pub fn enable(mut self) -> Counted<Dfll<OpenLoop>, Zero> {
        self.token.set_open_mode();
        self.token.enable();
        Counted::new(Dfll::new(self))
    }
    pub fn free(self) -> DfllToken {
        self.token
    }
}

impl<T: PclkSourceType> DfllConfig<ClosedLoop<T>> {
    pub fn in_closed_mode(
        token: DfllToken,
        reference_clk: Pclk<Dfll48, T>,
        multiplication_factor: MultiplicationFactor,
        coarse_maximum_step: CoarseMaximumStep,
        fine_maximum_step: FineMaximumStep,
    ) -> DfllConfig<ClosedLoop<T>> {
        Self {
            token,
            freq: reference_clk.freq(),
            mode: ClosedLoop {
                reference_clk,
                coarse_maximum_step,
                fine_maximum_step,
            },
            multiplication_factor,
            standby_sleep_mode: false,
            on_demand_mode: false,
        }
    }
    pub fn set_multiplication_factor(&mut self, multiplication_factor: MultiplicationFactor) {
        self.multiplication_factor = multiplication_factor;
    }
    pub fn set_coarse_maximum_step(&mut self, coarse_maximum_step: CoarseMaximumStep) {
        self.mode.coarse_maximum_step = coarse_maximum_step;
    }
    pub fn set_fine_maximum_step(&mut self, fine_maximum_step: FineMaximumStep) {
        self.mode.fine_maximum_step = fine_maximum_step;
    }
    pub fn enable(mut self) -> Counted<Dfll<ClosedLoop<T>>, Zero> {
        self.token
            .set_fine_maximum_step(self.mode.fine_maximum_step);
        self.token
            .set_coarse_maximum_step(self.mode.coarse_maximum_step);
        self.token
            .set_multiplication_factor(self.multiplication_factor);
        self.token.set_closed_mode();
        Counted::new(Dfll::new(self))
    }
    pub fn free(self) -> (DfllToken, Pclk<Dfll48, T>) {
        (self.token, self.mode.reference_clk)
    }
}

/// TODO
pub struct Dfll<TMode: LoopMode> {
    #[allow(dead_code)]
    config: DfllConfig<TMode>,
}

impl<TMode: LoopMode> Dfll<TMode> {
    fn new(config: DfllConfig<TMode>) -> Self {
        Dfll { config }
    }

    fn disable(self) -> DfllConfig<TMode> {
        // TODO: Disable Dfll
        self.config
    }

    pub fn freq(&self) -> Hertz {
        self.config.freq()
    }
}

impl Dfll<OpenLoop> {
    /// TODO
    #[inline]
    pub(crate) unsafe fn init() -> Self {
        Dfll::new(DfllConfig::in_open_mode(DfllToken::new()))
    }
}

impl<TMode: LoopMode> Sealed for Dfll<TMode> {}

impl<TMode: LoopMode> Counted<Dfll<TMode>, Zero> {
    /// TODO
    #[inline]
    pub fn disable(self) -> DfllConfig<TMode> {
        self.0.disable()
    }
}

//==============================================================================
// GclkSource
//==============================================================================

impl<G: GenNum, N: Count> GclkSource<G> for Counted<Dfll<OpenLoop>, N> {
    type Type = marker::Dfll<OpenLoop>;
    #[inline]
    fn freq(&self) -> Hertz {
        self.0.freq()
    }
}

impl<G: GenNum, T: PclkSourceType, N: Count> GclkSource<G> for Counted<Dfll<ClosedLoop<T>>, N> {
    type Type = marker::Dfll<marker::ClosedLoop>;
    #[inline]
    fn freq(&self) -> Hertz {
        self.0.freq()
    }
}

pub mod marker {
    use super::{GclkSourceEnum, GclkSourceType, Sealed};
    use core::marker::PhantomData;

    pub trait ModeMarker: Sealed {}
    /// TODO
    /// super::ClosedLoop type is polluted with a generic parameter describing
    /// reference clock. It is undesirable to have a marker type owned by a
    /// Gclk that knows about source of its source.
    /// This is a reason for existence of this type.
    pub struct ClosedLoop {
        __: (),
    }

    impl Sealed for ClosedLoop {}
    impl ModeMarker for ClosedLoop {}

    impl ModeMarker for super::OpenLoop {}

    pub struct Dfll<T: ModeMarker> {
        __: PhantomData<T>,
    }

    impl<T: ModeMarker> Sealed for Dfll<T> {}

    impl<T: ModeMarker> GclkSourceType for Dfll<T> {
        const GCLK_SRC: GclkSourceEnum = GclkSourceEnum::DFLL;
    }
}
