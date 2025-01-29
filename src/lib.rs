#![doc = include_str!("../README.md")]

use std::{cell::{RefCell, UnsafeCell}, marker::PhantomData, rc::Rc};

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
    pub fn compute(&mut self) -> Result<(), QhError> {
        unsafe { Qh::try_on_qh_mut(self, |qh| sys::qh_qhull(qh)) }
    }

    /// Check the output of the qhull instance
    pub fn check_output(&mut self) -> Result<(), QhError> {
        unsafe { Qh::try_on_qh_mut(self, |qh| sys::qh_check_output(qh)) }
    }

    pub fn check_points(&mut self) -> Result<(), QhError> {
        unsafe {
            Qh::try_on_qh_mut(self, |qh| {
                println!("qh_check_points!!!");
                sys::qh_check_points(qh)
            })
        }
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
        let mut current = Facet::from_ptr(
            unsafe { sys::qh_get_facet_list(self.qh.get() as *mut _) },
            self.dim,
        );

        std::iter::from_fn(move || current.take().map(|v| {
            current = v.next();
            v
        }))
    }

    /// Get all the facets in the hull in reverse order
    ///
    /// See [`Qh::all_facets`] for more information.
    pub fn all_facets_rev(&self) -> impl Iterator<Item = Facet> {
        let mut current = Facet::from_ptr(
            unsafe { sys::qh_get_facet_tail(self.qh.get() as *mut _) },
            self.dim,
        );

        std::iter::from_fn(move || current.take().map(|v| {
            current = v.previous();
            v
        }))
    }

    /// Get the facets in the hull
    ///
    /// # Remarks
    /// * this function will not return the sentinel face, which is the last face in the list of facets.
    ///   To get it, use the [`Qh::all_facets`] function.
    pub fn facets(&self) -> impl Iterator<Item = Facet> {
        self.all_facets().filter(|f| !f.is_sentinel())
    }

    pub fn all_vertices(&self) -> impl Iterator<Item = Vertex> {
        let mut current = Vertex::from_ptr(
            unsafe { sys::qh_get_vertex_list(self.qh.get() as *mut _) },
            self.dim,
        );

        std::iter::from_fn(move || current.take().map(|v| {
            current = v.next();
            v
        }))
    }

    pub fn all_vertices_rev(&self) -> impl Iterator<Item = Vertex> {
        let mut current = Vertex::from_ptr(
            unsafe { sys::qh_get_vertex_tail(self.qh.get() as *mut _) },
            self.dim,
        );

        std::iter::from_fn(move || current.take().map(|v| {
            current = v.previous();
            v
        }))
    }

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

    /// Try a function on the qhull instance
    ///
    /// This function provides a way to access and possibly modify the qhull instance.  
    /// You should use only this function to access the qhull instance, as it provides a way to handle errors.
    ///
    /// # Safety
    /// This function is unsafe because it provides a way to access and possibly modify the qhull instance.
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
    /// unsafe {
    ///     Qh::try_on_qh_mut(&mut qh, |qh| {
    ///         sys::qh_qhull(qh)
    ///     }).unwrap();
    /// }
    /// ```
    ///
    /// It is advised to call as few Qhull fallible functions as possible in order to better locate the source of the error and avoid mistakes. For example:
    /// ```
    /// # use qhull::*;
    /// # let mut qh = Qh::builder()
    /// #     .build_from_iter([
    /// #         [0.0, 0.0],
    /// #         [1.0, 0.0],
    /// #         [0.0, 1.0],
    /// #         [0.25, 0.25]
    /// #    ]).unwrap();
    /// unsafe {
    ///     Qh::try_on_qh_mut(&mut qh, |qh| {
    ///         sys::qh_qhull(qh)
    ///     }).expect("qhull computation failed");
    ///
    ///     Qh::try_on_qh_mut(&mut qh, |qh| {
    ///         sys::qh_check_output(qh)
    ///     }).expect("qhull output check failed");
    /// }
    /// ```
    ///
    pub unsafe fn try_on_qh<'b, R>(
        qh: &'b Qh,
        f: impl FnOnce(*const sys::qhT) -> R,
    ) -> Result<R, QhError<'b>> {
        unsafe { QhError::try_on_raw(qh.qh.get(), &mut qh.buffers.borrow_mut().err_file, |p| f(p)) }
    }

    pub unsafe fn try_on_qh_mut<'b, R>(
        qh: &'b mut Qh,
        f: impl FnOnce(*mut sys::qhT) -> R,
    ) -> Result<R, QhError<'b>> {
        unsafe { QhError::try_on_raw(qh.qh.get(), &mut qh.buffers.borrow_mut().err_file, f) }
    }

    /// Get the pointer to the raw qhT instance
    ///
    /// # Warning
    /// Prefer using the [`Qh::try_on_qh`] when calling a fallible qhull function.
    pub unsafe fn raw_ptr(qh: &Qh) -> *const sys::qhT {
        qh.qh.get()
    }

    /// Get the mutable pointer to the raw qhT instance
    ///
    /// # Warning
    /// Prefer using the [`Qh::try_on_qh_mut`] when calling a fallible qhull function.
    pub unsafe fn raw_ptr_mut(qh: &mut Qh) -> *mut sys::qhT {
        qh.qh.get_mut()
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
    good_point_coords: Option<Rc<Vec<f64>>>,
    good_vertex_coords: Option<Rc<Vec<f64>>>,
    first_point: Option<Rc<Vec<f64>>>,
    upper_threshold: Option<Rc<Vec<f64>>>,
    lower_threshold: Option<Rc<Vec<f64>>>,
    upper_bound: Option<Rc<Vec<f64>>>,
    lower_bound: Option<Rc<Vec<f64>>>,
    feasible_point: Option<Rc<Vec<f64>>>,
    feasible_string: Option<Rc<Vec<core::ffi::c_char>>>,
    near_zero: Option<Rc<Vec<f64>>>,
}
