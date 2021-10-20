use core::marker::PhantomData;

use bitfield::*;

use typenum::U0;

use crate::clock::v2::{
    osculp32k::OscUlp32k,
    pclk::{Eic, Pclk, PclkSourceMarker},
    rtc::{Active32k, Output1k},
    types::{
        Counter as ClockCounter, Decrement as ClockDecrement, Enabled as ClockEnabled,
        Increment as ClockIncrement,
    },
};
use crate::gpio::v2::{Interrupt, InterruptConfig, Pin};

// Copied from crate::clock::v2::types, just importing from
// there causes cargo doc to combine clocking and EIC
// This needs revisiting
use types::{Counter, Enabled, PrivateDecrement, PrivateIncrement};

use crate::eic::v2::*;

use super::extint::*;

//==============================================================================
// Clock
//==============================================================================

/// EIC clock source marker type
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

/// EIC Clock Source
pub trait EIClkSrc: Sealed {
    type Type: EIClkSrcMarker;
}

impl<T> EIClkSrc for Pclk<Eic, T>
where
    T: PclkSourceMarker,
{
    type Type = Pclk<Eic, T>;
}

impl<Y: Output1k, N: ClockCounter> EIClkSrc for ClockEnabled<OscUlp32k<Active32k, Y>, N> {
    type Type = ClockEnabled<OscUlp32k<Active32k, Y>, N>;
}

/// Trait describing which clock-source the [`EIController`]
/// can use
///
/// Either with an external clock source, or without:
///
/// * [`WithoutClock`]
/// * [`WithClock`]
///
/// `WithClock` support these external clock sources:
///
/// * [`CLK_GCLK`][crate::clock::v2::gclk]
/// * [`CLK_ULP32K`][crate::clock::v2::osculp32k]
///
/// This clock selection is stored in `EIC->CTRLA->CKSEL`
pub trait Clock: Sealed {}

/// Only allows asynchronous detection
pub struct WithoutClock {}
impl Sealed for WithoutClock {}
impl Clock for WithoutClock {}

/// This mode allows full EIC functionality
///
/// Required if:
/// * The NMI is using edge detection or filtering
/// * One EXTINT uses filtering
/// * One EXTINT uses synchronous edge detection
/// * One EXTINT uses debouncing
//pub struct WithClock<CS: ClkSrc> {
pub struct WithClock<CS> {
    /// Clock resource
    _clksrc: CS,
}
impl<C: ClkSrc> Sealed for WithClock<C> {}
impl<C: ClkSrc> Clock for WithClock<C> {}

pub struct ClockMode<C: Clock> {
    _mode: C,
}

pub trait ClkSrc: Sealed {}

/// EIC driven from a GCLK connected through the Eic [`Pclk`]
pub struct PclkDriven<T: PclkSourceMarker> {
    _reference_clk: Pclk<Eic, T>,
}
impl<T: PclkSourceMarker> ClkSrc for PclkDriven<T> {}
impl<T: PclkSourceMarker> Sealed for PclkDriven<T> {}

/// EIC driven from [`OscUlp32k`]
pub struct OscUlp32kDriven {}
impl ClkSrc for OscUlp32kDriven {}
impl Sealed for OscUlp32kDriven {}

/// Type class for all possible [`Clock`] types
///
/// This trait uses the [`AnyKind`] trait pattern to create a [type class] for
/// [`Clock`] types. See the `AnyKind` documentation for more details on the
/// pattern.
///
/// [`AnyKind`]: crate::typelevel#anykind-trait-pattern
/// [type class]: crate::typelevel#type-classes
pub trait AnyClock: Sealed {
    type Mode: Clock;
}

impl AnyClock for WithoutClock {
    type Mode = WithoutClock;
}

impl<CS> AnyClock for WithClock<CS>
where
    CS: ClkSrc,
{
    type Mode = WithClock<CS>;
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

impl<CS: ClkSrc> AsRef<Self> for WithClock<CS> {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<CS: ClkSrc> AsMut<Self> for WithClock<CS> {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
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
    clockmode: ClockMode<AK::Mode>,
    _enablestate: PhantomData<EP>,
}

impl EIController<WithClock<OscUlp32kDriven>, Configurable> {
    /// Create an EIC Controller with a clock source
    ///
    /// This allows for full EIC functionality
    ///
    /// # Safety
    ///
    /// Safe because you trade a singleton PAC struct for new singletons
    #[allow(clippy::type_complexity)]
    pub fn from_osculp<T, S>(
        eic: crate::pac::EIC,
        clock: S,
    ) -> (
        Enabled<EIController<WithClock<OscUlp32kDriven>, Configurable>, U0>,
        Tokens,
        S::Inc,
    )
    where
        T: EIClkSrcMarker,
        S: EIClkSrc<Type = T> + ClockIncrement,
    {
        // Software reset the EIC controller on creation
        eic.ctrla.modify(|_, w| w.swrst().set_bit());
        while eic.syncbusy.read().swrst().bit_is_set() {}

        // Set CKSEL to match the clock resource provided
        eic.ctrla.modify(|_, w| w.cksel().variant(T::CKSEL));

        unsafe {
            (
                Enabled::new(Self {
                    eic,
                    clockmode: ClockMode {
                        _mode: WithClock {
                            _clksrc: OscUlp32kDriven {},
                        },
                    },
                    _enablestate: PhantomData,
                }),
                Tokens::new(),
                clock.inc(),
            )
        }
    }
}

impl<T> EIController<WithClock<PclkDriven<T>>, Configurable>
where
    T: PclkSourceMarker,
{
    /// Create an EIC Controller with a clock source
    ///
    /// This allows for full EIC functionality
    ///
    /// # Safety
    ///
    /// Safe because you trade a singleton PAC struct for new singletons
    #[allow(clippy::type_complexity)]
    pub fn from_pclk(
        eic: crate::pac::EIC,
        clock: Pclk<Eic, T>,
    ) -> (
        Enabled<EIController<WithClock<PclkDriven<T>>, Configurable>, U0>,
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
                    clockmode: ClockMode {
                        _mode: WithClock {
                            _clksrc: PclkDriven {
                                _reference_clk: clock,
                            },
                        },
                    },
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
                    clockmode: ClockMode {
                        _mode: WithoutClock {},
                    },
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

impl Enabled<EIController<WithClock<OscUlp32kDriven>, Configurable>, U0> {
    /// Disable and destroy the EIC controller
    pub fn destroy<S>(self, _tokens: Tokens, clock: S) -> (crate::pac::EIC, S::Dec)
    where
        S: ClockDecrement,
    {
        (self.0.eic, clock.dec())
    }
}

impl<T> Enabled<EIController<WithClock<PclkDriven<T>>, Configurable>, U0>
where
    T: PclkSourceMarker,
{
    /// Disable and destroy the EIC controller
    pub fn destroy(self, _tokens: Tokens) -> (crate::pac::EIC, Pclk<Eic, T>) {
        (self.0.eic, self.0.clockmode._mode._clksrc._reference_clk)
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
    CS: ClkSrc,
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
    #[allow(clippy::type_complexity)]
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
    #[allow(clippy::type_complexity)]
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
    CS: ClkSrc,
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
    CS: ClkSrc,
    N: Counter + PrivateIncrement,
{
    /// Create an ExtIntNmi with a clock source
    #[allow(clippy::type_complexity)]
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
