use core::marker::PhantomData;

use bitfield::*;

use typenum::U0;

use crate::gpio::v2::{Interrupt, InterruptConfig, Pin};
// Copied from crate::clock::v2::types, just importing from
// there causes cargo doc to combine clocking and EIC
// This needs revisiting
use crate::clock::types::{
    Counter as ClockCounter, Decrement as ClockDecrement, Enabled as ClockEnabled,
    Increment as ClockIncrement,
};
use types::{Counter, Enabled, PrivateDecrement, PrivateIncrement};

use crate::eic::v2::*;

use super::extint::*;

//==============================================================================
// Clock
//==============================================================================

// Synchronous vs. asynchronous detection
/// Trait describing which clock-source the [`EIController`]
/// can use
///
/// Either with an external clock source, or without
///
/// Supported external clock sources are:
///
/// * [`CLK_GCLK`][crate::clock::v2::gclk]
/// * [`CLK_ULP32K`][crate::clock::v2::osculp32k]
///
/// This clock selection is written to hardware,
/// `EIC->CTRLA->CKSEL`
/// TODO
pub trait Clock: Sealed {}

/// Only allows asynchronous detection
pub struct WithoutClock {}
impl Sealed for WithoutClock {}
impl Clock for WithoutClock {}

// When in WithClock, we have to store a clock resource
/// This mode allows full EIC functionality
///
/// Required if:
/// * The NMI is using edge detection or filtering
/// * One EXTINT uses filtering
/// * One EXTINT uses synchronous edge detection
/// * One EXTINT uses debouncing
pub trait WithClock<K>: Sealed {}

/// TODO
pub struct Osc32kDriven<C: EIClkSrcMarker> {
    /// Clock resource
    reference_clk: PhantomData<C>,
}
impl<C: EIClkSrcMarker> Sealed for Osc32kDriven<C> {}
impl<C: EIClkSrcMarker> Clock for Osc32kDriven<C> {}
impl<C: EIClkSrcMarker> WithClock<Osc32kDriven<C>> for Osc32kDriven<C> {}

pub struct PclkDriven<T>
where
    T: PclkSourceMarker,
{
    /// Clock resource
    reference_clk: Pclk<Eic, T>,
}
impl<T: PclkSourceMarker> Sealed for PclkDriven<T> {}
impl<T: PclkSourceMarker> Clock for PclkDriven<T> {}
impl<T: PclkSourceMarker> WithClock<PclkDriven<T>> for PclkDriven<T> {}

/// Type class for all possible [`Clock`] types
///
/// This trait uses the [`AnyKind`] trait pattern to create a [type class] for
/// [`Clock`] types. See the `AnyKind` documentation for more details on the
/// pattern.
///
/// [`AnyKind`]: crate::typelevel#anykind-trait-pattern
/// [type class]: crate::typelevel#type-classes
pub trait AnyClock: Sealed + Is<Type = SpecificClock<Self>> {
    type Mode: Clock;
    type ClockSource: EIClkSrcMarker;
}

/// Type alias for extracting a specific clock from [`AnyClock`]
pub type SpecificClock<K> = <K as AnyClock>::Mode;

impl AnyClock for WithoutClock {
    type Mode = WithoutClock;
    type ClockSource = NoneT;
}

impl AsRef<Self> for WithoutClock {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<Self> for WithoutClock {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<CS> AnyClock for Osc32kDriven<CS>
where
    CS: EIClkSrcMarker,
{
    type Mode = Osc32kDriven<CS>;
    type ClockSource = CS;
}

impl<CS: EIClkSrcMarker> AsRef<Self> for Osc32kDriven<CS> {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<CS: EIClkSrcMarker> AsMut<Self> for Osc32kDriven<CS> {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T> AnyClock for PclkDriven<T>
where
    T: PclkSourceMarker,
{
    type Mode = PclkDriven<T>;
    type ClockSource = T;
}
impl<T: PclkSourceMarker> AsRef<Self> for PclkDriven<T> {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T: PclkSourceMarker> AsMut<Self> for PclkDriven<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

pub trait EIClkSrc: Sealed {
    type Type: EIClkSrcMarker;
}

/// EIController clock source
///
/// See [`Clock`]
pub trait EIClkSrcMarker {
    const CKSEL: CKSEL_A;
}

// Peripheral channel clock, routed from a GCLK
impl<T: PclkSourceMarker> EIClkSrcMarker for Pclk<Eic, T> {
    /// Peripheral channel GCLK_EIC used to clock EIC
    const CKSEL: CKSEL_A = CKSEL_A::CLK_GCLK;
}

// Ultra-low power oscillator can be used instead
impl<Y: Output1k, N: ClockCounter> EIClkSrcMarker for ClockEnabled<OscUlp32k<Active32k, Y>, N> {
    /// Ultra-low power OSCULP32K used to clock EIC
    const CKSEL: CKSEL_A = CKSEL_A::CLK_ULP32K;
}

/*
impl EIClkSrcMarker for NoneT {
    /// Used in the case of [`WithoutClock`]
    ///
    /// This is the default value at reset,
    /// so even without a clock this reflects hardware
    ///
    /// This is a workaround to be able to extract ClockSource
    const CKSEL: CKSEL_A = CKSEL_A::CLK_ULP32K;
}
*/

impl<T> EIClkSrc for Pclk<Eic, T>
where
    T: PclkSourceMarker,
{
    type Type = Pclk<Eic, T>;
}

impl<Y: Output1k, N: ClockCounter> EIClkSrc for ClockEnabled<OscUlp32k<Active32k, Y>, N> {
    type Type = ClockEnabled<OscUlp32k<Active32k, Y>, N>;
}

//==============================================================================
// EIController Enabled / Enable Protect
//==============================================================================

/// "Enable Protect" mode is active
///
/// When `CTRL.ENABLE` is set registers
///
/// * EVCTRL
/// * CONFIG
/// * ASYNCH
/// * DEBOUNCEN
/// * DPRESCALER
///
/// becomes write protected and the EIC is active
///
/// The exception is the NMI-interrupt which becomes
/// active when sense-mode is changed to any other mode
/// than `None`
pub enum Protected {}

/// "Enable Protect" mode is inactive
///
/// When `CTRL.ENABLE` is cleared registers
///
/// * EVCTRL
/// * CONFIG
/// * ASYNCH
/// * DEBOUNCEN
/// * DPRESCALER
///
/// are accessible
pub enum Configurable {}

impl Sealed for Protected {}
impl Sealed for Configurable {}

/// Used to encode EIController "Enable protect" state
///
/// When [`Protected`] the EIC is active and all [`ExtInt`]s
/// acts as configured. When set to [`Configurable`]
/// they are dormant, with the exception of [`NmiExtInt`]
/// which is active whenever [`SenseMode::SENSE`] differ from `None`.
pub trait EnableProtection: Sealed {}

impl EnableProtection for Protected {}
impl EnableProtection for Configurable {}

//==============================================================================
// EIController
//==============================================================================
// Struct to represent the external interrupt controller
// You need exclusive access to this to set registers that
// share multiple pins, like the Sense configuration register
/// Controller interface for External Interrupt Controller (EIC)
///
/// Used to create up to 16 [`ExtInt`] and one [`NmiExtInt`]
///
/// EIController has access to all of EIC registers
///
/// TODO
pub struct EIController<AK, EP>
where
    AK: AnyClock,
    EP: EnableProtection,
{
    eic: crate::pac::EIC,
    clockmode: AK,
    _enablestate: PhantomData<EP>,
}

impl<CS> EIController<Osc32kDriven<CS>, Configurable>
where
    CS: EIClkSrcMarker,
{
    /// Create an EIC Controller with a clock source
    ///
    /// This allows for full EIC functionality
    ///
    /// # Safety
    ///
    /// Safe because you trade a singleton PAC struct for new singletons
    pub fn from_osc32k<S>(
        eic: crate::pac::EIC,
        clock: S,
    ) -> (
        Enabled<EIController<Osc32kDriven<CS>, Configurable>, U0>,
        Tokens,
        S::Inc,
    )
    where
        S: EIClkSrc<Type = CS> + ClockIncrement,
    {
        // Software reset the EIC controller on creation
        eic.ctrla.modify(|_, w| w.swrst().set_bit());
        while eic.syncbusy.read().swrst().bit_is_set() {}

        // Set CKSEL to match the clock resource provided
        eic.ctrla
            .modify(|_, w| w.cksel().variant(CKSEL_A::CLK_ULP32K));

        unsafe {
            (
                Enabled::new(Self {
                    eic,
                    clockmode: Osc32kDriven {
                        reference_clk: PhantomData,
                    },
                    _enablestate: PhantomData,
                }),
                Tokens::new(),
                clock.inc(),
            )
        }
    }
}
impl<T> EIController<PclkDriven<T>, Configurable>
where
    T: PclkSourceMarker,
{
    /// TODO
    pub fn from_pclk(
        eic: crate::pac::EIC,
        reference_clk: Pclk<Eic, T>,
    ) -> (
        Enabled<EIController<PclkDriven<T>, Configurable>, U0>,
        Tokens,
    ) {
        // Software reset the EIC controller on creation
        eic.ctrla.modify(|_, w| w.swrst().set_bit());
        while eic.syncbusy.read().swrst().bit_is_set() {}

        // Set CKSEL to match the clock resource provided
        eic.ctrla
            .modify(|_, w| w.cksel().variant(CKSEL_A::CLK_GCLK));

        unsafe {
            (
                Enabled::new(Self {
                    eic,
                    clockmode: PclkDriven { reference_clk },
                    _enablestate: PhantomData,
                }),
                Tokens::new(),
            )
        }
    }
}

impl EIController<WithoutClock, Configurable> {
    /// Create an EIC Controller without a clock source
    ///
    /// This limits the EIC functionality
    ///
    /// # Safety
    ///
    /// Safe because you trade a singleton PAC struct for new singletons
    pub fn new_only_async(
        eic: crate::pac::EIC,
    ) -> (
        Enabled<EIController<WithoutClock, Configurable>, U0>,
        Tokens,
    ) {
        // Software reset the EIC controller on creation
        eic.ctrla.modify(|_, w| w.swrst().set_bit());
        while eic.syncbusy.read().swrst().bit_is_set() {}

        // Setup mode to async for all channels
        // FIXME
        // Is this sensible?
        eic.asynch.write(|w| unsafe { w.bits(0xFFFF) });

        // Does not use or need any external clock, `CKSEL` is ignored

        unsafe {
            (
                Enabled::new(Self {
                    eic,
                    clockmode: WithoutClock {},
                    _enablestate: PhantomData,
                }),
                Tokens::new(),
            )
        }
    }
}

impl<AK> Enabled<EIController<AK, Configurable>, U0>
where
    AK: AnyClock,
{
    /// Software reset needs to be synchronised
    fn syncbusy_swrst(&self) {
        while self.0.eic.syncbusy.read().swrst().bit_is_set() {}
    }
    /// Softare reset the EIC controller
    ///
    /// Will clear all registers and leave the controller disabled
    pub fn swrst(self) -> Self {
        self.0.eic.ctrla.modify(|_, w| w.swrst().set_bit());
        // Wait until done
        self.syncbusy_swrst();
        self
    }
}

/*
impl<AK, EP, N> Enabled<EIController<AK, EP>, N>
where
    AK: AnyClock,
    EP: EnableProtection,
    N: Counter,
{
    /// Enabling the EIC controller needs to be synchronised
    fn syncbusy_enable(&self) {
        while self.0.eic.syncbusy.read().enable().bit_is_set() {}
    }

    fn read_config_reg(&self, index: usize) -> EIConfigReg {
        EIConfigReg(self.0.eic.config[index].read().bits())
    }
}

impl<AK, N> Enabled<EIController<AK, Configurable>, N>
where
    AK: AnyClock,
    N: Counter,
{
    /// Change ExtInt sensemode
    ///
    /// Available modes: see [`Sense`]
    pub(super) fn set_sense_mode<E: EINum>(&self, sense: Sense) {
        let index: usize = E::OFFSET.into();
        let msb: usize = E::SENSEMSB.into();
        let lsb: usize = E::SENSELSB.into();

        // Read the register and parse it as a [`EIConfigReg`]
        let mut config_reg = self.read_config_reg(index);

        // Modify only the relevant part of the configuration
        config_reg.set_bit_range(msb, lsb, sense as u8);

        // Write the configuration state to hardware
        self.0.eic.config[index].write(|w| unsafe { w.bits(config_reg.bit_range(31, 0)) });
    }
    /// Change NmiExtInt sensemode
    ///
    /// Available modes: see [`Sense`]
    pub(super) fn set_sense_mode_nmi(&self, sense: Sense) {
        // Write the configuration state to hardware
        self.0
            .eic
            .nmictrl
            .write(|w| unsafe { w.nmisense().bits(sense as u8) });
    }

    /// Configure Event Output for ExtInt
    ///
    /// Requires that the Event System (EVSYS) peripheral is configured
    pub(super) fn set_event_output<E: EINum>(&self, set_event_output: bool) {
        let val = self.0.eic.evctrl.read().bits();

        let data = match set_event_output {
            true => val | E::MASK as u32,
            false => val & !(E::MASK as u32),
        };

        // Write to hardware
        self.0.eic.evctrl.write(|w| unsafe { w.bits(data) });
    }

    /// Finalize the configuration phase and activate EIC
    ///
    /// This "finalizes" the configuration phase meaning
    /// further modifications are not allowed.
    pub fn finalize(self) -> Enabled<EIController<AK, Protected>, N> {
        self.0.eic.ctrla.modify(|_, w| w.enable().set_bit());
        self.syncbusy_enable();

        Enabled::new(EIController {
            eic: self.0.eic,
            clockmode: self.0.clockmode,
            _enablestate: PhantomData,
        })
    }
}

impl<AK, N> Enabled<EIController<AK, Protected>, N>
where
    AK: AnyClock,
    N: Counter,
{
    /// Disable the EIC and return to a [`Configurable`] state
    pub fn disable(self) -> Enabled<EIController<AK, Configurable>, N> {
        self.0.eic.ctrla.modify(|_, w| w.enable().clear_bit());
        self.syncbusy_enable();

        Enabled::new(EIController {
            eic: self.0.eic,
            clockmode: self.0.clockmode,
            _enablestate: PhantomData,
        })
    }
}

impl<T> Enabled<EIController<PclkDriven<T>, Configurable>, U0>
where
    T: PclkSourceMarker,
{
    /// Disable and destroy the EIC controller
    pub fn destroy(self, _tokens: Tokens) -> (crate::pac::EIC, Pclk<Eic, T>) {
        (self.0.eic, self.reference_clk)
    }
}

impl<CS> Enabled<EIController<Osc32kDriven<CS>, Configurable>, U0>
where
    CS: EIClkSrcMarker,
{
    /// Disable and destroy the EIC controller
    pub fn destroy<S>(self, _tokens: Tokens, clock: S) -> (crate::pac::EIC, S::Dec)
    where
        S: ClockDecrement,
    {
        (self.0.eic, clock.dec())
    }
}

impl Enabled<EIController<WithoutClock, Configurable>, U0> {
    /// Disable and destroy the EIC controller
    pub fn destroy(self, _tokens: Tokens) -> crate::pac::EIC {
        self.0.eic
    }
}

impl<CS, N> Enabled<EIController<WithClock<CS>, Configurable>, N>
where
    CS: EIClkSrcMarker,
    N: Counter + PrivateIncrement,
{
    /// Create an EIController with a clocksource
    ///
    /// Capable of using all ExtInt detection modes and features
    ///
    /// Including:
    ///
    /// * [`Normal`] ExtInt
    /// * [`Debounced`] ExtInt
    /// * [`Filtered`] ExtInt
    /// * Running in [`AsyncOnly`] mode
    pub fn new_sync<I, C>(
        self,
        token: Token<I::EINum>,
        pin: Pin<I, Interrupt<C>>,
    ) -> (
        <Self as PrivateIncrement>::Inc,
        ExtInt<I, C, Normal, WithClock<CS>, SenseNone>,
    )
    where
        I: GetEINum,
        C: InterruptConfig,
    {
        (self.inc(), ExtInt::new_sync(token, pin))
    }
}

impl<K, N> Enabled<EIController<K, Configurable>, N>
where
    K: AnyClock,
    N: Counter + PrivateIncrement,
{
    /// Create an EIController without a clocksource
    ///
    /// Limited capabilities, restricted to only using [`AsyncOnly`] mode
    pub fn new_async_only<I, C>(
        self,
        token: Token<I::EINum>,
        pin: Pin<I, Interrupt<C>>,
    ) -> (
        <Self as PrivateIncrement>::Inc,
        ExtInt<I, C, AsyncOnly, WithoutClock, SenseNone>,
    )
    where
        I: GetEINum,
        C: InterruptConfig,
    {
        (self.inc(), ExtInt::new_async(token, pin))
    }
}

impl<AK, N> Enabled<EIController<AK, Configurable>, N>
where
    AK: AnyClock,
    N: Counter + PrivateDecrement,
{
    /// Disable the ExtInt
    ///
    /// Return the token and GPIO pin
    pub fn disable_ext_int<I, C, AM, AS, AK2>(
        self,
        ext_int: ExtInt<I, C, AM, AK2, AS>,
    ) -> (
        <Self as PrivateDecrement>::Dec,
        Token<I::EINum>,
        Pin<I, Interrupt<C>>,
    )
    where
        I: GetEINum,
        C: InterruptConfig,
        AM: AnyMode,
        AS: AnySenseMode,
        AK2: AnyClock,
    {
        (self.dec(), ext_int.token, ext_int.pin)
    }
}

impl<CS, N> Enabled<EIController<WithClock<CS>, Configurable>, N>
where
    CS: EIClkSrcMarker,
    N: Counter,
{
    // Private function that should be accessed through the ExtInt
    // Could pass the MASK directly instead of making this function
    // generic over the EINum. Either way is fine.
    /// Enable ExtInt Debouncing
    ///
    /// Requires access to a clock-source
    pub(super) fn enable_debouncing<E: EINum>(&self) {
        self.0.eic.debouncen.modify(|r, w| unsafe {
            let bits = r.debouncen().bits();
            w.debouncen().bits(bits | E::MASK)
        });
    }

    /// Disable ExtInt Debouncing
    pub(super) fn disable_debouncing<E: EINum>(&self) {
        self.0.eic.debouncen.modify(|r, w| unsafe {
            let bits = r.debouncen().bits();
            // Clear specific bit
            w.debouncen().bits(bits & !(E::MASK))
        });
    }

    /// Set debouncer settings
    ///
    /// Changes debouncing for ALL ExtInts
    pub fn set_debouncer_settings(&self, settings: &DebouncerSettings) {
        self.0.eic.dprescaler.write({
            |w| {
                w.tickon()
                    .variant(settings.tickon)
                    .prescaler0()
                    .variant(settings.prescaler0)
                    .states0()
                    .variant(settings.states0)
                    .prescaler1()
                    .variant(settings.prescaler1)
                    .states1()
                    .variant(settings.states1)
            }
        });
    }

    /// Enable ExtInt Filtering
    pub(super) fn enable_filtering<E: EINum>(&self) {
        let index: usize = E::OFFSET.into();
        let bitnum: usize = E::FILTEN.into();

        // Read the register and parse it as a [`EIConfigReg`]
        let mut config_reg = self.read_config_reg(index);
        config_reg.set_bit(bitnum, true);

        // Write the configuration state to hardware
        self.0.eic.config[index].write(|w| unsafe { w.bits(config_reg.bit_range(31, 0)) });
    }

    /// Disable ExtInt Filtering
    pub(super) fn disable_filtering<E: EINum>(&self) {
        let index: usize = E::OFFSET.into();
        let bitnum: usize = E::FILTEN.into();

        // Read the register and parse it as a [`EIConfigReg`]
        let mut config_reg = self.read_config_reg(index);
        config_reg.set_bit(bitnum, false);

        // Write the configuration state to hardware
        self.0.eic.config[index].write(|w| unsafe { w.bits(config_reg.bit_range(31, 0)) });
    }

    /// Enable Async Operation
    pub(super) fn enable_async<E: EINum>(&self) {
        let val = self.0.eic.asynch.read().bits();

        // Write to hardware
        self.0
            .eic
            .asynch
            .write(|w| unsafe { w.bits(val | (E::MASK as u32)) });
    }

    /// Disable Async Operation
    pub(super) fn disable_async<E: EINum>(&self) {
        let val = self.0.eic.asynch.read().bits();

        // Write to hardware
        self.0
            .eic
            .asynch
            .write(|w| unsafe { w.bits(val & !(E::MASK as u32)) });
    }
}

impl<CS, N> Enabled<EIController<WithClock<CS>, Configurable>, N>
where
    CS: EIClkSrcMarker,
    N: Counter + PrivateIncrement,
{
    /// Create an ExtIntNmi with a clock source
    pub fn new_sync_nmi<I, C>(
        self,
        token: NmiToken,
        pin: Pin<I, Interrupt<C>>,
    ) -> (
        <Self as PrivateIncrement>::Inc,
        NmiExtInt<I, C, Normal, WithClock<CS>, SenseNone>,
    )
    where
        I: NmiEI,
        C: InterruptConfig,
    {
        (self.inc(), NmiExtInt::new_sync(token, pin))
    }
}

impl<K, N> Enabled<EIController<K, Configurable>, N>
where
    K: AnyClock,
    N: Counter + PrivateIncrement,
{
    /// Create an ExtIntNmi without a clock source
    ///
    /// Limited capabilities, restricted to only using [`AsyncOnly`] mode
    pub fn new_async_only_nmi<I, C>(
        self,
        token: NmiToken,
        pin: Pin<I, Interrupt<C>>,
    ) -> (
        <Self as PrivateIncrement>::Inc,
        NmiExtInt<I, C, AsyncOnly, WithoutClock, SenseNone>,
    )
    where
        I: NmiEI,
        C: InterruptConfig,
    {
        (self.inc(), NmiExtInt::new_async(token, pin))
    }
}

impl<AK, N> Enabled<EIController<AK, Configurable>, N>
where
    AK: AnyClock,
    N: Counter + PrivateDecrement,
{
    /// Disable ExtIntNmi
    pub fn disable_ext_int_nmi<I, C, AM, AS, AK2>(
        self,
        ext_int_nmi: NmiExtInt<I, C, AM, AK2, AS>,
    ) -> (
        <Self as PrivateDecrement>::Dec,
        NmiToken,
        Pin<I, Interrupt<C>>,
    )
    where
        I: NmiEI,
        C: InterruptConfig,
        AM: AnyMode,
        AS: AnySenseMode,
        AK2: AnyClock,
    {
        (self.dec(), ext_int_nmi.token, ext_int_nmi.pin)
    }
}
*/
