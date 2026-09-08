use std::{fmt::Debug, marker::PhantomData, ops::Not};

use crate::{dbg_face_set, helpers::QhTypeRef, sys, Ridge, Set, Vertex};

/// A face of the convex hull
///
/// This is a reference to the underlying qhull [`facetT`](qhull_sys::facetT).
#[derive(Clone, Copy)]
pub struct Facet<'a> {
    ptr: *mut sys::facetT,
    dim: usize,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Debug for Facet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Face")
            .field("id", &self.id())
            .field("visit_id", &self.visit_id())
            .field("furthest_dist", &self.furthest_dist())
            .field("max_outside", &self.max_outside())
            .field("offset", &self.offset())
            .field("normal", &self.normal())
            .field("f", &"union { ... }")
            .field("center", &self.center())
            .field("previous", &self.previous().map(|f| f.id()))
            .field("next", &self.next().map(|f| f.id()))
            .field("vertices", &self.vertices())
            .field("ridges", &self.ridges())
            .field("neighbors", &dbg_face_set(self.neighbors()))
            .field("outside_set", &self.outside_set())
            .field("coplanar_set", &self.coplanar_set())
            .field("tricoplanar", &self.tricoplanar())
            .field("new_facet", &self.new_facet())
            .field("visible", &self.visible())
            .field("top_orient", &self.top_orient())
            .field("simplicial", &self.simplicial())
            .field("seen", &self.seen())
            .field("seen2", &self.seen2())
            .field("flipped", &self.flipped())
            .field("upper_delaunay", &self.upper_delaunay())
            .field("not_furthest", &self.not_furthest())
            .field("good", &self.good())
            .field("is_area", &self.is_area())
            .field("dup_ridge", &self.dup_ridge())
            .field("merge_ridge", &self.merge_ridge())
            .field("merge_ridge2", &self.merge_ridge2())
            .field("coplanar_horizon", &self.coplanar_horizon())
            .field("merge_horizon", &self.merge_horizon())
            .field("cycle_done", &self.cycle_done())
            .field("tested", &self.tested())
            .field("keep_centrum", &self.keep_centrum())
            .field("new_merge", &self.new_merge())
            .field("degenerate", &self.degenerate())
            .field("redundant", &self.redundant())
            .finish()
    }
}

impl<'a> Facet<'a> {
    /// Check if the vertex is a sentinel (id = 0)
    ///
    /// A sentinel is a special vertex that is used to mark the end of a list
    /// an it (should) have:
    /// - `id = 0`
    /// - `next = null`
    /// - `point = null`
    ///
    /// # Remarks
    /// * This method only checks the id of the vertex
    pub fn is_sentinel(&self) -> bool {
        self.id() == 0
    }

    /// The dimensionality of the face
    pub fn dim(&self) -> usize {
        self.dim
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

    pub fn normal(&self) -> Option<&'a [f64]> {
        unsafe {
            let face = self.raw_ref();
            face.normal
                .is_null()
                .not()
                .then(|| std::slice::from_raw_parts(face.normal, self.dim()))
        }
    }

    // TODO that union??

    pub fn center(&self) -> Option<&'a [f64]> {
        unsafe {
            let face = self.raw_ref();
            face.center
                .is_null()
                .not()
                .then(|| std::slice::from_raw_parts(face.center, self.dim()))
        }
    }

    pub fn previous(&self) -> Option<Facet<'a>> {
        let face = unsafe { self.raw_ref() };
        Self::from_ptr(face.previous, self.dim())
    }

    pub fn next(&self) -> Option<Facet<'a>> {
        let face = unsafe { self.raw_ref() };
        Self::from_ptr(face.next, self.dim())
    }

    pub fn vertices(&self) -> Option<Set<'a, Vertex<'a>>> {
        let face = unsafe { self.raw_ref() };
        Set::maybe_new(face.vertices, self.dim())
    }

    pub fn ridges(&self) -> Option<Set<'a, Ridge<'a>>> {
        if self.dim() == 0 {
            None
        } else {
            let face = unsafe { self.raw_ref() };
            Set::maybe_new(face.ridges, self.dim() - 1)
        }
    }

    pub fn neighbors(&self) -> Option<Set<'a, Facet<'a>>> {
        let face = unsafe { self.raw_ref() };
        Set::maybe_new(face.neighbors, self.dim())
    }

    pub fn outside_set(&self) -> Option<Set<'a, Vertex<'a>>> {
        let face = unsafe { self.raw_ref() };
        Set::maybe_new(face.outsideset, self.dim())
    }

    pub fn coplanar_set(&self) -> Option<Set<'a, Vertex<'a>>> {
        let face = unsafe { self.raw_ref() };
        Set::maybe_new(face.coplanarset, self.dim())
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

impl<'a> QhTypeRef for Facet<'a> {
    type FFIType = sys::facetT;

    fn from_ptr(ptr: *mut Self::FFIType, dim: usize) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self {
                ptr,
                dim,
                _marker: PhantomData,
            })
        }
    }

    unsafe fn raw_ptr(&self) -> *mut Self::FFIType {
        self.ptr
    }

    fn dim(&self) -> usize {
        self.dim()
    }
}

// TODO wrong, maybe we cannot implement DoubleEndedIterator
//impl<'a> DoubleEndedIterator for RefIterator<Face<'a>> {
//    fn next_back(&mut self) -> Option<Self::Item> {
//        if let Some(v) = self.0.take() {
//            self.0 = Face::previous(&v);
//            Some(v)
//        } else {
//            None
//        }
//    }
//}
