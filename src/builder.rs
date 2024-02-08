use std::{marker::PhantomData, ptr};

use crate::{helpers::{collect_coords, CollectedCoords}, io_buffers::IOBuffers, sys, Qh, QhError};

/// A builder for a Qhull instance
///
/// # Example
/// ```
/// # use qhull::*;
/// let mut points = [
///    0.0, 0.0,
///    1.0, 0.0,
///    0.0, 1.0,
///    0.25, 0.25,
/// ];
///
/// let qh = QhBuilder::new()
///     .build(2, &mut points)
///     .unwrap();
/// 
/// assert_eq!(qh.num_faces(), 3);
/// ```
#[must_use]
pub struct QhBuilder {
    dim: Option<usize>,
    capture_stdout: bool,
    capture_stderr: bool,
    compute: bool,
    configs: Vec<Box<dyn for<'a> Fn(&'a mut Qh) -> Result<(), QhError<'a>>>>,
}

impl QhBuilder {
    /// Create a new builder
    ///
    /// Default settings:
    /// * No [dimension hint](QhBuilder::dim)
    /// * [stdout](QhBuilder::capture_stdout) is not captured
    /// * [stderr](QhBuilder::capture_stderr) is captured
    /// * [compute](QhBuilder::compute) is `true`
    pub fn new() -> Self {
        Self {
            dim: None,
            capture_stdout: false,
            capture_stderr: true,
            compute: true,
            configs: Vec::new(),
        }
    }

    /// Sets a dimension hint for the data
    ///
    /// This is useful when you want to **assert** that the data
    /// has the correct dimensionality before building the Qhull instance.
    ///
    /// This build will panic if the dimensionality of the data does not match the hint.
    pub fn dim(mut self, dim: usize) -> Self {
        assert!(dim > 0, "dim must be > 0");
        self.dim = Some(dim);
        self
    }

    pub fn capture_stdout(mut self, capture: bool) -> Self {
        self.capture_stdout = capture;
        self
    }

    pub fn capture_stderr(mut self, capture: bool) -> Self {
        self.capture_stderr = capture;
        self
    }

    pub fn compute(mut self, compute: bool) -> Self {
        self.compute = compute;
        self
    }

    /// Build a Qhull instance
    ///
    /// # Example
    /// ```
    /// # use qhull::*;
    /// let mut points = [
    ///    0.0, 0.0,
    ///    1.0, 0.0,
    ///    0.0, 1.0,
    ///    0.25, 0.25,
    /// ];
    ///
    /// let qh = QhBuilder::new()
    ///     .build(2, &mut points).unwrap();
    /// 
    /// assert_eq!(qh.num_faces(), 3);
    /// ```
    ///
    /// # Panics
    /// * If the number of points is not divisible by the dimension
    /// * If the dimensionality of the points does not match the hint
    /// * Cannot create a tempfile for capturing stdout or stderr
    pub fn build<'a>(
        self,
        dim: usize,
        points: &'a mut [f64],
    ) -> Result<Qh<'a>, QhError> {
        if let Some(dim_hint) = self.dim {
            assert_eq!(dim, dim_hint, "data dimensionality does not match hint that was given with QhBuilder::dim");
        }

        assert_eq!(points.len() % dim, 0, "points.len() % dim != 0");
        let num_points = points.len() / dim;

        unsafe {
            let mut qh: sys::qhT = std::mem::zeroed();
            let buffers = IOBuffers::new(
                self.capture_stdout,
                self.capture_stderr,
            );

            // Note: this function cannot be called
            // inside of a try
            sys::qh_init_A(
                &mut qh,
                buffers.in_file(),
                buffers.out_file(),
                buffers.err_file(),
                0,
                ptr::null_mut(),
            );

            let mut qh = Qh {
                qh,
                _coords_holder: None,
                dim,
                buffers,
                phantom: PhantomData,
            };

            for config in self.configs {
                config(&mut qh).map_err(|e| e.into_static())?;
            }

            Qh::try_on_qh(
                &mut qh,
                |qh| {
                    sys::qh_init_B(
                        qh,
                        points.as_ptr() as *mut f64,
                        num_points as _,
                        dim as _,
                        false as _,
                    );
                },
            ).map_err(|e| e.into_static())?;

            if self.compute {
                qh.compute().map_err(|e| e.into_static())?;
                qh.check_output().map_err(|e| e.into_static())?;
            }

            Ok(qh)
        }
    }

    /// Build a Qhull instance with managed points
    ///
    /// This is useful when you want to keep the points alive
    /// for the lifetime of the Qhull instance.
    ///
    /// # Example
    /// ```
    /// # use qhull::*;
    ///
    /// let qh = QhBuilder::new()
    ///     .build_managed(2, vec![
    ///         0.0, 0.0,
    ///         1.0, 0.0,
    ///         0.0, 1.0,
    ///         0.25, 0.25,
    ///     ]).unwrap();
    /// 
    /// assert_eq!(qh.num_faces(), 3);
    /// ```
    pub fn build_managed(
        self,
        dim: usize,
        points: impl ToOwned<Owned = Vec<f64>>,
    ) -> Result<Qh<'static>, QhError<'static>> {
        let mut points = points.to_owned();
        let points_ptr = points.as_mut_ptr();
        let mut qh: Qh<'static> = self.build(dim, unsafe {
            std::slice::from_raw_parts_mut(points_ptr, points.len())
        })?;
        assert!(qh._coords_holder.is_none());
        qh._coords_holder = Some(points);
        Ok(qh)
    }

    /// Build a Qhull instance from an iterator of points
    ///
    /// # Example
    /// ```
    /// # use qhull::*;
    /// let qh = QhBuilder::new()
    ///     .build_from_iter([
    ///         [0.0, 0.0],
    ///         [1.0, 0.0],
    ///         [0.0, 1.0],
    ///         [0.25, 0.25],
    ///     ]).unwrap();
    ///
    /// assert_eq!(qh.num_faces(), 3);
    /// ```
    pub fn build_from_iter<I>(
        self,
        points: impl IntoIterator<Item = I>,
    ) -> Result<Qh<'static>, QhError<'static>>
    where
        I: IntoIterator<Item = f64>,
    {
        let CollectedCoords {
            coords,
            count: _,
            dim,
        } = collect_coords(points);
        self.build_managed(dim, coords)
    }

    /// Configure the qhull instance with a closure
    ///
    /// # Example
    /// ```
    /// # use qhull::*;
    /// let builder = unsafe {
    ///     QhBuilder::new()
    ///         .with_configure(|qh| {
    ///             Qh::try_on_qh(qh, |qh| {
    ///                 qh.DELAUNAY = true as _;
    ///             })
    ///         });
    /// };
    /// ```
    pub unsafe fn with_configure(
        mut self,
        configurator: impl for<'a> Fn(&'a mut Qh) -> Result<(), QhError<'a>> + 'static,
    ) -> Self {
        self.configs.push(Box::new(configurator));
        self
    }
}