use std::cell::UnsafeCell;
use std::fmt::Display;
use std::mem::MaybeUninit;
use std::slice;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use crossbeam_utils::CachePadded;

use crate::base::{RbBase, RbRead, RbWrite};
use crate::consumer::Consumer;
use crate::producer::Producer;

pub struct SharedRb<T, const N: usize>
{
    buffer: UnsafeCell<[MaybeUninit<T>; N]>,
    // prevent share false
    head: CachePadded<AtomicUsize>,
    tail: CachePadded<AtomicUsize>,
    count: AtomicUsize,
}

impl<T, const N: usize> SharedRb<T, N>
    where T: Display
{
    pub fn new() -> Self {
        Self {
            head: CachePadded::new(AtomicUsize::new(0))  ,
            tail: CachePadded::new(AtomicUsize::new(0)),
            count: AtomicUsize::new(0),
            buffer: UnsafeCell::new(uninit_array()),
        }
    }

    pub fn split(self) -> (Producer<T, Arc<Self>>, Consumer<T, Arc<Self>>)
    {
        let arc = Arc::new(self);
        return (Producer::new(arc.clone()), Consumer::new(arc));
    }
}

impl<T, const N: usize> RbBase<T> for SharedRb<T, N>
{
    fn cap(&self) -> usize {
        N
    }

    fn head(&self) -> usize {
        self.head.load(Ordering::Acquire)
    }

    fn tail(&self) -> usize {
        self.tail.load(Ordering::Acquire)
    }

    fn count(&self) -> usize {
        self.count.load(Ordering::Acquire)    }

    fn set_count(&self, value: usize) {
        self.count.store(value, Ordering::Release)
    }
}

impl<T, const N: usize> RbRead<T> for SharedRb<T, N>
{
    fn set_tail(&self, value: usize) {
        self.tail.store(value, Ordering::Release)
    }

    unsafe fn occupied_slices(&self) -> (&[MaybeUninit<T>], &[MaybeUninit<T>]) {
        let ptr = self.buffer.get() as *mut MaybeUninit<T>;
        let ranges = self.occupied_range();
        (
            slice::from_raw_parts(ptr.add(ranges.0.start), ranges.0.len()),
            slice::from_raw_parts(ptr.add(ranges.1.start), ranges.1.len()),
        )
    }
}

impl<T, const N: usize> RbWrite<T> for SharedRb<T, N>
{
    fn set_head(&self, value: usize) {
        self.head.store(value, Ordering::Release)
    }

    unsafe fn vacant_slices(&self) -> (&mut [MaybeUninit<T>], &mut [MaybeUninit<T>]) {
        let ptr = self.buffer.get() as *mut MaybeUninit<T>;
        let ranges = self.vacant_range();
        (
            slice::from_raw_parts_mut(ptr.add(ranges.0.start), ranges.0.len()),
            slice::from_raw_parts_mut(ptr.add(ranges.1.start), ranges.1.len())
        )
    }
}

unsafe impl<T, const N: usize> Sync for SharedRb<T, N>{}

pub fn uninit_array<T, const N: usize>() -> [MaybeUninit<T>; N] {
    unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() }
}
