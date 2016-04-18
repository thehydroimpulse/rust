// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![unstable(feature = "async", issue = "0")]
#![allow(missing_docs)]

use prelude::v1::*;
use sync::mpsc::{Receiver, channel};

/// Doc
pub trait Awaitable {
    type Unit;

    fn completed(&self) -> bool;
    fn unwrap(self) -> Self::Unit;
}

macro_rules! impl_awaitable {
    ($ty:ty) => (
        // Implement for all primitive types.
        impl Awaitable for $ty {
            type Unit = Self;

            /// Doc
            #[inline]
            fn completed(&self) -> bool {
                true
            }

            /// Doc
            #[inline]
            fn unwrap(self) -> Self {
                self
            }
        }
    );

    ($ty:ty, $c:ident) => (
        impl<$c> Awaitable for $ty {
            type Unit = Self;

            /// Doc
            #[inline]
            fn completed(&self) -> bool {
                true
            }

            /// Doc
            #[inline]
            fn unwrap(self) -> Self {
                self
            }
        }
    );
}

impl_awaitable!(usize);
impl_awaitable!(isize);
impl_awaitable!(String);
impl_awaitable!(i64);
impl_awaitable!(i32);
impl_awaitable!(i16);
impl_awaitable!(i8);
impl_awaitable!(Vec<T>, T);

impl_awaitable!(u64);

/// Doc
pub struct Future<T, E=()>
    where T: 'static + Send,
          E: 'static + Send
{
    rx: Receiver<Result<T, E>>,
    resolved: Option<Result<T, E>>
}

impl<T, E> Future<T, E>
    where T: 'static + Send,
          E: 'static + Send
{
    /// Doc
    pub fn unit(val: T) -> Future<T, E> {
        let (_, rx) = channel();

        Future {
            rx: rx,
            resolved: Some(Ok(val))
        }
    }

    /// Doc
    pub fn get(self) -> Result<T, E> {
        self.rx.recv().unwrap()
    }
}

impl<T, E> Awaitable for Future<T, E>
    where T: 'static + Send,
          E: 'static + Send
{
    type Unit = Result<T, E>;

    /// Doc
    fn completed(&self) -> bool {
        match self.resolved {
            Some(_) => true,
            None => false
        }
    }


    /// Doc
    fn unwrap(self) -> Self::Unit {
        match self.resolved {
            Some(res) => res,
            None => self.get()
        }
    }
}
