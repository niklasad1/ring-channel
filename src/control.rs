use crate::buffer::*;
use crossbeam_utils::CachePadded;
use derivative::Derivative;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering, AtomicPtr};
use std::ops::Deref;

#[cfg(feature = "futures_api")]
use crate::waitlist::*;

#[cfg(feature = "futures_api")]
use std::task::Waker;

#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub(super) struct ControlBlock<T> {
    pub(super) senders: CachePadded<AtomicUsize>,
    pub(super) receivers: CachePadded<AtomicUsize>,
    pub(super) connected: AtomicBool,
    pub(super) buffer: RingBuffer<T>,

    #[cfg(feature = "futures_api")]
    pub(super) waitlist: Waitlist<Waker>,
}

impl<T> ControlBlock<T> {
    fn new(capacity: usize) -> Self {
        Self {
            senders: CachePadded::new(AtomicUsize::new(1)),
            receivers: CachePadded::new(AtomicUsize::new(1)),
            connected: AtomicBool::new(true),
            buffer: RingBuffer::new(capacity),

            #[cfg(feature = "futures_api")]
            waitlist: Waitlist::new(),
        }
    }
}

#[derive(Debug)]
pub(super) struct ControlBlockRef<T>(AtomicPtr<ControlBlock<T>>);

impl<T> Unpin for ControlBlockRef<T> {}

impl<T> ControlBlockRef<T> {
    pub(super) fn new(capacity: usize) -> Self {
        ControlBlockRef(
            AtomicPtr::new(Box::into_raw(Box::new(ControlBlock::new(capacity))))
        )
    }
}

impl<T> PartialEq for ControlBlockRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.load(Ordering::SeqCst) == other.0.load(Ordering::SeqCst)
    }
}

impl<T> Clone for ControlBlockRef<T> {
    fn clone(&self) -> Self {
        Self ( AtomicPtr::new(self.0.load(Ordering::SeqCst)) )
    }
}

impl<T> Eq for ControlBlockRef<T> {}

impl<T> Deref for ControlBlockRef<T> {
    type Target = ControlBlock<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.load(Ordering::SeqCst).as_ref().unwrap() }
    }
}

impl<T> Drop for ControlBlockRef<T> {
    fn drop(&mut self) {
        debug_assert!(!self.connected.load(Ordering::Relaxed));
        debug_assert_eq!(self.senders.load(Ordering::Relaxed), 0);
        debug_assert_eq!(self.receivers.load(Ordering::Relaxed), 0);

        unsafe { Box::from_raw(&**self as *const ControlBlock<T> as *mut ControlBlock<T>) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn control_block_starts_connected() {
        let ctrl = ControlBlock::<()>::new(1);
        assert_eq!(ctrl.connected.load(Ordering::Relaxed), true);
    }

    #[test]
    fn control_block_starts_with_reference_counters_equal_to_one() {
        let ctrl = ControlBlock::<()>::new(1);
        assert_eq!(ctrl.senders.load(Ordering::Relaxed), 1);
        assert_eq!(ctrl.receivers.load(Ordering::Relaxed), 1);
    }

    proptest! {
        #[test]
        fn control_block_allocates_buffer_given_capacity(cap in 1..=100usize) {
            let ctrl = ControlBlock::<()>::new(cap);
            assert_eq!(ctrl.buffer.capacity(), cap);
        }
    }
}
