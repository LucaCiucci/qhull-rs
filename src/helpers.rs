use std::{ffi::CString, os::raw::{c_char, c_int}};

/// A trait for types that can be created from a pointer to a C type and a dimension.
pub trait QhTypeRef: Sized {
    type FFIType;

    fn from_ptr(ptr: *mut Self::FFIType, dim: usize) -> Option<Self>;

    /// Returns a raw pointer to the C type.
    ///
    /// # Safety
    /// * Users must not invalidate the instance or
    /// make it inconsistent with the current flow of the program.
    unsafe fn raw_ptr(&self) -> *mut Self::FFIType;

    /// Returns a reference to the C type.
    ///
    /// # Safety
    /// * Users must not invalidate the instance or
    /// make it inconsistent with the current flow of the program.
    unsafe fn raw_ref(&self) -> &Self::FFIType {
        unsafe { &*self.raw_ptr() }
    }

    fn dim(&self) -> usize;
}

pub struct CollectedCoords {
    pub coords: Vec<f64>,
    pub count: usize,
    pub dim: usize,
}

/// Collects coordinates from an iterator of points.
///
/// # Example
/// ```
/// # use qhull::helpers::*;
/// // 2D triangle
/// let CollectedCoords {
///     coords,
///     count,
///     dim,
/// } = collect_coords([
///         [0.0, 0.0],
///         [1.0, 0.0],
///         [0.0, 2.0],
/// ]);
/// assert_eq!(coords, vec![0.0, 0.0, 1.0, 0.0, 0.0, 2.0]);
/// assert_eq!(count, 3);
/// assert_eq!(dim, 2);
/// ```
pub fn collect_coords<I>(points: impl IntoIterator<Item = I>) -> CollectedCoords
where
    I: IntoIterator<Item = f64>,
{
    let mut dim: Option<usize> = None;
    let mut coords: Vec<f64> = Vec::new();
    let mut pt: Vec<f64> = Vec::new();
    for point in points.into_iter() {
        pt.clear();
        pt.extend(point.into_iter());
        if let Some(d) = dim {
            assert_eq!(pt.len(), d, "points have different dimensions");
        } else {
            dim = Some(pt.len());
        }
        coords.extend(pt.iter());
    }
    drop(pt);
    assert!(!coords.is_empty(), "no points");
    let dim = dim.unwrap();
    debug_assert_eq!(coords.len() % dim, 0);
    let count = coords.len() / dim;
    CollectedCoords { coords, count, dim }
}

/// Prepares points for Delaunay triangulation.
///
/// This function builds a paraboloid adding a "z" coordinate to each point.
///
/// # Example
/// ```
/// # use qhull::helpers::*;
/// let CollectedCoords {
///     coords,
///     count,
///     dim,
/// } = prepare_delaunay_points([[-1.0], [0.0], [1.0]]);
/// assert_eq!(coords, vec![-1.0, 1.0, 0.0, 0.0, 1.0, 1.0]);
/// assert_eq!(count, 3);
/// assert_eq!(dim, 2);
/// ```
pub fn prepare_delaunay_points<I>(points: impl IntoIterator<Item = I>) -> CollectedCoords
where
    I: IntoIterator<Item = f64>,
{
    let points = points
        .into_iter()
        .map(|point| point.into_iter().chain(std::iter::once(0.0)));
    let CollectedCoords {
        mut coords,
        count,
        dim,
    } = collect_coords(points);
    let orig_dim = dim - 1;

    let mut center: Vec<f64> = vec![0.0; orig_dim];
    let mut min_coords: Vec<f64> = vec![std::f64::MAX; orig_dim];
    let mut max_coords: Vec<f64> = vec![std::f64::MIN; orig_dim];

    for point in coords.windows(orig_dim + 1).step_by(orig_dim + 1) {
        for (i, coord) in point.iter().take(orig_dim).enumerate() {
            center[i] += coord;
            if *coord < min_coords[i] {
                min_coords[i] = *coord;
            }
            if *coord > max_coords[i] {
                max_coords[i] = *coord;
            }
        }
    }
    center.iter_mut().for_each(|coord| *coord /= count as f64);
    let widths: Vec<f64> = min_coords
        .iter()
        .zip(max_coords.iter())
        .map(|(min, max)| (max - min) / 2.0)
        .collect();

    // build paraboloid
    for point in 0..count {
        let point = &mut coords[point * dim..(point + 1) * dim];
        for i in 0..orig_dim {
            let d = (point[i] - center[i]) / widths[i];
            point[orig_dim] += d * d;
        }
    }

    CollectedCoords { coords, count, dim }
}

pub struct CArgs {
    args: Vec<CString>,
    args_ptr: Vec<*const c_char>,
}

impl CArgs {
    pub fn from_env() -> Self {
        let args: Vec<CString> = std::env::args().map(|arg| CString::new(arg).unwrap()).collect();
        let args_ptr: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();
        Self {
            args,
            args_ptr,
        }
    }

    pub fn argc_argv(&self) -> (c_int, *mut *mut c_char) {
        (self.args.len() as c_int, self.args_ptr.as_ptr() as *mut *mut c_char)
    }
}

#[macro_export]
macro_rules! __impl_qhull_program {
    ($main:ident) => {
        fn main() {
            std::process::exit(unsafe {
                let args = qhull::helpers::CArgs::from_env();
                let (argc, argv) = args.argc_argv();
                if argc <= 1 {
                    println!("This binary, provided by the qhull crate, uses the Qhull library:");
                    println!("{}\n", qhull::sys::QHULL_LICENSE_TEXT);
                }

                qhull::sys::$main(argc, argv)
            });
        }
    };
}