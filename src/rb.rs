use std::fmt::Display;
use crate::base::{RbRead, RbWrite};
use crate::consumer::Consumer;
use crate::producer::Producer;

pub trait Rb<T>: RbWrite<T> + RbRead<T>

    where T: Display
{

    #[inline]
    fn pop(&mut self) -> Option<T> {
         Consumer::new(self as &Self).pop()
    }

    // Removes all items from the buffer and safely drops them
    fn clear(&mut self) -> usize {
        0
    }

    // Appends an item to the ring buffer
    // On failure returns an 'Err' containing the item that hasn't been appended
    fn push(&mut self, elem: T) -> Result<(), T> {
       Producer::new(self as &Self).push(elem)
    }

}





