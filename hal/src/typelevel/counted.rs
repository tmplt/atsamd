use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use super::*;

pub struct Counted<T, N: Count>(pub(crate) T, PhantomData<N>);

impl<T> Counted<T, Zero> {
    pub(crate) fn new(t: T) -> Self {
        Self(t, PhantomData)
    }
}

impl<T, N: Count> Counted<T, N> {
    pub(crate) unsafe fn new_unsafe(t: T) -> Self {
        Counted(t, PhantomData)
    }
}

impl<T, N: Count> Sealed for Counted<T, N> {}
impl<T, N: Count> Count for Counted<T, N> {}

impl<T, N: Increment> PrivateIncrement for Counted<T, N> {
    type Inc = Counted<T, N::Inc>;

    fn inc(self) -> Self::Inc {
        Counted(self.0, PhantomData)
    }
}

impl<T, N: Decrement> PrivateDecrement for Counted<T, N> {
    type Dec = Counted<T, N::Dec>;

    fn dec(self) -> Self::Dec {
        Counted(self.0, PhantomData)
    }
}

impl<T, N: Count> Deref for Counted<T, N> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T, N: Count> DerefMut for Counted<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
