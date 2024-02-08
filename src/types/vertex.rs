
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
            .field("neighbors", &self.neighbors().iter().map(|n| n.iter().map(|v| v.id()).collect::<Vec<_>>()).collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> Vertex<'a> {
    pub fn dim(&self) -> usize {
        self.1
    }

    pub fn next(&self) -> Option<Vertex<'a>> {
        let vertex = unsafe { self.raw_ref() };
        Self::from_ptr(vertex.next, self.dim())
    }

    pub fn previous(&self) -> Option<Vertex<'a>> {
        let vertex = unsafe { self.raw_ref() };
        Self::from_ptr(vertex.previous, self.dim())
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

    pub fn neighbors(&self) -> Option<Set<'a, Vertex<'a>>> {
        let vertex = unsafe { self.raw_ref() };
        if vertex.neighbors.is_null() {
            None
        } else {
            Some(Set::new(vertex.neighbors, self.dim()))
        }
    }
}

impl<'a> QhTypeRef for Vertex<'a> {
    type FFIType = sys::vertexT;

    fn from_ptr(ptr: *mut Self::FFIType, dim: usize) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr, dim, PhantomData))
        }
    }

    unsafe fn raw_ptr(&self) -> *mut Self::FFIType {
        self.0
    }

    fn dim(&self) -> usize {
        self.1
    }
}