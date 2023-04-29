use std::mem::MaybeUninit;
use std::ops::{Deref, Range};
use std::sync::Arc;

pub trait RbBase<T>
{
    fn cap(&self) -> usize;

    fn head(&self) -> usize;

    fn tail(&self) -> usize;

    fn count(&self) -> usize;

    fn is_full(&self) -> bool {
        self.count() == self.cap()
    }

    fn is_empty(&self) -> bool {
       self.count() == 0
    }

    fn occupied_len(&self) -> usize {
        self.count()
    }

    fn free_len(&self) -> usize {
        self.cap() - self.count()
    }

    fn set_count(&self, value: usize);
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
        self.set_count(self.count() - count);
        Ok(())
    }

    unsafe fn occupied_slices(&self) -> (&[MaybeUninit<T>], &[MaybeUninit<T>]);

    fn occupied_range(&self) -> (Range<usize>, Range<usize>) {
        let head = self.head();
        let tail = self.tail();
        let count = self.count();
        let cap = self.cap();
        if count == 0 {
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
        self.set_count(self.count() + count);
        Ok(())
    }

    fn vacant_range(&self) -> (Range<usize>, Range<usize>) {
        let head = self.head();
        let tail = self.tail();
        let count = self.count();
        let cap = self.cap();
        if count == 0 && head == 0 && tail ==0 {
            // Buffer is empty
            (0..cap, 0..0)
        } else if count == cap {
            // Buffer is full
            (0..0, 0..0)
        } else if head >= tail {
            // Contiguous vacant range
            (head..cap, 0..tail)
        } else {
            // Two vacant ranges
            (head..tail, 0..0)
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









