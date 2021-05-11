//! TODO

use core::marker::PhantomData;

use num_traits::AsPrimitive;
use seq_macro::seq;

use crate::pac;
use crate::pac::NVMCTRL;

pub use crate::pac::gclk::genctrl::SRC_A as GclkSourceEnum;
pub use crate::pac::gclk::{RegisterBlock, GENCTRL};

use crate::time::Hertz;
use crate::typelevel::{Count, Decrement, Increment, Is, Lockable, One, Sealed, Unlockable, Zero};

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
            // Maximum reach of DIV1 mode is 255 or 65535
            Div::Div(div) => {
                self.genctrl().modify(|_, w| unsafe {
                    w.div().bits(div.as_());
                    w.divsel().div1()
                });
            }
            // Maximum reach of DIV2 mode is 512 or 131072
            Div::DivPow2(div) => {
                self.genctrl().modify(|_, w| unsafe {
                    w.div().bits(div.as_());
                    w.divsel().div2()
                });
            }
            // Maximum division value 
            // Division factor: 2.pow(1 + MAX - 1) = 256 or 65536
            Div::MaxMinusOne => {
                self.genctrl().modify(|_, w| unsafe {
                    match G::NUM {
                        1 =>  w.div().bits(16 - 1),
                        _ => w.div().bits(8 - 1)
                    };
                    // To reach the maximum divider minus one divsel DIV2 mode is required
                    w.divsel().div2()
                });
            }
            // Maximum division value 
            // Division factor: 2.pow(1 + MAX) = 512 or 131072
            Div::Max => {
                self.genctrl().modify(|_, w| unsafe {
                    match G::NUM {
                        1 =>  w.div().bits(16),
                        _ => w.div().bits(8)
                    };
                    // To reach the maximum divider divsel DIV2 mode is required
                    w.divsel().div2()
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
    const DIV_MAX_MINUS_ONE: u32;
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
    const DIV_MAX_MINUS_ONE: u32 = 256;
    const DIV_MAX: u32 = 512;
}

/// TODO
pub enum Gen1 {}
impl Sealed for Gen1 {}
impl NotGen0 for Gen1 {}
impl GenNum for Gen1 {
    const NUM: usize = 1;
    type Div = u16;
    const DIV_MAX_MINUS_ONE: u32 = 65536;
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
        const DIV_MAX_MINUS_ONE: u32 = 256;
        const DIV_MAX: u32 = 512;
    }
});

//==============================================================================
// Div
//==============================================================================

/// TODO
/// Represents a generator divider.
///
/// Division is interpreted differently depending on state of `DIVSEL` flag.
///
/// In `DIVSEL` mode `DIV1` (value 0) the division factor is directly interpreted from
/// the `DIV` register.  The division factor is a u8 or u16 value,
/// depending on the generator. Generator 1 accepts a u16, while all others
/// accept a u8. The upper bits of the `Div` variant are ignored for generators
/// other than Generator 1. 
///
/// In `DIVSEL` mode `DIV2` (value 1) the division factor is calculated as
///
/// ```
/// division_factor = 2.pow(1 + DIV_register)
/// ```
///
/// The maximum division factor for both modes are 131072 for `Gclk` 1 and 512 for
/// all others.
///
/// `DIVSEL` mode `DIV2` is able to reach this maximum division factor value by setting
/// `DIV` to 8 or 16, since 2.pow(1 + 8) = 512, 2.pow(1 + 16) = 131072 for `Gclk` 1.
/// `DIVSEL` mode `DIV1` is limited to 65535 for `Gclk` 1 and 255 for all others.
///
/// See the datasheet for more details.
pub enum Div<G: GenNum> {
    /// For `Gclk` 0 and 2..11: maximum divison 255. `Gclk` 1: 65535
    /// Using `DIVSEL` mode `DIV1`
    Div(G::Div),
    /// For `Gclk` 0 and 2..11: maximum divison 512. `Gclk` 1: 131072
    /// Using `DIVSEL` mode `DIV2`
    DivPow2(G::Div),
    /// `Gclk` 0,2..11: Division 2.pow(1+8-1) = 256
    /// `Gclk` 1: Division 2.pow(1+16-1) = 65536
    MaxMinusOne,
    /// `Gclk` 0,2..11: Division 2.pow(1+8) = 512
    /// `Gclk` 1: Division 2.pow(1+16) = 131072
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
            Div::DivPow2(div) => div.as_(),
            Div::MaxMinusOne => G::DIV_MAX_MINUS_ONE,
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
    divsel: bool,
}

impl GclkConfig<Gen0, marker::Dfll<OpenLoop>> {
    unsafe fn init(freq: impl Into<Hertz>) -> Self {
        let token = GclkToken::new();
        GclkConfig {
            token,
            src: PhantomData,
            freq: freq.into(),
            div: 1,
            divsel: false,
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
    pub fn new<S>(mut token: GclkToken<G>, source: S) -> (GclkConfig<G, T>, S::Locked)
    where
        S: GclkSource<G, Type = T> + Lockable,
    {
        let freq = source.freq();
        let div = 1;
        let divsel = false;
        token.set_source(T::GCLK_SRC);
        let config = GclkConfig {
            token,
            src: PhantomData,
            freq,
            div,
            divsel,
        };
        (config, source.lock())
    }
}

impl<G, T> GclkConfig<G, T>
where
    G: GenNum,
    T: GclkSourceType,
{
    /// TODO
    #[inline]
    pub fn free<S>(self, source: S) -> (GclkToken<G>, S::Unlocked)
    where
        S: GclkSource<G, Type = T> + Unlockable,
    {
        (self.token, source.unlock())
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
    ) -> (GclkConfig<G, New::Type>, Old::Unlocked, New::Locked)
    where
        Old: GclkSource<G, Type = T> + Unlockable,
        New: GclkSource<G> + Lockable,
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
    /// Calculate the frequency of the `Gclk`
    ///
    /// The frequency differ dependent on the value of `DIVSEL`,
    /// see [Div]
    ///
    ///
    #[inline]
    pub fn freq(&self) -> Hertz {
        match self.divsel {
            false => {
                // Handle the allowed case with DIV-field set to zero
                match self.div {
                    0 => Hertz(self.freq.0),
                    _ => Hertz(self.freq.0 / self.div)
                }
            },
            true => {
                Hertz(self.freq.0 / 2_u32.pow(1 + self.div))
            },
        }
    }

    /// TODO
    #[inline]
    pub fn enable(mut self) -> Gclk<G, T> {
        self.token.enable();
        Gclk::create(self, Zero::new())
    }
}

//==============================================================================
// Gclk
//==============================================================================

/// TODO
pub struct Gclk<G, T, N = Zero>
where
    G: GenNum,
    T: GclkSourceType,
    N: Count,
{
    config: GclkConfig<G, T>,
    count: N,
}

impl Gclk0<marker::Dfll<OpenLoop>, One> {
    pub(super) unsafe fn init(freq: impl Into<Hertz>) -> Self {
        let config = GclkConfig::init(freq);
        let count = One::new();
        Gclk::create(config, count)
    }
}

impl<G, T> Gclk<G, T>
where
    G: NotGen0,
    T: GclkSourceType,
{
    /// TODO
    #[inline]
    pub fn disable(mut self) -> GclkConfig<G, T> {
        self.config.token.disable();
        self.config
    }
}

impl<T: GclkSourceType> Gclk<Gen0, T, One> {
    /// TODO
    #[inline]
    pub unsafe fn swap<Old, New>(
        self,
        old: Old,
        new: New,
    ) -> (Gclk<Gen0, New::Type, One>, Old::Unlocked, New::Locked)
    where
        Old: GclkSource<Gen0, Type = T> + Unlockable,
        New: GclkSource<Gen0> + Lockable,
    {
        let (config, old, new) = self.config.swap(old, new);
        (Gclk::create(config, self.count), old, new)
    }
}

impl Gclk<Gen0, marker::Dfll<OpenLoop>, One> {
    pub unsafe fn change_mode<T: PclkSourceType>(
        self,
        old: Dfll<OpenLoop, One>,
        exchange: impl FnOnce(Dfll<OpenLoop>) -> Dfll<ClosedLoop<T>>,
    ) -> (
        Gclk<Gen0, marker::Dfll<marker::ClosedLoop>, One>,
        Dfll<ClosedLoop<T>, One>,
    ) {
        let (token, old) = self.config.free(old);
        let new = exchange(old);
        let (config, new) = GclkConfig::new(token, new);
        (Gclk::create(config, self.count), new)
    }
}

impl Gclk<Gen0, marker::Dfll<marker::ClosedLoop>, One> {
    pub unsafe fn change_mode<T: PclkSourceType>(
        self,
        old: Dfll<ClosedLoop<T>, One>,
        exchange: impl FnOnce(Dfll<ClosedLoop<T>>) -> (Dfll<OpenLoop>, Pclk<Dfll48, T>),
    ) -> (
        Gclk<Gen0, marker::Dfll<OpenLoop>, One>,
        Dfll<OpenLoop, One>,
        Pclk<Dfll48, T>,
    ) {
        let (token, old) = self.config.free(old);
        let (new, pclk) = exchange(old);
        let (config, new) = GclkConfig::new(token, new);
        (Gclk::create(config, self.count), new, pclk)
    }
}

impl<G, T, N> Gclk<G, T, N>
where
    G: GenNum,
    T: GclkSourceType,
    N: Count,
{
    #[inline]
    fn create(config: GclkConfig<G, T>, count: N) -> Self {
        Gclk { config, count }
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
// Lockable
//==============================================================================

impl<G, T, N> Lockable for Gclk<G, T, N>
where
    G: GenNum,
    T: GclkSourceType,
    N: Increment,
{
    type Locked = Gclk<G, T, N::Inc>;
    fn lock(self) -> Self::Locked {
        Gclk::create(self.config, self.count.inc())
    }
}

//==============================================================================
// Unlockable
//==============================================================================

impl<G, T, N> Unlockable for Gclk<G, T, N>
where
    G: GenNum,
    T: GclkSourceType,
    N: Decrement,
{
    type Unlocked = Gclk<G, T, N::Dec>;
    fn unlock(self) -> Self::Unlocked {
        Gclk::create(self.config, self.count.dec())
    }
}

//==============================================================================
// Gclk aliases
//==============================================================================

seq!(G in 0..=11 {
    /// TODO
    pub type Gclk#G<S, N> = Gclk<Gen#G, S, N>;
});

//==============================================================================
// Gclk1 SourceType
//==============================================================================

impl GclkSourceType for Gen1 {
    const GCLK_SRC: GclkSourceEnum = GclkSourceEnum::GCLKGEN1;
}

macro_rules! impl_gclk1_source {
    ($GenNum:ident) => {
        impl<T, N> GclkSource<$GenNum> for Gclk1<T, N>
        where
            T: GclkSourceType,
            N: Count,
        {
            type Type = Gen1;

            #[inline]
            fn freq(&self) -> Hertz {
                self.freq()
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

    /// TODO
    type Count: Count;
}

/// TODO
pub type SpecificGclk<G> =
    Gclk<<G as AnyGclk>::GenNum, <G as AnyGclk>::Source, <G as AnyGclk>::Count>;

impl<G, T, N> Sealed for Gclk<G, T, N>
where
    G: GenNum,
    T: GclkSourceType,
    N: Count,
{
}

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

impl<G, T, N> AnyGclk for Gclk<G, T, N>
where
    G: GenNum,
    T: GclkSourceType,
    N: Count,
{
    type GenNum = G;
    type Source = T;
    type Count = N;
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
