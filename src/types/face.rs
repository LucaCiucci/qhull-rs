use std::{fmt::Debug, marker::PhantomData};

use crate::{helpers::QhTypeRef, sys, Set, Vertex};

#[derive(Clone, Copy)]
pub struct Face<'a>(*mut sys::facetT, usize, PhantomData<&'a ()>);

impl<'a> Debug for Face<'a> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<'a> Face<'a> {
    pub fn new(facet: *mut sys::facetT, dim: usize) -> Self {
        assert_eq!(facet.is_null(), false, "facet is null");
        Self(facet, dim, PhantomData)
    }

    pub fn vertices(&self) -> impl Iterator<Item = Vertex> {
        let face = unsafe { self.raw_ref() };
        let it = if face.vertices.is_null() {
            None
        } else {
            Some(Set::new(face.vertices, self.dim()).iter())
        };
        it.into_iter().flatten()
    }

    pub fn simplicial(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.simplicial() != 0
    }
}

impl<'a> QhTypeRef for Face<'a> {
    type FFIType = sys::facetT;

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

pub struct FaceIterator<'a>(*mut sys::facetT, usize, PhantomData<&'a ()>);

impl<'a> FaceIterator<'a> {
    pub fn new(
        facet: *mut sys::facetT,
        dim: usize,
    ) -> Self {
        assert_eq!(facet.is_null(), false, "facet is null");
        Self(facet, dim, PhantomData)
    }

    pub fn ptr(&self) -> *mut sys::facetT {
        self.0
    }
}

impl<'a> Iterator for FaceIterator<'a> {
    type Item = Face<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let facet = self.0;
        //println!("facet: {:?}", facet);
        if facet.is_null() {
            None
        } else {
            self.0 = unsafe { (*facet).next };
            Some(Face::new(facet, self.1))
        }
    }
}

impl<'a> DoubleEndedIterator for FaceIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let facet = self.0;
        if facet.is_null() {
            None
        } else {
            self.0 = unsafe { (*facet).previous };
            Some(Face::new(facet, self.1))
        }
    }
}