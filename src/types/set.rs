use std::ffi::c_void;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::helpers::QhTypeRef;

use crate::{sys, Face};

#[derive(Clone, Copy)]
pub struct Set<'a, T: QhTypeRef> {
    set: *mut sys::setT,
    dim: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: QhTypeRef> Debug for Set<'a, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("Set<{}>", std::any::type_name::<T>()))
            .field("elements:", &self.iter().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a, T: QhTypeRef> Set<'a, T> {
    pub fn maybe_new(set: *mut sys::setT, dim: usize) -> Option<Self> {
        if set.is_null() {
            None
        } else {
            Some(Self {
                set,
                dim,
                _phantom: PhantomData,
            })
        }
    }

    pub fn iter(&self) -> SetIterator<'a, T> {
        SetIterator::new(self)
    }
}

pub(crate) fn dbg_face_set(set: Option<Set<Face>>) -> Option<Vec<u32>> {
    set.map(|s| s.iter().map(|f| f.id()).collect())
}

#[derive(Clone, Copy)]
pub struct SetIterator<'a, T: QhTypeRef> {
    ptr: *mut *mut T::FFIType,
    dim: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: QhTypeRef> SetIterator<'a, T> {
    pub fn new(set: &Set<'a, T>) -> Self {
        let dim = set.dim;
        assert!(!set.set.is_null());
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
        let element = T::from_ptr(value_ptr, self.dim);
        if element.is_some() {
            self.ptr = unsafe { self.ptr.add(1) };
        }
        element
    }
}
