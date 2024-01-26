#![cfg_attr(not(feature = "std"), no_std)]
#![allow(async_fn_in_trait)]

use core::{fmt::Debug, marker::PhantomData};

pub mod asynch;
pub mod notification;

pub trait Sender {
    type Error: Debug;

    type Data;

    fn send(&mut self, data: &Self::Data) -> Result<(), Self::Error>;
}

impl<'t, T> Sender for &'t mut T
where
    T: Sender + 't,
{
    type Error = T::Error;

    type Data = T::Data;

    fn send(&mut self, data: &Self::Data) -> Result<(), Self::Error> {
        (*self).send(data)
    }
}

impl<D, E: Debug> Sender for PhantomData<fn() -> (D, E)> {
    type Error = E;

    type Data = D;

    fn send(&mut self, _data: &Self::Data) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn nil_sender<D, E: Debug>() -> impl Sender<Error = E, Data = D> {
    PhantomData
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

impl<F: FnMut(), D, E: Debug> Receiver for (F, PhantomData<fn() -> (D, E)>) {
    type Error = E;

    type Data = D;

    fn recv(&mut self) -> Result<Self::Data, Self::Error> {
        loop {
            (self.0)()
        }
    }
}

pub fn nil_receiver<F: FnMut(), D, E: Debug>(f: F) -> impl Receiver<Error = E, Data = D> {
    (f, PhantomData)
}
