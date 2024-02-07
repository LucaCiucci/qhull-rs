use crate::*;

use self::helpers::CollectedCoords;

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
/// let qh = QhBuilder::new(2)
///     .build(&mut points);
/// 
/// assert_eq!(qh.num_faces(), 3);
/// ```
#[must_use]
pub struct QhBuilder {
    qh: Option<sys::qhT>,
    dim: usize,
}

impl QhBuilder {
    pub fn new(dim: usize) -> Self {
        assert!(dim > 0, "dim must be > 0");

        unsafe {
            let mut qh: sys::qhT = std::mem::zeroed();

            // from C:
            //#define stdin  (__acrt_iob_func(0))
            //#define stdout (__acrt_iob_func(1))
            //#define stderr (__acrt_iob_func(2))
            let stdin = sys::__acrt_iob_func(0);
            let stdout = sys::__acrt_iob_func(1);
            let stderr = sys::__acrt_iob_func(2);

            sys::qh_init_A(
                &mut qh,
                stdin,
                stdout,
                stderr,
                0,
                ptr::null_mut(),
            );

            Self { qh: Some(qh), dim }
        }
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
    /// let qh = QhBuilder::new(2)
    ///     .build(&mut points);
    /// 
    /// assert_eq!(qh.num_faces(), 3);
    /// ```
    pub fn build<'a>(
        mut self,
        points: &'a mut [f64],
    ) -> Qh<'a> {
        assert_eq!(points.len() % self.dim, 0, "points.len() % dim != 0");
        let num_points = points.len() / self.dim;

        unsafe {
            let points_ptr = points.as_ptr() as *mut f64;
            let mut qh = self.qh.take().unwrap();

            assert_eq!(points.len(), num_points * self.dim, "points.len() != num_points * dim");

            sys::qh_init_B(
                &mut qh,
                points_ptr,
                num_points as _,
                self.dim as _,
                false as _,
            );

            sys::qh_qhull(&mut qh);

            Qh {
                qh,
                _coords_holder: None,
                dim: self.dim,
                phantom: PhantomData,
            }
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
    /// let qh = QhBuilder::new(2)
    ///     .build_managed(vec![
    ///         0.0, 0.0,
    ///         1.0, 0.0,
    ///         0.0, 1.0,
    ///         0.25, 0.25,
    /// ]);
    /// 
    /// assert_eq!(qh.num_faces(), 3);
    /// ```
    pub fn build_managed(
        self,
        points: impl ToOwned<Owned = Vec<f64>>,
    ) -> Qh<'static> {
        let mut points = points.to_owned();
        let points_ptr = points.as_mut_ptr();
        let mut qh: Qh<'static> = self.build(unsafe {
            std::slice::from_raw_parts_mut(points_ptr, points.len())
        });
        assert!(qh._coords_holder.is_none());
        qh._coords_holder = Some(points);
        qh
    }

    /// Build a Qhull instance from an iterator of points
    ///
    /// # Example
    /// ```
    /// # use qhull::*;
    /// let qh = QhBuilder::new(2)
    ///     .build_from_iter([
    ///         [0.0, 0.0],
    ///         [1.0, 0.0],
    ///         [0.0, 1.0],
    ///         [0.25, 0.25],
    ///     ]);
    ///
    /// assert_eq!(qh.num_faces(), 3);
    /// ```
    pub fn build_from_iter<I>(
        self,
        points: impl IntoIterator<Item = I>,
    ) -> Qh<'static>
    where
        I: IntoIterator<Item = f64>,
    {
        let CollectedCoords {
            coords,
            count: _,
            dim,
        } = collect_coords(points);
        assert_eq!(dim, self.dim, "points have wrong dimensionality");
        self.build_managed(coords)
    }

    /// Configure the qhull instance with a closure
    ///
    /// # Example
    /// ```
    /// # use qhull::*;
    /// let _builder = unsafe {
    ///     QhBuilder::new(2)
    ///         .with_configure(|qh| {
    ///            qh.DELAUNAY = true as _;
    ///        })
    /// };
    /// ```
    pub unsafe fn with_configure(
        mut self,
        configurator: impl FnOnce(&mut sys::qhT) -> (),
    ) -> Self {
        self.configure(configurator);
        self
    }

    /// Configure the qhull instance with a closure
    ///
    /// # Example
    /// ```
    /// # use qhull::*;
    /// # let mut qh = QhBuilder::new(2);
    /// let value = unsafe {
    ///     qh.configure(|qh| {
    ///        qh.DELAUNAY = true as _;
    ///        qh.DELAUNAY != 0
    ///     })
    /// };
    /// assert_eq!(value, true);
    /// ```
    pub unsafe fn configure<R>(
        &mut self,
        configurator: impl FnOnce(&mut sys::qhT) -> R,
    ) -> R {
        configurator(self.qh.as_mut().unwrap())
    }
}