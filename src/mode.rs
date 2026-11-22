//! The driver's measurement mode, tracked in the type so only the commands valid in that mode can be called.

mod sealed {
    pub trait Sealed {}
}

/// A measurement mode of [`Sht3x`](crate::Sht3x). Implemented only by [`SingleShot`] and [`Periodic`].
pub trait Mode: sealed::Sealed {}

/// Idle between on-demand measurements. Every command is available. This is the mode after power-up and after any reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SingleShot;

/// Measuring continuously at a fixed rate. Only fetching results, stopping and resetting are available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Periodic;

impl sealed::Sealed for SingleShot {}
impl sealed::Sealed for Periodic {}
impl Mode for SingleShot {}
impl Mode for Periodic {}
