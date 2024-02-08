
use std::{fmt::Debug, marker::PhantomData};

use crate::{helpers::QhTypeRef, sys, Set};

#[derive(Clone, Copy)]
pub struct Vertex<'a>(*mut sys::vertexT, usize, PhantomData<&'a ()>);

impl<'a> Debug for Vertex<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vertex")
            .field("id", &self.id())
            .field("visit_id", &self.visit_id())
            .field("point", &self.point())
            .field("neighbors_count", &self.neighbors().iter().count())
            .finish()
    }
}

impl<'a> Vertex<'a> {
    pub fn new(vertex: *mut sys::vertexT, dim: usize) -> Self {
        assert_eq!(vertex.is_null(), false, "vertex is null");
        Self(vertex, dim, PhantomData)
    }

    pub fn dim(&self) -> usize {
        self.1
    }

    pub fn next(&self) -> Vertex {
        let vertex = unsafe { self.raw_ref() };
        Self::new(vertex.next, self.dim())
    }

    pub fn previous(&self) -> Vertex {
        let vertex = unsafe { self.raw_ref() };
        Self::new(vertex.previous, self.dim())
    }

    pub fn point(&self) -> &[f64] {
        unsafe {
            let vertex = self.raw_ref();
            std::slice::from_raw_parts(vertex.point, self.dim())
        }
    }

    pub fn id(&self) -> u32 {
        let vertex = unsafe { self.raw_ref() };
        vertex.id
    }

    pub fn visit_id(&self) -> u32 {
        let vertex = unsafe { self.raw_ref() };
        vertex.visitid
    }

    pub fn neighbors(&self) -> Set<Vertex> {
        let vertex = unsafe { self.raw_ref() };
        Set::new(vertex.neighbors, self.dim())
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