#![doc = include_str!("../README.md")]

use std::{cell::{RefCell, UnsafeCell}, ffi::CString, marker::PhantomData};

use helpers::{prepare_delaunay_points, CollectedCoords, QhTypeRef};
use io_buffers::IOBuffers;
pub use qhull_sys as sys;

mod error;
pub mod helpers;
pub mod io_buffers;
pub mod tmp_file;
pub use error::*;
mod builder;
pub use builder::*;
mod types;
pub use types::*;
pub mod examples;

/// A Qhull instance
///
/// This struct is the main interface to the qhull library.
/// It provides a way to compute the convex hull of a set of points and to access the results.
///
/// See the main [`crate` documentation](crate) and the [`examples`] module/folder for some examples.
pub struct Qh<'a> {
    qh: UnsafeCell<sys::qhT>,
    coords_holder: Option<Vec<f64>>,
    dim: usize,
    buffers: RefCell<IOBuffers>,
    owned_values: OwnedValues,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Qh<'a> {
    /// Create a new builder
    pub fn builder() -> QhBuilder {
        QhBuilder::default()
    }

    /// Compute the convex hull
    ///
    /// Wraps [`qhull_sys::qh_qhull`],
    pub fn compute(&mut self) -> Result<(), QhError> {
        let qh = unsafe { Qh::raw_ptr(self) };
        unsafe { QhError::try_1(
            qh,
            &mut self.buffers().borrow_mut().err_file,
            sys::qh_qhull,
            (qh,),
        ) }
    }

    /// Prepare the output of the qhull instance
    ///
    /// Wraps [`qhull_sys::qh_prepare_output`],
    pub fn prepare_output(&mut self) -> Result<(), QhError> {
        let qh = unsafe { Qh::raw_ptr(self) };
        unsafe { QhError::try_1(
            qh,
            &mut self.buffers().borrow_mut().err_file,
            sys::qh_prepare_output,
            (qh,),
        ) }
    }

    /// Check the output of the qhull instance
    ///
    /// Wraps [`qhull_sys::qh_check_output`],
    pub fn check_output(&mut self) -> Result<(), QhError> {
        let qh = unsafe { Qh::raw_ptr(self) };
        unsafe { QhError::try_1(
            qh,
            &mut self.buffers().borrow_mut().err_file,
            sys::qh_check_output,
            (qh,),
        ) }
    }

    pub fn check_points(&mut self) -> Result<(), QhError> {
        let qh = unsafe { Qh::raw_ptr(self) };
        unsafe { QhError::try_1(
            qh,
            &mut self.buffers().borrow_mut().err_file,
            sys::qh_check_points,
            (qh,),
        ) }
    }

    /// Creates a new Delaunay triangulation
    ///
    /// See the `examples` directory for an example.
    pub fn new_delaunay<I>(points: impl IntoIterator<Item = I>) -> Result<Self, QhError<'static>>
    where
        I: IntoIterator<Item = f64>,
    {
        let CollectedCoords {
            coords,
            count: _,
            dim,
        } = prepare_delaunay_points(points);

        // TODO check correctness, use qdelaunay as reference
        QhBuilder::default()
            .delaunay(true)
            .upper_delaunay(true)
            .scale_last(true)
            .triangulate(true)
            .keep_coplanar(true)
            .build_managed(dim, coords)
    }

    /// Get all the facets in the hull
    ///
    /// # Remarks
    /// * this function will also return the sentinel face, which is the last face in the list of facets.
    ///   To avoid it, use the [`Qh::facets`] function or just [`filter`](std::iter::Iterator::filter) the iterator
    ///   checking for [`Facet::is_sentinel`].
    pub fn all_facets(&self) -> impl Iterator<Item = Facet> {
        /// Iterator for facets with size hint
        struct FacetIter<'a> {
            front: Option<Facet<'a>>,
            back: Option<Facet<'a>>,
            remaining: usize,
        }

        impl<'a> Iterator for FacetIter<'a> {
            type Item = Facet<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                if self.remaining == 0 {
                    return None;
                }
                let front = self.front.take()?;
                self.front = front.next();
                debug_assert!(self.remaining > 0);
                self.remaining = self.remaining.max(1) - 1;
                Some(front)
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = self.remaining;
                (len, Some(len))
            }
        }

        impl<'a> DoubleEndedIterator for FacetIter<'a> {
            fn next_back(&mut self) -> Option<Self::Item> {
                if self.remaining == 0 {
                    return None;
                }
                let back = self.back.take()?;
                self.back = back.previous();
                debug_assert!(self.remaining > 0);
                self.remaining = self.remaining.max(1) - 1;
                Some(back)
            }
        }
        
        let first = Facet::from_ptr(
            unsafe { sys::qh_get_facet_list(self.qh.get() as *mut _) },
            self.dim,
        );
        let last = Facet::from_ptr(
            unsafe { sys::qh_get_facet_tail(self.qh.get() as *mut _) },
            self.dim,
        );
        FacetIter {
            front: first,
            back: last,
            remaining: self.num_facets(),
        }
    }

    /// Get the facets in the hull
    ///
    /// # Remarks
    /// * this function will not return the sentinel face, which is the last face in the list of facets.
    ///   To get it, use the [`Qh::all_facets`] function.
    pub fn facets(&self) -> impl Iterator<Item = Facet> {
        self.all_facets().filter(|f| !f.is_sentinel())
    }

    /// Get all the vertices in the hull
    ///
    /// # Remarks
    /// * This function will return all vertices, including the sentinel vertex.
    pub fn all_vertices(&self) -> impl Iterator<Item = Vertex> {
        /// Iterator for vertices with size hint
        struct VertexIter<'a> {
            front: Option<Vertex<'a>>,
            back: Option<Vertex<'a>>,
            remaining: usize,
        }

        impl<'a> Iterator for VertexIter<'a> {
            type Item = Vertex<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                if self.remaining == 0 {
                    return None;
                }
                let front = self.front.take()?;
                self.front = front.next();
                debug_assert!(self.remaining > 0);
                self.remaining = self.remaining.max(1) - 1;
                Some(front)
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = self.remaining;
                (len, Some(len))
            }
        }

        impl<'a> DoubleEndedIterator for VertexIter<'a> {
            fn next_back(&mut self) -> Option<Self::Item> {
                if self.remaining == 0 {
                    return None;
                }
                let back = self.back.take()?;
                self.back = back.previous();
                debug_assert!(self.remaining > 0);
                self.remaining = self.remaining.max(1) - 1;
                Some(back)
            }
        }

        let first = Vertex::from_ptr(
            unsafe { sys::qh_get_vertex_list(self.qh.get() as *mut _) },
            self.dim,
        );
        let last = Vertex::from_ptr(
            unsafe { sys::qh_get_vertex_tail(self.qh.get() as *mut _) },
            self.dim,
        );
        VertexIter {
            front: first,
            back: last,
            remaining: self.num_vertices(),
        }
    }

    /// Get the vertices in the hull
    ///
    /// # Remarks
    /// * This function will not return the sentinel vertex.
    pub fn vertices(&self) -> impl Iterator<Item = Vertex> {
        self.all_vertices().filter(|v| !v.is_sentinel())
    }

    /// Number of facets in the hull (sentinel excluded)
    ///
    /// # Example
    /// ```
    /// # use qhull::*;
    /// # let mut qh = Qh::builder()
    /// #     .build_from_iter([
    /// #         [0.0, 0.0],
    /// #         [1.0, 0.0],
    /// #         [0.0, 1.0],
    /// #         [0.25, 0.25]
    /// #    ]).unwrap();
    /// assert_eq!(qh.num_facets(), qh.facets().count());
    /// ```
    pub fn num_facets(&self) -> usize {
        unsafe { sys::qh_get_num_facets(self.qh.get()) as _ }
    }

    /// Number of vertices in the hull (sentinel excluded)
    ///
    /// # Example
    /// ```
    /// # use qhull::*;
    /// # let mut qh = Qh::builder()
    /// #     .build_from_iter([
    /// #         [0.0, 0.0],
    /// #         [1.0, 0.0],
    /// #         [0.0, 1.0],
    /// #         [0.25, 0.25]
    /// #    ]).unwrap();
    /// assert_eq!(qh.num_vertices(), qh.vertices().count());
    /// ```
    pub fn num_vertices(&self) -> usize {
        unsafe { sys::qh_get_num_vertices(self.qh.get()) as _ }
    }

    pub fn simplices(&self) -> impl Iterator<Item = Facet> {
        self.facets().filter(|f| f.simplicial())
    }

    /// Get the pointer to the raw qhT instance
    ///
    /// # Warning
    /// Always use a try function (e.g. [`QhError::try_1`]) when calling a fallible qhull function,
    /// but not on non-fallible functions such as [`qhull_sys::qh_init_A`] since it would be invalid.
    pub unsafe fn raw_ptr(qh: &Qh) -> *mut sys::qhT {
        qh.qh.get()
    }

    pub fn buffers(&self) -> &RefCell<IOBuffers> {
        &self.buffers
    }
}

impl<'a> Drop for Qh<'a> {
    fn drop(&mut self) {
        unsafe {
            sys::qh_freeqhull(self.qh.get_mut(), !sys::qh_ALL);
        }
    }
}

#[derive(Default)]
#[allow(unused)]
struct OwnedValues {
    good_point_coords: Option<Vec<f64>>,
    good_vertex_coords: Option<Vec<f64>>,
    first_point: Option<Vec<f64>>,
    upper_threshold: Option<Vec<f64>>,
    lower_threshold: Option<Vec<f64>>,
    upper_bound: Option<Vec<f64>>,
    lower_bound: Option<Vec<f64>>,
    feasible_point: Option<Vec<f64>>,
    feasible_string: Option<CString>,
    near_zero: Option<Vec<f64>>,
}
