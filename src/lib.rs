#![feature(raw_vec_internals, core_intrinsics)]
extern crate alloc;

use std::cell::Cell;
use std::intrinsics::assume;
use std::ops::{Deref, DerefMut};
use alloc::raw_vec::RawVec;

pub struct ConstVec<T> {
    buf: RawVec<T>,
    len: Cell<usize>
}

impl<T> ConstVec<T> {
    pub fn new(capacity: usize) -> ConstVec<T> {
        ConstVec {
            buf: RawVec::with_capacity(capacity),
            len: Cell::new(0)
        }
    }

    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    pub fn len(&self) -> usize {
        self.len.get()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        let ptr = self.buf.ptr();
        unsafe { assume(!ptr.is_null()); }
        ptr
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        // We shadow the slice method of the same name to avoid going through
        // `deref_mut`, which creates an intermediate reference.
        let ptr = self.buf.ptr();
        unsafe { assume(!ptr.is_null()); }
        ptr
    }

    pub fn push(&self, value: T) {
        if self.len() == self.buf.capacity() {
            panic!("not enough space!")
        }
        unsafe {
            let ptr = self.buf.ptr();
            assume(!ptr.is_null());

            let end = ptr.add(self.len());
            std::ptr::write(end, value);
            self.len.set(self.len() + 1);
        }
    }
}

impl<T> Deref for ConstVec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(self.as_ptr(), self.len())
        }
    }
}

impl<T> DerefMut for ConstVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len())
        }
    }
}
