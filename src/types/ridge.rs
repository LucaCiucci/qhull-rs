use std::{fmt::Debug, marker::PhantomData};

use crate::{helpers::QhTypeRef, sys, Face, Set, Vertex};

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
    pub fn vertices(&self) -> Set<'a, Vertex<'a>> {
        let ridge = unsafe { self.raw_ref() };
        Set::new(ridge.vertices, self.dim)
    }

    pub fn top(&self) -> Face<'a> {
        let ridge = unsafe { self.raw_ref() };
        Face::new(ridge.top, self.dim)
    }

    pub fn bottom(&self) -> Face<'a> {
        let ridge = unsafe { self.raw_ref() };
        Face::new(ridge.bottom, self.dim)
    }

    pub fn id(&self) -> u32 {
        let ridge = unsafe { self.raw_ref() };
        ridge.id
    }

    pub fn seen(&self) -> bool {
        let ridge = unsafe { self.raw_ref() };
        ridge.seen() != 0
    }

    pub fn tested(&self) -> bool {
        let ridge = unsafe { self.raw_ref() };
        ridge.tested() != 0
    }

    pub fn non_convex(&self) -> bool {
        let ridge = unsafe { self.raw_ref() };
        ridge.nonconvex() != 0
    }

    pub fn merge_vertex(&self) -> bool {
        let ridge = unsafe { self.raw_ref() };
        ridge.mergevertex() != 0
    }

    pub fn merge_vertex_2(&self) -> bool {
        let ridge = unsafe { self.raw_ref() };
        ridge.mergevertex2() != 0
    }

    pub fn simplicial_top(&self) -> bool {
        let ridge = unsafe { self.raw_ref() };
        ridge.simplicialtop() != 0
    }

    pub fn simplicial_bottom(&self) -> bool {
        let ridge = unsafe { self.raw_ref() };
        ridge.simplicialbot() != 0
    }
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