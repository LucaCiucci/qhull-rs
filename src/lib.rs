#![doc = include_str!("../README.md")]

use std::{marker::PhantomData, ptr};

use helpers::{collect_coords, prepare_delaunay_points, CollectedCoords};
pub use qhull_sys as sys;

pub mod helpers;
mod types; pub use types::*;

mod builder; pub use builder::*;


/// A Qhull instance
///
/// # Example
/// ```
/// # use qhull::*;
/// let qh = Qh::builder(2)
///     .build_from_iter([
///         [0.0, 0.0],
///         [1.0, 0.0],
///         [0.0, 1.0],
///         [0.25, 0.25],
///     ]);
///
/// assert_eq!(qh.num_faces(), 3);
///
/// for simplex in qh.simplices() {
///     println!("{:?}", simplex.vertices().map(|v| v.id()).collect::<Vec<_>>());
/// }
/// ```
pub struct Qh<'a> {
    qh: sys::qhT,
    _coords_holder: Option<Vec<f64>>,
    dim: usize,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Qh<'a> {
    /// Create a new builder
    ///
    /// # Example
    /// ```
    /// # use qhull::*;
    /// let qh = Qh::builder(2)
    ///     .build_from_iter([
    ///         [0.0, 0.0],
    ///         [1.0, 0.0],
    ///         [0.0, 1.0],
    ///         [0.25, 0.25],
    ///     ]);
    ///
    /// assert_eq!(qh.num_faces(), 3);
    /// ```
    pub fn builder(dim: usize) -> QhBuilder {
        QhBuilder::new(dim)
    }

    /// Creates a new Delaunay triangulation
    ///
    /// # Example
    /// ```
    /// # use qhull::*;
    /// let qh = Qh::new_delaunay([
    ///     [0.0, 0.0],
    ///     [1.0, 0.0],
    ///     [0.0, 1.0],
    ///     [0.25, 0.25],
    /// ]);
    ///
    /// let mut simpleces = qh
    ///     .simplices()
    ///     .map(|f| f.vertices().map(|v| v.id() - 1).collect::<Vec<_>>())
    ///     .collect::<Vec<_>>();
    ///
    /// simpleces.iter_mut().for_each(|s| s.sort());
    /// simpleces.sort();
    /// assert_eq!(simpleces, vec![
    ///     vec![0, 1, 2],
    ///     vec![0, 1, 3],
    ///     vec![0, 2, 3],
    ///     vec![1, 2, 3],
    /// ]);
    /// ```
    pub fn new_delaunay<I>(
        points: impl IntoIterator<Item = I>,
    ) -> Self
    where
        I: IntoIterator<Item = f64>,
    {
        let CollectedCoords {
            coords,
            count: _,
            dim,
        } = prepare_delaunay_points(points);

        let mut builder = QhBuilder::new(dim);
        unsafe {
            builder.configure(|qh| {
                // TODO implement all the required options and test
                qh.DELAUNAY = true as _;
                qh.DELAUNAY = true as _;
                qh.SCALElast = true as _;
                qh.TRIangulate = true as _;
                qh.KEEPcoplanar = true as _;
            })
        };

        builder.build_managed(coords)
    }

    /// Check the output of the qhull instance
    ///
    /// This function uses [`sys::qh_check_output`] to check the output of the qhull instance.
    pub fn check(&mut self) {
        unsafe {
            sys::qh_check_output(&mut self.qh);
            // TODO check error flags
        }
    }

    pub fn faces(&self) -> FaceIterator {
        unsafe {
            let list = sys::qh_get_facet_list(&self.qh);
            FaceIterator::new(list, self.dim)
        }
    }

    pub fn simplices(&self) -> impl Iterator<Item = Face> {
        self
            .faces()
            .take(self.num_faces()) // TODO last is empty ???
            .filter(|f| f.simplicial())
    }

    pub fn num_faces(&self) -> usize {
        unsafe {
            sys::qh_get_num_facets(&self.qh) as _
        }
    }
}

impl<'a> Drop for Qh<'a> {
    fn drop(&mut self) {
        unsafe {
            sys::qh_freeqhull(&mut self.qh, !sys::qh_ALL);
        }
    }
}