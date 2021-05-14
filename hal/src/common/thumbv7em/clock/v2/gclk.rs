//! TODO

use core::marker::PhantomData;

use num_traits::AsPrimitive;
use seq_macro::seq;

use crate::pac;
use crate::pac::NVMCTRL;

pub use crate::pac::gclk::genctrl::SRC_A as GclkSourceEnum;
pub use crate::pac::gclk::{RegisterBlock, GENCTRL};

use crate::time::Hertz;
use crate::typelevel::counted::Counted;
use crate::typelevel::{Count, Decrement, Increment, Is, One, Sealed, Zero};

use crate::clock::v2::pclk::{Dfll48, Pclk, PclkSourceType};
use crate::clock::v2::sources::dfll::{marker, ClosedLoop, Dfll, OpenLoop};

//==============================================================================
// Registers
//==============================================================================

/// TODO
pub type GclkToken<G> = Registers<G>;

/// TODO
pub struct Registers<G: GenNum> {
    gen: PhantomData<G>,
}

impl<G: GenNum> Registers<G> {
    /// TODO
    #[inline]
    unsafe fn new() -> Self {
        Registers { gen: PhantomData }
    }

    #[inline]
    fn mask(&self) -> u16 {
        1 << G::NUM
    }

    #[inline]
    fn gclk(&self) -> &RegisterBlock {
        unsafe { &*pac::GCLK::ptr() }
    }

    /// TODO
    #[inline]
    fn genctrl(&self) -> &GENCTRL {
        &self.gclk().genctrl[G::NUM]
    }

    /// TODO
    #[inline]
    fn wait_syncbusy(&self) {
        while self.gclk().syncbusy.read().genctrl().bits() & self.mask() != 0 {}
    }

    /// TODO
    #[inline]
    fn set_source(&mut self, variant: GclkSourceEnum) {
        self.genctrl().modify(|_, w| w.src().variant(variant));
        self.wait_syncbusy();
    }

    /// TODO
    #[inline]
    fn set_div(&mut self, div: Div<G>) {
        match div {
            Div::Div(div) => {
                self.genctrl().modify(|_, w| unsafe {
                    w.divsel().div1();
                    w.div().bits(div.as_())
                });
            }
            Div::Max => {
                self.genctrl().modify(|_, w| unsafe {
                    w.divsel().div2();
                    w.div().bits(0)
                });
            }
        }
        self.wait_syncbusy();
    }

    /// TODO
    #[inline]
    fn improve_duty_cycle(&mut self, flag: bool) {
        self.genctrl().modify(|_, w| w.idc().bit(flag));
    }

    /// TODO
    #[inline]
    fn enable_gclk_out(&mut self, pol: bool) {
        self.genctrl().modify(|_, w| {
            w.oe().set_bit();
            w.oov().bit(pol)
        });
        self.wait_syncbusy();
    }

    /// TODO
    #[inline]
    fn disable_gclk_out(&mut self) {
        self.genctrl().modify(|_, w| w.oe().clear_bit());
        self.wait_syncbusy();
    }

    /// TODO
    #[inline]
    fn enable(&mut self) {
        self.genctrl().modify(|_, w| w.genen().set_bit());
        self.wait_syncbusy();
    }

    /// TODO
    #[inline]
    fn disable(&mut self) {
        self.genctrl().modify(|_, w| w.genen().clear_bit());
        self.wait_syncbusy();
    }
}

//==============================================================================
// GenNum
//==============================================================================

/// TODO
pub trait GenNum: Sealed {
    const NUM: usize;
    type Div: Copy + AsPrimitive<u16> + AsPrimitive<u32>;
    const DIV_MAX: u32;
}

/// TODO
pub trait NotGen0: GenNum {}

/// TODO
pub enum Gen0 {}
impl Sealed for Gen0 {}
impl GenNum for Gen0 {
    const NUM: usize = 0;
    type Div = u8;
    const DIV_MAX: u32 = 512;
}

/// TODO
pub enum Gen1 {}
impl Sealed for Gen1 {}
impl NotGen0 for Gen1 {}
impl GenNum for Gen1 {
    const NUM: usize = 1;
    type Div = u16;
    const DIV_MAX: u32 = 131072;
}

seq!(N in 2..=11 {
    /// TODO
    pub enum Gen#N {}
    impl Sealed for Gen#N {}
    impl NotGen0 for Gen#N {}
    impl GenNum for Gen#N {
        const NUM: usize = N;
        type Div = u8;
        const DIV_MAX: u32 = 512;
    }
});

//==============================================================================
// Div
//==============================================================================

/// TODO
/// Represents a generator divider. The division factor is a u8 or u16 value,
/// depending on the generator. Generator 1 accepts a u16, while all others
/// accept a u8. The upper bits of the `Div` variant are ignored for generators
/// other than Generator 1. The `DIVSEL` field can be used to boost the division
/// factor to a single value above the normal range. Use the `Max` variant to
/// set the `DIVSEL` field appropriately. See the datasheet for more details.
pub enum Div<G: GenNum> {
    Div(G::Div),
    Max,
}

impl<G: GenNum> Clone for Div<G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<G: GenNum> Copy for Div<G> {}

impl<G: GenNum> Div<G> {
    pub fn as_u32(&self) -> u32 {
        match self {
            Div::Div(div) => div.as_(),
            Div::Max => G::DIV_MAX,
        }
    }
}

//==============================================================================
// GclkSource
//==============================================================================

/// TODO
pub trait GclkSourceType: Sealed {
    const GCLK_SRC: GclkSourceEnum;
}

/// TODO
pub trait GclkSource<G: GenNum>: Sealed {
    type Type: GclkSourceType;
    fn freq(&self) -> Hertz;
}

//==============================================================================
// GclkConfig
//==============================================================================

/// TODO
pub struct GclkConfig<G, T>
where
    G: GenNum,
    T: GclkSourceType,
{
    token: GclkToken<G>,
    src: PhantomData<T>,
    freq: Hertz,
    div: u32,
}

impl GclkConfig<Gen0, marker::Dfll<OpenLoop>> {
    unsafe fn init(freq: impl Into<Hertz>) -> Self {
        let token = GclkToken::new();
        GclkConfig {
            token,
            src: PhantomData,
            freq: freq.into(),
            div: 1,
        }
    }
}

impl<G, T> GclkConfig<G, T>
where
    G: GenNum,
    T: GclkSourceType,
{
    /// TODO
    #[inline]
    pub fn new<S>(mut token: GclkToken<G>, source: S) -> (GclkConfig<G, T>, S::Inc)
    where
        S: GclkSource<G, Type = T> + Increment,
    {
        let freq = source.freq();
        let div = 1;
        token.set_source(T::GCLK_SRC);
        let config = GclkConfig {
            token,
            src: PhantomData,
            freq,
            div,
        };
        (config, source.inc())
    }
}

impl<G, T> GclkConfig<G, T>
where
    G: GenNum,
    T: GclkSourceType,
{
    /// TODO
    #[inline]
    pub fn free<S>(self, source: S) -> (GclkToken<G>, S::Dec)
    where
        S: GclkSource<G, Type = T> + Decrement,
    {
        (self.token, source.dec())
    }
}

impl<G, T> GclkConfig<G, T>
where
    G: GenNum,
    T: GclkSourceType,
{
    /// TODO
    #[inline]
    pub fn swap<Old, New>(
        self,
        old: Old,
        new: New,
    ) -> (GclkConfig<G, New::Type>, Old::Dec, New::Inc)
    where
        Old: GclkSource<G, Type = T> + Decrement,
        New: GclkSource<G> + Increment,
    {
        let (token, old) = self.free(old);
        let (config, new) = GclkConfig::new(token, new);
        (config, old, new)
    }

    /// TODO
    #[inline]
    pub fn div(mut self, div: Div<G>) -> Self {
        self.token.set_div(div);
        self.div = div.as_u32();
        self
    }

    /// TODO
    #[inline]
    pub fn improve_duty_cycle(mut self, flag: bool) -> Self {
        self.token.improve_duty_cycle(flag);
        self
    }

    /// TODO
    #[inline]
    pub fn freq(&self) -> Hertz {
        Hertz(self.freq.0 / self.div)
    }

    /// TODO
    #[inline]
    pub fn enable(mut self) -> Counted<Gclk<G, T>, Zero> {
        self.token.enable();
        Counted::new(Gclk::create(self))
    }
}

//==============================================================================
// Gclk
//==============================================================================

/// TODO
pub struct Gclk<G, T>
where
    G: GenNum,
    T: GclkSourceType,
{
    config: GclkConfig<G, T>,
}

impl Gclk0<marker::Dfll<OpenLoop>> {
    pub(super) unsafe fn init(freq: impl Into<Hertz>) -> Self {
        Gclk::create(GclkConfig::init(freq))
    }
}

impl<G, T> Gclk<G, T>
where
    G: GenNum,
    T: GclkSourceType,
{
    /// TODO
    #[inline]
    fn disable(mut self) -> GclkConfig<G, T> {
        self.config.token.disable();
        self.config
    }
}

impl<G, T> Counted<Gclk<G, T>, Zero>
where
    G: GenNum,
    T: GclkSourceType,
{
    /// TODO
    #[inline]
    fn disable(self) -> GclkConfig<G, T> {
        self.0.disable()
    }
}

impl<T: GclkSourceType> Counted<Gclk<Gen0, T>, One> {
    /// TODO
    #[inline]
    pub unsafe fn swap<Old, New>(
        self,
        old: Old,
        new: New,
    ) -> (Counted<Gclk<Gen0, New::Type>, One>, Old::Dec, New::Inc)
    where
        Old: GclkSource<Gen0, Type = T> + Decrement,
        New: GclkSource<Gen0> + Increment,
    {
        let (config, old, new) = self.0.config.swap(old, new);
        (Counted::new_unsafe(Gclk::create(config)), old, new)
    }
}

impl Counted<Gclk<Gen0, marker::Dfll<OpenLoop>>, One> {
    pub unsafe fn change_mode<T: PclkSourceType>(
        self,
        old: Counted<Dfll<OpenLoop>, One>,
        exchange: impl FnOnce(Counted<Dfll<OpenLoop>, Zero>) -> Counted<Dfll<ClosedLoop<T>>, Zero>,
    ) -> (
        Counted<Gclk<Gen0, marker::Dfll<marker::ClosedLoop>>, One>,
        Counted<Dfll<ClosedLoop<T>>, One>,
    ) {
        let (token, old) = self.0.config.free(old);
        let new = exchange(old);
        let (config, new) = GclkConfig::new(token, new);
        (Counted::new_unsafe(Gclk::create(config)), new)
    }
}

impl Counted<Gclk<Gen0, marker::Dfll<marker::ClosedLoop>>, One> {
    pub unsafe fn change_mode<T: PclkSourceType>(
        self,
        old: Counted<Dfll<ClosedLoop<T>>, One>,
        exchange: impl FnOnce(
            Counted<Dfll<ClosedLoop<T>>, Zero>,
        ) -> (Counted<Dfll<OpenLoop>, Zero>, Pclk<Dfll48, T>),
    ) -> (
        Counted<Gclk<Gen0, marker::Dfll<OpenLoop>>, One>,
        Counted<Dfll<OpenLoop>, One>,
        Pclk<Dfll48, T>,
    ) {
        let (token, old) = self.0.config.free(old);
        let (new, pclk) = exchange(old);
        let (config, new) = GclkConfig::new(token, new);
        (Counted::new_unsafe(Gclk::create(config)), new, pclk)
    }
}

impl<G, T> Gclk<G, T>
where
    G: GenNum,
    T: GclkSourceType,
{
    #[inline]
    fn create(config: GclkConfig<G, T>) -> Self {
        Gclk { config }
    }

    /// TODO
    #[inline]
    pub unsafe fn disable_unchecked(mut self) -> GclkConfig<G, T> {
        self.config.token.disable();
        self.config
    }

    /// TODO
    #[inline]
    pub unsafe fn div(&mut self, div: Div<G>) {
        self.config.token.set_div(div);
        self.config.div = div.as_u32();
    }

    /// TODO
    #[inline]
    pub unsafe fn improve_duty_cycle(&mut self, flag: bool) {
        self.config.token.improve_duty_cycle(flag);
    }

    /// TODO
    #[inline]
    pub fn freq(&self) -> Hertz {
        self.config.freq()
    }

    /// TODO
    #[inline]
    pub(super) fn enable_gclk_out(&mut self, pol: bool) {
        self.config.token.enable_gclk_out(pol);
    }

    /// TODO
    #[inline]
    pub(super) fn disable_gclk_out(&mut self) {
        self.config.token.disable_gclk_out();
    }
}

//==============================================================================
// Gclk aliases
//==============================================================================

seq!(G in 0..=11 {
    /// TODO
    pub type Gclk#G<S> = Gclk<Gen#G, S>;
});

//==============================================================================
// Gclk1 SourceType
//==============================================================================

impl GclkSourceType for Gen1 {
    const GCLK_SRC: GclkSourceEnum = GclkSourceEnum::GCLKGEN1;
}

macro_rules! impl_gclk1_source {
    ($GenNum:ident) => {
        impl<T, N> GclkSource<$GenNum> for Counted<Gclk1<T>, N>
        where
            T: GclkSourceType,
            N: Count,
        {
            type Type = Gen1;

            #[inline]
            fn freq(&self) -> Hertz {
                self.0.freq()
            }
        }
    };
}

impl_gclk1_source!(Gen0);

seq!(N in 2..=11 {
    impl_gclk1_source!(Gen#N);
});

//==============================================================================
// AnyGclk
//==============================================================================

pub trait AnyGclk
where
    Self: Sealed,
    Self: Is<Type = SpecificGclk<Self>>,
{
    /// TODO
    type GenNum: GenNum;

    /// TODO
    type Source: GclkSourceType;
}

/// TODO
impl<G, T> Sealed for Gclk<G, T>
where
    G: GenNum,
    T: GclkSourceType,
{
}

pub type SpecificGclk<G> = Gclk<<G as AnyGclk>::GenNum, <G as AnyGclk>::Source>;

impl<G: AnyGclk> AsRef<G> for SpecificGclk<G> {
    fn as_ref(&self) -> &G {
        // Always a NOP, since G == SpecificGclk<G>
        unsafe { core::mem::transmute(self) }
    }
}

impl<G: AnyGclk> AsMut<G> for SpecificGclk<G> {
    fn as_mut(&mut self) -> &mut G {
        // Always a NOP, since G == SpecificGclk<G>
        unsafe { core::mem::transmute(self) }
    }
}

impl<G, T> AnyGclk for Gclk<G, T>
where
    G: GenNum,
    T: GclkSourceType,
{
    type GenNum = G;
    type Source = T;
}

//==============================================================================
// Gclks
//==============================================================================

seq!(N in 2..=11 {
    /// TODO
    pub struct Tokens {
        pub gclk1: GclkToken<Gen1>,
        #( pub gclk#N: GclkToken<Gen#N>, )*
    }

    impl Tokens {
        pub(super) fn new(nvmctrl: &mut NVMCTRL) -> Self {
            // Use auto wait states
            nvmctrl.ctrla.modify(|_, w| w.autows().set_bit());
            // TODO
            unsafe {
                Tokens {
                    gclk1: GclkToken::new(),
                    #( gclk#N: GclkToken::new(), )*
                }
            }
        }
    }
});
