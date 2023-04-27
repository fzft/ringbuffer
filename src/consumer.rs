use std::fmt::{Debug, Display};
use std::io;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use crate::base::{RbBase, RbRead, RbRef};

pub struct Consumer<T, R: RbRef>
    where
        R::Rb: RbRead<T>,
{
    inner: R,
    _marker: PhantomData<T>,
}

impl<T, R: RbRef> Consumer<T, R>
    where
        R::Rb: RbRead<T>,
        T: Display
{
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    pub fn cap(&self) -> usize {
        self.inner.cap()
    }

    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn advance(&self, count: usize) -> Result<(), String> {
        self.inner.advance_tail(count)
    }

    pub fn write_into<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let left = unsafe { self.inner.occupied_slices().0 };
        let init_ref = unsafe { &*(left as *const [MaybeUninit<T>] as *const [T] as *const [u8]) };
        let write_count = writer.write(init_ref)?;
        let _ = self.advance(write_count);
        Ok(write_count)
    }

    pub fn pop(&self) -> Option<T> {
        if !self.is_empty() {
            let elem = unsafe {
                self.inner.occupied_slices().0.get_unchecked(0).assume_init_read()
            };
            let _ = self.advance(1);
            return Some(elem);
        }
        return None;
    }
}

