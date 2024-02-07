use std::ffi::c_void;
use std::marker::PhantomData;

use crate::helpers::QhTypeRef;

use crate::sys;

pub struct Set<'a, T: QhTypeRef> {
    set: *mut sys::setT,
    dim: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: QhTypeRef> Set<'a, T> {
    pub fn new(set: *mut sys::setT, dim: usize) -> Self {
        assert_eq!(set.is_null(), false, "set is null");
        Self {
            set,
            dim,
            _phantom: PhantomData,
        }
    }

    pub fn iter(&self) -> SetIterator<'a, T> {
        SetIterator::new(self)
    }
}

pub struct SetIterator<'a, T: QhTypeRef> {
    ptr: *mut *mut T::FFIType,
    dim: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: QhTypeRef> SetIterator<'a, T> {
    pub fn new(set: &Set<'a, T>) -> Self {
        let dim = set.dim;
        assert_eq!(set.set.is_null(), false);
        let set = unsafe { &*set.set };
        let ptr = unsafe { (&(set.e[0].p)) as *const *mut c_void as *mut *mut T::FFIType };
        Self {
            ptr,
            dim,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: QhTypeRef> Iterator for SetIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let value_ptr = unsafe { *self.ptr };
        if value_ptr.is_null() {
            return None;
        }
        self.ptr = unsafe { self.ptr.add(1) };
        Some(T::from_ptr(value_ptr, self.dim))
    }
}