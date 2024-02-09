#![doc = include_str!("../README.md")]

use std::{marker::PhantomData, rc::Rc};

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
    qh: sys::qhT,
    _coords_holder: Option<Vec<f64>>,
    dim: usize,
    buffers: IOBuffers,
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
        unsafe { Qh::try_on_qh(self, |qh| sys::qh_qhull(qh)) }
    }

    /// Check the output of the qhull instance
    pub fn check_output(&mut self) -> Result<(), QhError> {
        unsafe { Qh::try_on_qh(self, |qh| sys::qh_check_output(qh)) }
    }

    pub fn check_points(&mut self) -> Result<(), QhError> {
        unsafe { Qh::try_on_qh(self, |qh| {
            println!("qh_check_points!!!");
            sys::qh_check_points(qh) })
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

        let mut builder = QhBuilder::default();
        unsafe {
            builder = builder.with_configure(|qh| {
                Self::try_on_qh(qh, |qh| {
                    // TODO implement all the required options and test
                    qh.DELAUNAY = true as _;
                    qh.DELAUNAY = true as _;
                    qh.SCALElast = true as _;
                    qh.TRIangulate = true as _;
                    qh.KEEPcoplanar = true as _;
                })
            })
        };

        builder.build_managed(dim, coords)
    }

    /// Get all the faces in the hull
    ///
    /// # Remarks
    /// * this function will also return the sentinel face, which is the last face in the list of faces.
    /// To avoid it, use the [`Qh::faces`] function or just [`filter`](std::iter::Iterator::filter) the iterator
    /// checking for [`Face::is_sentinel`].
    pub fn all_faces(&self) -> FaceIterator {
        unsafe {
            let list = sys::qh_get_facet_list(&self.qh);
            FaceIterator::new(Face::from_ptr(list, self.dim))
        }
    }

    /// Get the faces in the hull
    ///
    /// # Remarks
    /// * this function will not return the sentinel face, which is the last face in the list of faces.
    /// To get it, use the [`Qh::all_faces`] function.
    pub fn faces(&self) -> impl Iterator<Item = Face> {
        self.all_faces().filter(|f| !f.is_sentinel())
    }

    pub fn simplices(&self) -> impl Iterator<Item = Face> {
        self.faces().filter(|f| f.simplicial())
    }

    pub fn num_faces(&self) -> usize {
        unsafe { sys::qh_get_num_facets(&self.qh) as _ }
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
    ///     Qh::try_on_qh(&mut qh, |qh| {
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
    ///     Qh::try_on_qh(&mut qh, |qh| {
    ///         sys::qh_qhull(qh)
    ///     }).expect("qhull computation failed");
    ///
    ///     Qh::try_on_qh(&mut qh, |qh| {
    ///         sys::qh_check_output(qh)
    ///     }).expect("qhull output check failed");
    /// }
    /// ```
    ///
    pub unsafe fn try_on_qh<'b, R>(
        qh: &'b mut Qh,
        f: impl FnOnce(&mut sys::qhT) -> R,
    ) -> Result<R, QhError<'b>> {
        unsafe { QhError::try_on_raw(&mut qh.qh, &mut qh.buffers.err_file, f) }
    }
}

impl<'a> Drop for Qh<'a> {
    fn drop(&mut self) {
        unsafe {
            sys::qh_freeqhull(&mut self.qh, !sys::qh_ALL);
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
    feasible_string: Option<Rc<Vec<i8>>>,
    near_zero: Option<Rc<Vec<f64>>>,
}