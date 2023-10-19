#![cfg_attr(not(feature = "std"), no_std)]
#![allow(stable_features)]
#![allow(unknown_lints)]
#![cfg_attr(feature = "nightly", feature(async_fn_in_trait))]
#![cfg_attr(feature = "nightly", allow(async_fn_in_trait))]
#![cfg_attr(feature = "nightly", feature(impl_trait_projections))]

use core::fmt::Debug;

#[cfg(feature = "nightly")]
pub mod asynch;
pub mod notification;

pub trait Sender {
    type Error: Debug;

    type Data;

    fn send(&mut self, data: &Self::Data) -> Result<Self::Data, Self::Error>;
}

impl<'t, T> Sender for &'t mut T
where
    T: Sender + 't,
{
    type Error = T::Error;

    type Data = T::Data;

    fn send(&mut self, data: &Self::Data) -> Result<Self::Data, Self::Error> {
        (*self).send(data)
    }
}

pub trait Receiver {
    type Error: Debug;

    type Data;

    fn recv(&mut self) -> Result<Self::Data, Self::Error>;
}

impl<'t, T> Receiver for &'t mut T
where
    T: Receiver + 't,
{
    type Error = T::Error;

    type Data = T::Data;

    fn recv(&mut self) -> Result<Self::Data, Self::Error> {
        (*self).recv()
    }
}
