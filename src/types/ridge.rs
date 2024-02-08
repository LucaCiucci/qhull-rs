use std::{fmt::Debug, marker::PhantomData};

use crate::{helpers::QhTypeRef, sys};

#[derive(Clone, Copy)]
pub struct Ridge<'a> {
    ridge: *mut sys::ridgeT,
    dim: usize,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Debug for Ridge<'a> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<'a> Ridge<'a> {
    
}

impl<'a> QhTypeRef for Ridge<'a> {
    type FFIType = sys::ridgeT;

    fn from_ptr(ptr: *mut Self::FFIType, dim: usize) -> Self {
        Self {
            ridge: ptr,
            dim,
            _phantom: PhantomData,
        }
    }

    fn dim(&self) -> usize {
        self.dim
    }

    unsafe fn raw_ptr(&self) -> *mut Self::FFIType {
        self.ridge
    }
}