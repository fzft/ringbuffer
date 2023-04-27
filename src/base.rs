use std::f32::consts::E;
use std::mem::MaybeUninit;
use std::ops::{Deref, Range};
use std::sync::Arc;

pub trait RbBase<T>
{
    fn cap(&self) -> usize;

    fn head(&self) -> usize;

    fn tail(&self) -> usize;

    fn is_full(&self) -> bool {
        self.free_len() == 0
    }

    fn is_empty(&self) -> bool {
        self.head() == self.tail()
    }

    fn occupied_len(&self) -> usize {
        return if self.is_empty() {
            0
        } else {
            (self.head() + self.cap() - self.tail()) % self.cap()
        };
    }

    fn free_len(&self) -> usize {
        eprintln!("--------");
        eprintln!("cap: {}", self.cap());
        eprintln!("tail: {}", self.tail());
        eprintln!("head: {}", self.head());
        eprintln!("--------");
        if self.is_empty() {
            return self.cap();
        } else {
            (self.cap() + self.tail() - self.head() - 1) % self.cap()
        }
    }
}


pub trait RbRead<T>: RbBase<T>
{
    fn set_tail(&self, value: usize);

    fn advance_tail(&self, count: usize) -> Result<(), String> {
        if count > self.occupied_len() {
            return Err(format!("Cannot advance tail by a count {} larger than the occupied len. {}", count, self.occupied_len()));
        }
        let new_tail = (self.tail() + count) % self.cap();
        self.set_tail(new_tail);
        Ok(())
    }

    unsafe fn occupied_slices(&self) -> (&[MaybeUninit<T>], &[MaybeUninit<T>]);

    fn occupied_range(&self) -> (Range<usize>, Range<usize>) {
        let head = self.head();
        let tail = self.tail();
        let cap = self.cap();
        let occupied_len = self.occupied_len();
        if occupied_len == 0 {
            ((0..0), (0..0))
        } else if head > tail {
            ((tail..head), (0..0))
        } else {
            ((tail..cap), (0..head))
        }
    }
}

pub trait RbWrite<T>: RbBase<T> {
    fn set_head(&self, value: usize);

    fn advance_head(&self, count: usize) -> Result<(), String> {
        eprintln!("free len: {}", self.free_len());
        if count > self.free_len() {
            return Err(format!("Cannot advance head by a count {} larger than the free len. {}", count, self.free_len()));
        }
        let new_head = (self.head() + count) % self.cap();
        eprintln!("new head: {}", new_head);
        self.set_head(new_head);
        Ok(())
    }

    fn vacant_range(&self) -> (Range<usize>, Range<usize>) {
        let head = self.head();
        let tail = self.tail();
        let cap = self.cap();
        let vacant = self.free_len();
        if vacant == 0 {
            (0..0, 0..0)
        } else if head <= tail {
            ((head..tail), (0..0))
        } else {
            ((head..cap), (0..tail))
        }
    }

    unsafe fn vacant_slices(&self) -> (&mut [MaybeUninit<T>], &mut [MaybeUninit<T>]);
}

pub trait RbRef: Deref<Target=Self::Rb> {
    type Rb: ?Sized;
}


impl<'a, B: ?Sized> RbRef for &'a B {
    type Rb = B;
}

impl<'a, B: ?Sized> RbRef for Arc<B> {
    type Rb = B;
}









