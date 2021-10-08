pub mod v1;

// EIC::v2 is dependent on clocking::v2 which is not yet done
#[cfg(feature = "min-samd51g")]
pub mod v2;
