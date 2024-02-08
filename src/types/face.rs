use std::{fmt::Debug, marker::PhantomData};

use crate::{helpers::QhTypeRef, sys, Ridge, Set, Vertex};

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

    pub fn furthest_dist(&self) -> f64 {
        let face = unsafe { self.raw_ref() };
        face.furthestdist
    }

    pub fn max_outside(&self) -> f64 {
        let face = unsafe { self.raw_ref() };
        face.maxoutside
    }

    pub fn offset(&self) -> f64 {
        let face = unsafe { self.raw_ref() };
        face.offset
    }

    pub fn normal(&self) -> &[f64] {
        unsafe {
            let face = self.raw_ref();
            std::slice::from_raw_parts(face.normal, self.dim())
        }
    }

    // TODO that union??

    pub fn center(&self) -> &[f64] {
        unsafe {
            let face = self.raw_ref();
            std::slice::from_raw_parts(face.center, self.dim())
        }
    }

    pub fn previous(&self) -> Face {
        let face = unsafe { self.raw_ref() };
        Self::new(face.previous, self.dim())
    }

    pub fn next(&self) -> Face {
        let face = unsafe { self.raw_ref() };
        Self::new(face.next, self.dim())
    }

    pub fn vertices(&self) -> Set<Vertex> {
        let face = unsafe { self.raw_ref() };
        Set::new(face.vertices, self.dim())
    }

    pub fn ridges(&self) -> Option<Set<Ridge>> {
        if self.dim() == 0 {
            return None;
        } else {
            let face = unsafe { self.raw_ref() };
            let has_ridges = !face.ridges.is_null();
            has_ridges.then(|| Set::new(face.ridges, self.dim() - 1))
        }
    }

    pub fn neighbors(&self) -> Set<Face> {
        let face = unsafe { self.raw_ref() };
        Set::new(face.neighbors, self.dim())
    }

    pub fn outside_set(&self) -> Set<Vertex> {
        let face = unsafe { self.raw_ref() };
        Set::new(face.outsideset, self.dim())
    }

    pub fn coplanar_set(&self) -> Set<Vertex> {
        let face = unsafe { self.raw_ref() };
        Set::new(face.coplanarset, self.dim())
    }

    pub fn visit_id(&self) -> u32 {
        let face = unsafe { self.raw_ref() };
        face.visitid
    }

    pub fn id(&self) -> u32 {
        let face = unsafe { self.raw_ref() };
        face.id
    }

    pub fn num_merge(&self) -> u32 {
        let face = unsafe { self.raw_ref() };
        face.nummerge()
    }

    pub fn max_num_merge() -> u32 {
        sys::qh_MAXnummerge
    }

    pub fn tricoplanar(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.tricoplanar() != 0
    }

    pub fn new_facet(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.newfacet() != 0
    }

    pub fn visible(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.visible() != 0
    }

    pub fn top_orient(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.toporient() != 0
    }

    pub fn simplicial(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.simplicial() != 0
    }

    pub fn seen(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.seen() != 0
    }

    pub fn seen2(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.seen2() != 0
    }

    pub fn flipped(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.flipped() != 0
    }

    pub fn upper_delaunay(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.upperdelaunay() != 0
    }

    pub fn not_furthest(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.notfurthest() != 0
    }

    pub fn good(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.good() != 0
    }

    pub fn is_area(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.isarea() != 0
    }

    pub fn dup_ridge(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.dupridge() != 0
    }

    pub fn merge_ridge(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.mergeridge() != 0
    }

    pub fn merge_ridge2(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.mergeridge2() != 0
    }

    pub fn coplanar_horizon(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.coplanarhorizon() != 0
    }

    pub fn merge_horizon(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.mergehorizon() != 0
    }

    pub fn cycle_done(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.cycledone() != 0
    }

    pub fn tested(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.tested() != 0
    }

    pub fn keep_centrum(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.keepcentrum() != 0
    }

    pub fn new_merge(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.newmerge() != 0
    }

    pub fn degenerate(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.degenerate() != 0
    }

    pub fn redundant(&self) -> bool {
        let face = unsafe { self.raw_ref() };
        face.redundant() != 0
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