
use std::{fmt::Debug, marker::PhantomData};

use crate::{helpers::QhTypeRef, sys};

#[derive(Clone, Copy)]
pub struct Vertex<'a>(*mut sys::vertexT, usize, PhantomData<&'a ()>);

impl<'a> Debug for Vertex<'a> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<'a> Vertex<'a> {
    pub fn new(vertex: *mut sys::vertexT, dim: usize) -> Self {
        assert_eq!(vertex.is_null(), false, "vertex is null");
        Self(vertex, dim, PhantomData)
    }

    pub unsafe fn raw_ptr(&self) -> *mut sys::vertexT {
        self.0
    }

    pub unsafe fn raw_ref(&self) -> &sys::vertexT {
        unsafe { &*self.0 }
    }

    pub fn dim(&self) -> usize {
        self.1
    }

    pub fn id(&self) -> u32 {
        let vertex = unsafe { self.raw_ref() };
        vertex.id
    }
}

impl<'a> QhTypeRef for Vertex<'a> {
    type FFIType = sys::vertexT;

    fn from_ptr(ptr: *mut Self::FFIType, dim: usize) -> Self {
        Self::new(ptr, dim)
    }

    unsafe fn raw_ptr(&self) -> *mut Self::FFIType {
        self.0
    }

    fn dim(&self) -> usize {
        self.1
    }
}