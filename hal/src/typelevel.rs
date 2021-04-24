//! Module supporting type-level programming

use core::marker::PhantomData;
use core::ops::{Add, Sub};

use typenum::{Add1, Sub1, Bit, UInt, Unsigned, B1, U0, U1};

mod private {
    /// Super trait used to mark traits with an exhaustive set of
    /// implementations
    pub trait Sealed {}
}

pub(crate) use private::Sealed;

/// Type-level version of the [`None`] variant
pub struct NoneT;

impl Sealed for NoneT {}

//==============================================================================
// Is
//==============================================================================

/// Marker trait for type identity
///
/// This trait must be implemented with `Self::Type == Self`. When used as a
/// trait bound with a constrained `Type`, i.e.
/// `where T: Is<Type = SpecificType>`, it allows easy conversion between the
/// generic type `T` and the `SpecificType` using [`Into`], [`AsRef`] and
/// [`AsMut`].
///
/// This trait is used throughout the HAL to create various `Any*` meta-types.
pub trait Is
where
    Self: From<IsType<Self>>,
    Self: Into<IsType<Self>>,
    Self: AsRef<IsType<Self>>,
    Self: AsMut<IsType<Self>>,
    IsType<Self>: AsRef<Self>,
    IsType<Self>: AsMut<Self>,
{
    type Type;
}

/// Alias for [`Is`]`::Type`
pub type IsType<T> = <T as Is>::Type;

/// Blanket implementation
impl<T> Is for T
where
    T: AsRef<T> + AsMut<T>,
{
    type Type = T;
}

//==============================================================================
// NonZero
//==============================================================================

/// TODO
/// Local copy of `NonZero` so the compiler can prove it will never be
/// implemented for U0.
pub trait NonZero: Unsigned {}

impl<U: Unsigned, B: Bit> NonZero for UInt<U, B> {}

//==============================================================================
// Natural
//==============================================================================

/// TODO
/// Phantom `Unsigned` typenums, so they can be constructed from generic
/// parameters
pub struct Natural<N: Unsigned> {
    n: PhantomData<N>,
}

/// TODO
pub type Zero = Natural<U0>;

/// TODO
pub type One = Natural<U1>;

impl<N: Unsigned> Sealed for Natural<N> {}

impl<N: Unsigned> Natural<N> {
    /// TODO
    pub fn new() -> Self {
        Natural { n: PhantomData }
    }
}

//==============================================================================
// Count
//==============================================================================

/// TODO
/// Compile-time counting
pub trait Count: Sealed {}

impl<N: Unsigned> Count for Natural<N> {}

//==============================================================================
// Increment
//==============================================================================

/// TODO
pub trait Increment: Count {
    type Inc: Count;
    fn inc(self) -> Self::Inc;
}

impl<N> Increment for Natural<N>
where
    N: Unsigned + Add<B1>,
    Add1<N>: Unsigned,
{
    type Inc = Natural<Add1<N>>;
    fn inc(self) -> Self::Inc {
        Natural::new()
    }
}

//==============================================================================
// Decrement
//==============================================================================

/// TODO
pub trait Decrement: Count {
    type Dec: Count;
    fn dec(self) -> Self::Dec;
}

impl<N> Decrement for Natural<N>
where
    N: NonZero + Sub<B1>,
    Sub1<N>: Unsigned,
{
    type Dec = Natural<Sub1<N>>;
    fn dec(self) -> Self::Dec {
        Natural::new()
    }
}

//==============================================================================
// GreaterThanOne
//==============================================================================

pub trait GreaterThanOne {}

impl<U: Unsigned, X: Bit, Y: Bit> GreaterThanOne for UInt<UInt<U, X>, Y> {}

impl<N: Unsigned + GreaterThanOne> GreaterThanOne for Natural<N> {}

//==============================================================================
// Lockable
//==============================================================================

pub trait Lockable {
    type Locked;
    fn lock(self) -> Self::Locked;
}

//==============================================================================
// Unlockable
//==============================================================================

pub trait Unlockable {
    type Unlocked;
    fn unlock(self) -> Self::Unlocked;
}
