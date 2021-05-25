//! Module supporting type-level programming

use core::ops::{Add, Sub};

use typenum::{Add1, Bit, Sub1, UInt, UTerm, Unsigned, B1};

pub mod counted;

mod private {
    use super::*;
    /// Super trait used to mark traits with an exhaustive set of
    /// implementations
    pub trait Sealed {}
    pub trait Increment: Counter {
        type Inc: Counter;
        fn inc(self) -> Self::Inc;
    }
    pub trait Decrement: Counter {
        type Dec: Counter;
        fn dec(self) -> Self::Dec;
    }
}

pub(crate) use private::Decrement as PrivateDecrement;
pub(crate) use private::Increment as PrivateIncrement;
pub(crate) use private::Sealed;

/// TODO
pub trait Increment: PrivateIncrement {}

/// TODO
pub trait Decrement: PrivateDecrement {}

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

/// TODO ?
pub trait Counter: Sealed {}

impl<N: Unsigned + Sealed> Counter for N {}

impl<U: Unsigned, B: Bit> Sealed for UInt<U, B> {}

impl Sealed for UTerm {}

// Helper ergonomic impls of *crement traits for PhantomData<N: Unsigned>
impl<N> PrivateIncrement for N
where
    N: Sealed + Unsigned + Add<B1>,
    Add1<N>: Sealed + Unsigned,
{
    type Inc = Add1<N>;
    fn inc(self) -> Self::Inc {
        Self::Inc::default()
    }
}

// TODO: Implement proper SourceType mechanism and remove this
impl<N> Increment for N
where
    N: Sealed + Unsigned + Add<B1>,
    Add1<N>: Sealed + Unsigned,
{
}

impl<N> PrivateDecrement for N
where
    N: Sealed + Unsigned + Sub<B1>,
    Sub1<N>: Sealed + Unsigned,
{
    type Dec = Sub1<N>;
    fn dec(self) -> Self::Dec {
        Self::Dec::default()
    }
}

// TODO: Implement proper SourceType mechanism and remove this
impl<N> Decrement for N
where
    N: Sealed + Unsigned + Sub<B1>,
    Sub1<N>: Sealed + Unsigned,
{
}
