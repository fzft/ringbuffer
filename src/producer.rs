use std::io;
use std::io::Read;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use crate::base::{RbBase, RbRef, RbWrite};

pub struct Producer<T, R: RbRef>
    where R::Rb: RbWrite<T>,
{
    inner: R,
    _marker: PhantomData<T>,
}


impl<T, R: RbRef> Producer<T, R>
    where R::Rb: RbWrite<T>,
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
        self.inner.advance_head(count)
    }

    pub fn read_from<P: Read>(&mut self, reader: &mut P) -> io::Result<usize> {
        let left = unsafe { self.inner.vacant_slices().0 };
        let init_ref = unsafe { &mut*(left as *mut [MaybeUninit<T>] as *mut [T] as *mut [u8]) };
        let read_count = reader.read(init_ref)?;
        let _ = self.advance(read_count);
        Ok(read_count)
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if !self.is_full() {
            unsafe {
                self.inner.vacant_slices().0.get_unchecked_mut(0).write(value);
            }
            let _ = self.advance(1);
            return Ok(());
        }
        eprintln!("is full");
        return Err(value);
    }
}

