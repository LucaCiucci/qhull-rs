use std::{fmt::Debug, marker::PhantomData, ops::Not};

use crate::{helpers::QhTypeRef, sys, Qh, QhError, Set};

/// A vertex of the convex hull
///
/// This is a reference to the underlying qhull [`vertexT`](qhull_sys::vertexT).
#[derive(Clone, Copy)]
pub struct Vertex<'a> {
    ptr: *mut sys::vertexT,
    dim: usize,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Debug for Vertex<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vertex")
            .field("id", &self.id())
            .field("visit_id", &self.visit_id())
            .field("point", &self.point())
            .field(
                "neighbors",
                &self
                    .neighbors()
                    .iter()
                    .map(|n| n.iter().map(|v| v.id()).collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl<'a> Vertex<'a> {
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

    /// Get the index of the vertex in the input points
    ///
    /// Returns none if the vertex:
    /// - is a sentinel
    /// - has no coordinates
    /// - coordinates do not belong to the original set of points
    ///
    /// Use [`Vertex::index_unchecked`] if you are sure that the vertex has coordinates
    pub fn index(&self, qh: &Qh) -> Option<usize> {
        debug_assert_eq!(qh.dim, unsafe { sys::qh_get_hull_dim(qh.qh.get()) as usize });

        let first_ptr = unsafe { sys::qh_get_first_point(qh.qh.get()) as *const f64 };
        let end_ptr = unsafe { first_ptr.add(sys::qh_get_num_points(qh.qh.get()) as usize * qh.dim) };

        // perform some additional checks if we own the coordinates
        if let Some(coords_holder) = qh.coords_holder.as_ref() {
            debug_assert_eq!(first_ptr, coords_holder.as_slice().as_ptr());
            debug_assert_eq!(end_ptr, unsafe { coords_holder.as_slice().as_ptr().add(coords_holder.len()) });
        }

        if self.is_sentinel() {
            return None;
        }

        let current_ptr = self.point()?.as_ptr();

        if current_ptr < first_ptr || current_ptr >= end_ptr {
            return None;
        } else {
            let diff = current_ptr as usize - first_ptr as usize;
            let point_size = std::mem::size_of::<f64>() * qh.dim;
            debug_assert_eq!(diff % point_size, 0);
            let index = diff / point_size;
            debug_assert!(index < unsafe { sys::qh_get_num_points(qh.qh.get()) as usize });
            Some(index)
        }
    }

    /// Get the id of the point that the vertex represents
    ///
    /// This method wraps the [`qhull_sys::qh_pointid`] function.
    pub fn point_id<'b>(&self, qh: &'b Qh) -> Result<i32, QhError<'b>> {
        unsafe {
            let ptr = self.raw_ref().point;
            Qh::try_on_qh(&qh, |qh| {
                qhull_sys::qh_pointid(qh as *mut _, ptr as *mut _)
            })
        }
    }

    /// Get the index of the vertex in the input points without any checks
    ///
    /// Note that the this might return and invalid index or overflow if the vertex does not belong to the original set of points
    /// (e.g. is a sentinel or has no coordinates)
    pub fn index_unchecked(&self, qh: &Qh) -> usize {
        let first_ptr = unsafe { (*qh.qh.get()).first_point as *const f64 };
        let current_ptr = self.point().map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
        let diff = current_ptr as usize - first_ptr as usize;
        let point_size = std::mem::size_of::<f64>() * qh.dim;
        debug_assert_eq!(diff % point_size, 0);
        let index = diff / point_size;
        index
    }

    /// The dimension of the vertex
    ///
    /// This corresponds to the number of coordinates of the vertex
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Get the next vertex in the list
    pub fn next(&self) -> Option<Vertex<'a>> {
        let vertex = unsafe { self.raw_ref() };
        Self::from_ptr(vertex.next, self.dim())
    }

    /// Get the previous vertex in the list
    pub fn previous(&self) -> Option<Vertex<'a>> {
        let vertex = unsafe { self.raw_ref() };
        Self::from_ptr(vertex.previous, self.dim())
    }

    /// Get the coordinates of the vertex
    ///
    /// A vertex might not have coordinates, for example if it is a sentinel.
    pub fn point(&self) -> Option<&'a [f64]> {
        unsafe {
            let vertex = self.raw_ref();
            vertex
                .point
                .is_null()
                .not()
                .then(|| std::slice::from_raw_parts(vertex.point, self.dim()))
        }
    }

    /// Qhull id of the vertex
    ///
    /// # Warning
    /// This is not the index of the vertex in the input points, use [`Vertex::index`] for that.
    pub fn id(&self) -> u32 {
        let vertex = unsafe { self.raw_ref() };
        vertex.id
    }

    /// Visit id of the vertex
    pub fn visit_id(&self) -> u32 {
        let vertex = unsafe { self.raw_ref() };
        vertex.visitid
    }

    /// Get the neighbors of the vertex
    pub fn neighbors(&self) -> Option<Set<'a, Vertex<'a>>> {
        let vertex = unsafe { self.raw_ref() };
        Set::maybe_new(vertex.neighbors, self.dim())
    }
}

impl<'a> QhTypeRef for Vertex<'a> {
    type FFIType = sys::vertexT;

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