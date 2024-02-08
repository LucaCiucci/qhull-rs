use std::{fmt::Debug, marker::PhantomData};

use crate::{helpers::QhTypeRef, sys, Face, Set, Vertex};

#[derive(Clone, Copy)]
pub struct Ridge<'a> {
    ridge: *mut sys::ridgeT,
    dim: usize,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Debug for Ridge<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ridge")
            .field("id", &self.id())
            .field("seen", &self.seen())
            .field("tested", &self.tested())
            .field("non_convex", &self.non_convex())
            .field("merge_vertex", &self.merge_vertex())
            .field("merge_vertex_2", &self.merge_vertex_2())
            .field("simplicial_top", &self.simplicial_top())
            .field("simplicial_bottom", &self.simplicial_bottom())
            .field("vertices", &self.vertices())
            .field("top", &self.top().id())
            .field("bottom", &self.bottom().id())
            .finish()
    }
}

impl<'a> Ridge<'a> {
    pub fn vertices(&self) -> Option<Set<'a, Vertex<'a>>> {
        let ridge = unsafe { self.raw_ref() };
        Set::maybe_new(ridge.vertices, self.dim)
    }

    pub fn top(&self) -> Face<'a> {
        let ridge = unsafe { self.raw_ref() };
        Face::from_ptr(ridge.top, self.dim).unwrap()
    }

    pub fn bottom(&self) -> Face<'a> {
        let ridge = unsafe { self.raw_ref() };
        Face::from_ptr(ridge.bottom, self.dim).unwrap()
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

    fn from_ptr(ptr: *mut Self::FFIType, dim: usize) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self {
                ridge: ptr,
                dim,
                _phantom: PhantomData,
            })
        }
    }

    fn dim(&self) -> usize {
        self.dim
    }

    unsafe fn raw_ptr(&self) -> *mut Self::FFIType {
        self.ridge
    }
}