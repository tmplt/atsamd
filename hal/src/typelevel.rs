//! Module supporting type-level programming

use core::marker::PhantomData;
use core::ops::{Add, Sub};

use typenum::{Add1, Bit, Sub1, UInt, Unsigned, B1, U0, U1};

mod private {
    /// Super trait used to mark traits with an exhaustive set of
    /// implementations
    pub trait Sealed {}
}

pub(crate) use private::Sealed;

/// Type-level version of the [`None`] variant
pub struct NoneT;

impl Sealed for NoneT {}

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

/// TODO
/// Local copy of `NonZero` so the compiler can prove it will never be
/// implemented for U0.
pub trait NonZero: Unsigned {}

impl<U: Unsigned, B: Bit> NonZero for UInt<U, B> {}

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

/// TODO
/// Compile-time counting
pub trait Count: Sealed {}

impl<N: Unsigned> Count for Natural<N> {}

/// TODO
/// `CountOps` must be a separate trait from `Count`
/// There are other ways to do this, but there are tradeoffs.
pub trait CountOps: Count {
    /// TODO
    type Add: Count;
    /// TODO
    type Sub: Count;
    /// TODO
    fn add(self) -> Self::Add;
    /// TODO
    fn sub(self) -> Self::Sub;
}

/// TODO
pub type CountAdd<C> = <C as CountOps>::Add;

/// TODO
pub type CountSub<C> = <C as CountOps>::Sub;

impl CountOps for Natural<U0> {
    type Add = Natural<U1>;
    type Sub = Natural<U0>;

    #[inline]
    fn add(self) -> Self::Add {
        Natural::new()
    }

    #[inline]
    fn sub(self) -> Self::Sub {
        panic!("Cannot subtract from Natural<U0>")
    }
}

impl<N> CountOps for Natural<N>
where
    N: NonZero + Add<B1> + Sub<B1>,
    Add1<N>: Unsigned,
    Sub1<N>: Unsigned,
{
    type Add = Natural<Add1<N>>;
    type Sub = Natural<Sub1<N>>;

    #[inline]
    fn add(self) -> Self::Add {
        Natural::new()
    }

    #[inline]
    fn sub(self) -> Self::Sub {
        Natural::new()
    }
}

/// TODO
/// Trait for compile-time lock counting
pub trait LockCount {
    /// TODO
    type Lock;

    /// TODO
    type Unlock;

    /// TODO
    unsafe fn lock(self) -> Self::Lock;

    /// TODO
    unsafe fn unlock(self) -> Self::Unlock;
}

/// TODO
pub type Lock<R> = <R as LockCount>::Lock;

/// TODO
pub type Unlock<R> = <R as LockCount>::Unlock;

