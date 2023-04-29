use std::{io, mem};
use std::fmt::{Debug, Display};
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

    pub fn occupied_slices(&self) -> (&[MaybeUninit<T>], &[MaybeUninit<T>]) {
        unsafe { self.inner.occupied_slices() }
    }

    pub fn pop_slice(&self, elems: &mut [T]) -> usize
        where T: Copy
    {
        let (left, right) = self.occupied_slices();
        let count = if elems.len() < left.len() {
            let uninit_src = unsafe { &*(&left[..elems.len()] as *const [MaybeUninit<T>] as *const [T]) };
            elems.copy_from_slice(uninit_src);
            elems.len()
        } else {
            let (left_elems, elems) = elems.split_at_mut(left.len());
            let uninit_src = unsafe { &*(left as *const [MaybeUninit<T>] as *const [T]) };
            left_elems.copy_from_slice(uninit_src);
            left.len()
                + if right.len() < elems.len() {
                let uninit_src = unsafe { &*(right as *const [MaybeUninit<T>] as *const [T]) };
                elems[..right.len()].copy_from_slice(uninit_src);
                right.len()
            } else {
                let uninit_src = unsafe { &*(right.split_at(elems.len()).0 as *const [MaybeUninit<T>] as *const [T]) };
                elems.copy_from_slice(uninit_src);
                elems.len()
            }
        };
        let _ = self.advance(count);
        count
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

