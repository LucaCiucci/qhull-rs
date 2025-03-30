use std::ffi::c_void;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Not;

use crate::helpers::QhTypeRef;

use crate::{sys, Facet, Qh};

/// Represents a set of Qhull elements
#[derive(Clone, Copy)]
pub struct Set<'a, T: QhTypeRef> {
    qh: *mut sys::qhT,
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
    pub(crate) fn maybe_new(qh: *mut sys::qhT, set: *mut sys::setT, dim: usize) -> Option<Self> {
        set.is_null().not().then(|| Self {
            qh,
            set,
            dim,
            _phantom: PhantomData,
        })
    }

    /// Iterate over the elements of the set
    pub fn iter(&self) -> impl Iterator<Item = T> + 'a {
        SetIterator::new(self)
    }

    pub fn maxsize(&self) -> i32 {
        let set = unsafe { &*self.set };
        set.maxsize
    }

    pub fn size(&self, qh: &Qh) -> usize {
        // TODO error handling, possibly reimplement like QhullSet.cpp to avoid
        // qh_setsize error handling
        unsafe {
            sys::qh_setsize(Qh::raw_ptr(qh) as *mut _, self.set) as usize
        }
    }
}

pub(crate) fn dbg_face_set(set: Option<Set<Facet>>) -> Option<Vec<u32>> {
    set.map(|s| s.iter().map(|f| f.id()).collect())
}

#[derive(Clone, Copy)]
struct SetIterator<'a, T: QhTypeRef> {
    qh: *mut sys::qhT,
    ptr: *mut *mut T::FFIType,
    dim: usize,
    s: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: QhTypeRef> SetIterator<'a, T> {
    pub fn new(set: &Set<'a, T>) -> Self {
        let qh = set.qh;
        let dim = set.dim;
        let s = unsafe {
            // TODO error handling, see above
            sys::qh_setsize(set.qh, set.set) as usize
        };
        assert!(!set.set.is_null());
        let set = unsafe { &*set.set };
        let ptr = unsafe { (&(set.e[0].p)) as *const *mut c_void as *mut *mut T::FFIType };
        Self {
            qh,
            ptr,
            dim,
            s,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: QhTypeRef> Iterator for SetIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO comment on how this works (see the corresponding macro in qhull)
        // TODO maybe this could also be reversed if the size is known
        let value_ptr = unsafe { *self.ptr };
        let element = T::from_ptr(self.qh, value_ptr, self.dim);
        if element.is_some() {
            self.ptr = unsafe { self.ptr.add(1) };
        }
        element
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // TODO check
        (self.s, Some(self.s))
    }
}