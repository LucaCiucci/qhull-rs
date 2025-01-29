use std::{cell::{RefCell, UnsafeCell}, marker::PhantomData, ptr, rc::Rc};

use crate::{
    helpers::{collect_coords, CollectedCoords},
    io_buffers::IOBuffers,
    sys, Qh, QhError,
};

type QhConfigurator = Box<dyn for<'b> Fn(&'b mut Qh) -> Result<(), QhError<'b>> + 'static>;

/// Builder for a Qhull instance
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
/// let qh = QhBuilder::default()
///     .build(2, &mut points)
///     .unwrap();
///
/// assert_eq!(qh.num_facets(), 3);
/// ```
#[must_use]
pub struct QhBuilder {
    dim: Option<usize>,
    capture_stdout: bool,
    capture_stderr: bool,
    compute: bool,
    check_output: bool,
    check_points: bool,
    configs: Vec<QhConfigurator>,
}

/// Default settings:
/// * No [dimension hint](QhBuilder::dim)
/// * [stdout](QhBuilder::capture_stdout) is not captured
/// * [stderr](QhBuilder::capture_stderr) is captured
/// * [compute](QhBuilder::compute) is `true`
impl Default for QhBuilder {
    fn default() -> Self {
        Self {
            dim: None,
            capture_stdout: false,
            capture_stderr: true,
            compute: true,
            check_output: false,
            check_points: false,
            configs: Vec::new(),
        }
    }
}

impl QhBuilder {
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

    /// Capture stdout
    ///
    /// When enabled, the output of the qhull library will be captured instead of printed to the console.
    pub fn capture_stdout(mut self, capture: bool) -> Self {
        self.capture_stdout = capture;
        self
    }

    /// Capture stderr
    ///
    /// When enabled, the error output of the qhull library will be captured instead of printed to the console.
    pub fn capture_stderr(mut self, capture: bool) -> Self {
        self.capture_stderr = capture;
        self
    }

    /// Set whether to compute the hull when building the Qhull instance
    ///
    /// When enabled, [`Qh::compute`] will be called.
    /// When disabled, you will have to call this method manually.
    ///
    /// # Example
    /// ```
    /// # use qhull::*;
    /// let points = [
    ///     [0.0, 0.0],
    ///     [1.0, 0.0],
    ///     [0.0, 1.0],
    ///     [0.25, 0.25],
    /// ];
    /// let qh = QhBuilder::default()
    ///     .compute(true) // this is the default
    ///     .build_from_iter(points)
    ///     .unwrap();
    /// assert_eq!(qh.num_facets(), 3);
    ///
    /// let mut qh = QhBuilder::default()
    ///     .compute(false)
    ///     .build_from_iter(points)
    ///     .unwrap();
    /// assert_eq!(qh.num_facets(), 0);
    /// qh.compute().unwrap();
    /// assert_eq!(qh.num_facets(), 3);
    /// ```
    pub fn compute(mut self, compute: bool) -> Self {
        self.compute = compute;
        self
    }

    /// Set whether to check the output when building the Qhull instance
    ///
    /// When enabled, [`Qh::check_output`] will be called after computing the hull.  
    /// If [`compute`](QhBuilder::compute) is disabled, this setting will have no effect.
    pub fn check_output(mut self, check: bool) -> Self {
        self.check_output = check;
        self
    }

    /// Set whether to check the points when building the Qhull instance
    ///
    /// When enabled, [`Qh::check_points`] will be called after computing the hull.
    /// If [`compute`](QhBuilder::compute) is disabled, this setting will have no effect.
    pub fn check_points(mut self, check: bool) -> Self {
        self.check_points = check;
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
    /// let qh = QhBuilder::default()
    ///     .build(2, &mut points).unwrap();
    ///
    /// assert_eq!(qh.num_facets(), 3);
    /// ```
    ///
    /// # Panics
    /// * If the number of points is not divisible by the dimension
    /// * If the dimensionality of the points does not match the hint
    /// * Cannot create a temporary file for capturing stdout or stderr
    pub fn build(self, dim: usize, points: &mut [f64]) -> Result<Qh, QhError> {
        if let Some(dim_hint) = self.dim {
            assert_eq!(
                dim, dim_hint,
                "data dimensionality does not match hint that was given with QhBuilder::dim"
            );
        }

        assert_eq!(points.len() % dim, 0, "points.len() % dim != 0");
        let num_points = points.len() / dim;

        unsafe {
            let mut qh: sys::qhT = std::mem::zeroed();
            let buffers = IOBuffers::new(self.capture_stdout, self.capture_stderr);

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
                qh: UnsafeCell::new(qh),
                coords_holder: None,
                dim,
                buffers: RefCell::new(buffers),
                owned_values: Default::default(),
                phantom: PhantomData,
            };

            for config in self.configs {
                config(&mut qh).map_err(|e| e.into_static())?;
            }

            Qh::try_on_qh_mut(&mut qh, |qh| {
                sys::qh_init_B(
                    qh,
                    points.as_ptr() as *mut f64,
                    num_points as _,
                    dim as _,
                    false as _,
                );
            })
            .map_err(|e| e.into_static())?;

            if self.compute {
                qh.compute().map_err(|e| e.into_static())?;
                if self.check_output {
                    qh.check_output().map_err(|e| e.into_static())?;
                }
                if self.check_points {
                    qh.check_points().map_err(|e| e.into_static())?;
                }
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
    /// let qh = QhBuilder::default()
    ///     .build_managed(2, vec![
    ///         0.0, 0.0,
    ///         1.0, 0.0,
    ///         0.0, 1.0,
    ///         0.25, 0.25,
    ///     ]).unwrap();
    ///
    /// assert_eq!(qh.num_facets(), 3);
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
        assert!(qh.coords_holder.is_none());
        qh.coords_holder = Some(points);
        Ok(qh)
    }

    /// Build a Qhull instance from an iterator of points
    ///
    /// # Example
    /// ```
    /// # use qhull::*;
    /// let qh = QhBuilder::default()
    ///     .build_from_iter([
    ///         [0.0, 0.0],
    ///         [1.0, 0.0],
    ///         [0.0, 1.0],
    ///         [0.25, 0.25],
    ///     ]).unwrap();
    ///
    /// assert_eq!(qh.num_facets(), 3);
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
    /// # Safety
    /// * closure must not panic
    /// * closure shall not invalidate the qhull instance
    /// * closure shall not keep references to the qhull instance
    /// * closure shall not initialize the qhull instance
    /// * closure shall not modify the error handling state of the qhull instance
    ///
    /// # Example
    /// ```
    /// # use qhull::*;
    /// let builder = unsafe {
    ///     QhBuilder::default()
    ///         .with_configure(|qh| {
    ///             Qh::try_on_qh_mut(qh, |qh| {
    ///                 (*qh).DELAUNAY = true as _;
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

    // TODO args and checkflags
}

// https://doc.rust-lang.org/book/ch03-02-data-types.html

macro_rules! add_setting {
    ($(
        $(#[$meta:meta])*
        $class:ident $($unsafe:ident)? ($($ty:tt)*) $setter:ident => $qhull_name:ident $($orig_doc:literal)?,
    )*) => {
        /// Raw settings
        ///
        /// Methods to set options that have a direct mapping to the qhull library.
        ///
        /// Some methods may invalidate the qhull instance if used incorrectly, they are marked as `unsafe`.
        impl QhBuilder {
            $(
                add_setting! {
                    $(#[$meta])*
                    $class $($unsafe)? ($($ty)*) $setter => $qhull_name $($orig_doc)?
                }
            )*
        }
    };
    (
        basic documentation: $setter:ident => $qhull_name:ident: $(#[$meta:meta])* $($orig_doc:literal)?
    ) => {
        concat!(
            "setter for [`", stringify!($qhull_name), "`](crate::sys::qhT::", stringify!($qhull_name), ")",
            "\n\n",
            $(
                concat!("Original documentation:\n> <em>", $orig_doc, "</em>\n"),
            )?
            "\n\n",
        )
    };
    (
        safety documentation: unsafe
    ) => {
        "# Safety
* This setting in unsafe because it can lead to undefined behavior if used incorrectly.

"
    };
    (
        safety documentation:
    ) => { "\n\n" };
    (
        $(#[$meta:meta])*
        scalar $($unsafe:ident)? ($ty:ident) $setter:ident => $qhull_name:ident $($orig_doc:literal)?
    ) => {
        #[doc = add_setting!(basic documentation: $setter => $qhull_name: $($orig_doc)?)]
        $(#[$meta])*
        #[doc = add_setting!(safety documentation: $($unsafe)?)]
        pub $($unsafe)? fn $setter(mut self, $setter: type_mapping::$ty) -> Self {
            self = unsafe {
                self.with_configure(move |qh| {
                    Qh::try_on_qh_mut(qh, |qh| {
                        (*qh).$qhull_name = $setter as _;
                    })
                })
            };
            self
        }
    };
    (
        $(#[$meta:meta])*
        array $($unsafe:ident)? ($ty:ident[$N:expr]) $setter:ident => $qhull_name:ident $($orig_doc:literal)?
    ) => {
        #[doc = add_setting!(basic documentation: $setter => $qhull_name: $($orig_doc)?)]
        $(#[$meta])*
        #[doc = add_setting!(safety documentation: unsafe)]
        pub unsafe fn $setter(mut self, $setter: [type_mapping::$ty; $N]) -> Self {
            self = unsafe {
                self.with_configure(move |qh| {
                    Qh::try_on_qh_mut(qh, |qh| {
                        (*qh).$qhull_name = $setter;
                    })
                })
            };
            self
        }
    };
    (
        $(#[$meta:meta])*
        dyn_array $($unsafe:ident)? ($ty:ident*) $setter:ident => $qhull_name:ident $($orig_doc:literal)?
    ) => {
        #[doc = add_setting!(basic documentation: $setter => $qhull_name: $($orig_doc)?)]
        $(#[$meta])*
        #[doc = add_setting!(safety documentation: unsafe)]
        pub unsafe fn $setter(mut self, $setter: impl IntoIterator<Item = type_mapping::$ty>) -> Self {
            let $setter = Rc::new($setter.into_iter().collect::<Vec<_>>());
            self = unsafe {
                self.with_configure(move |qh| {
                    let ptr = $setter.as_ptr();
                    qh.owned_values.$setter = Some($setter.clone());
                    Qh::try_on_qh_mut(qh, |qh| {
                        (*qh).$qhull_name = ptr as *mut _;
                    })
                })
            };
            self
        }
    };
    (
        $(#[$meta:meta])*
        point $($unsafe:ident)? ($ty:ident*) $setter:ident => $qhull_name:ident $($orig_doc:literal)?
    ) => {
        #[doc = add_setting!(basic documentation: $setter => $qhull_name: $($orig_doc)?)]
        $(#[$meta])*
        #[doc = add_setting!(safety documentation: $($unsafe)?)]
        pub $($unsafe)? fn $setter(mut self, $setter: impl IntoIterator<Item = type_mapping::$ty>) -> Self {
            let dim = self.dim.expect(concat!("dimension hint is required for ", stringify!($setter), " setter"));
            let $setter = Rc::new($setter.into_iter().collect::<Vec<_>>());
            assert_eq!($setter.len() % dim, 0, concat!("number of elements in ", stringify!($setter), " must be divisible by dim"));
            self = unsafe {
                self.with_configure(move |qh| {
                    let ptr = $setter.as_ptr();
                    qh.owned_values.$setter = Some($setter.clone());
                    Qh::try_on_qh_mut(qh, |qh| {
                        (*qh).$qhull_name = ptr as *mut _;
                    })
                })
            };
            self
        }
    };
}

add_setting!(
    scalar unsafe(boolT)  all_points => ALLpoints "true 'Qs' if search all points for initial simplex",
    scalar(boolT)  allow_short => ALLOWshort "true 'Qa' allow input with fewer or more points than coordinates",
    scalar(boolT)  allow_warning => ALLOWwarning "true 'Qw' if allow option warnings",
    scalar(boolT)  allow_wide => ALLOWwide "true 'Q12' if allow wide facets and wide dupridges, c.f. qh_WIDEmaxoutside",
    scalar(boolT)  angle_merge => ANGLEmerge "true 'Q1' if sort potential merges by type/angle instead of type/distance ",
    scalar(boolT)  approx_hull => APPROXhull "true 'Wn' if MINoutside set",
    scalar(realT)  min_outside => MINoutside "  Minimum distance for an outside point ('Wn' or 2*qh.MINvisible)",
    scalar(boolT)  annotate_output => ANNOTATEoutput "true 'Ta' if annotate output with message codes",
    scalar(boolT)  at_infinity => ATinfinity "true 'Qz' if point num_points-1 is \"at-infinity\" for improving precision in Delaunay triangulations",
    scalar(boolT)  avoid_old => AVOIDold "true 'Q4' if avoid old->new merges",
    scalar(boolT)  best_outside => BESToutside "true 'Qf' if partition points into best outsideset",
    scalar(boolT)  cdd_input => CDDinput "true 'Pc' if input uses CDD format (1.0/offset first)",
    scalar(boolT)  cdd_output => CDDoutput "true 'PC' if print normals in CDD format (offset first)",
    scalar(boolT)  check_duplicates => CHECKduplicates "true 'Q15' if qh_maybe_duplicateridges after each qh_mergefacet",
    scalar(boolT)  check_frequently => CHECKfrequently "true 'Tc' if checking frequently",
    scalar(realT)  premerge_cos => premerge_cos "'A-n'   cos_max when pre merging",
    scalar(realT)  postmerge_cos => postmerge_cos "'An'    cos_max when post merging",
    scalar(boolT)  delaunay => DELAUNAY "true 'd' or 'v' if computing DELAUNAY triangulation",
    scalar(boolT)  do_intersections => DOintersections "true 'Gh' if print hyperplane intersections",
    scalar(int)    drop_dim => DROPdim "drops dim 'GDn' for 4-d -> 3-d output",
    scalar(boolT)  flush_print => FLUSHprint "true 'Tf' if flush after qh_fprintf for segfaults",
    scalar(boolT)  force_output => FORCEoutput "true 'Po' if forcing output despite degeneracies",
    scalar(int)    good_point => GOODpoint "'QGn' or 'QG-n' (n+1, n-1), good facet if visible from point n (or not)",
    point(pointT*) good_point_coords => GOODpointp "the actual point",
    scalar(boolT)  good_threshold => GOODthreshold "true 'Pd/PD' if qh.lower_threshold/upper_threshold defined set if qh.UPPERdelaunay (qh_initbuild) false if qh.SPLITthreshold",
    scalar(int)    good_vertex => GOODvertex "'QVn' or 'QV-n' (n+1, n-1), good facet if vertex for point n (or not)",
    point(pointT*) good_vertex_coords => GOODvertexp "the actual point",
    scalar(boolT) half_space => HALFspace "true 'Hn,n,n' if halfspace intersection",
    // scalar(boolT) is_qhull_qh => ISqhullQh "Set by Qhull.cpp on initialization",
    scalar(int)  is_tracing => IStracing "'Tn' trace execution, 0=none, 1=least, 4=most, -1=events",
    scalar(int)  keep_area => KEEParea "'PAn' number of largest facets to keep",
    scalar(boolT) keep_coplanar => KEEPcoplanar "true 'Qc' if keeping nearest facet for coplanar points",
    scalar(boolT) keep_inside => KEEPinside "true 'Qi' if keeping nearest facet for inside points set automatically if 'd Qc'",
    scalar(int)   keep_merge => KEEPmerge "'PMn' number of facets to keep with most merges ",
    scalar(realT) keep_min_area => KEEPminArea "'PFn' minimum facet area to keep ",
    scalar(realT) max_coplanar => MAXcoplanar "'Un' max distance below a facet to be coplanar",
    scalar(int)   max_wide => MAXwide "'QWn' max ratio for wide facet, otherwise error unless Q12-allow-wide ",
    scalar(boolT) merge_exact => MERGEexact "true 'Qx' if exact merges (concave, degen, dupridge, flipped) tested by qh_checkzero and qh_test_*_merge",
    scalar(boolT) merge_independent => MERGEindependent "true if merging independent sets of coplanar facets. 'Q2' disables ",
    scalar(boolT) merging => MERGING "true if exact-, pre- or post-merging, with angle and centrum tests ",
    scalar(realT) premerge_centrum => premerge_centrum "  'C-n' centrum_radius when pre merging.  Default is round-off ",
    scalar(realT) postmerge_centrum => postmerge_centrum "  'Cn' centrum_radius when post merging.  Default is round-off ",
    scalar(boolT) merge_pinched => MERGEpinched "true 'Q14' if merging pinched vertices due to dupridge ",
    scalar(boolT) merge_vertices => MERGEvertices "true if merging redundant vertices, 'Q3' disables or qh.hull_dim > qh_DIMmergeVertex ",
    scalar(realT) min_visible => MINvisible "'Vn' min. distance for a facet to be visible ",
    scalar(boolT) no_narrow => NOnarrow "true 'Q10' if no special processing for narrow distributions ",
    scalar(boolT) no_near_inside => NOnearinside "true 'Q8' if ignore near-inside points when partitioning, qh_check_points may fail ",
    scalar(boolT) no_premerge => NOpremerge "true 'Q0' if no defaults for C-0 or Qx ",
    scalar(boolT) only_good => ONLYgood "true 'Qg' if process points with good visible or horizon facets ",
    scalar(boolT) only_max => ONLYmax "true 'Qm' if only process points that increase max_outside ",
    scalar(boolT) pick_furthest => PICKfurthest "true 'Q9' if process furthest of furthest points",
    scalar(boolT) post_merge => POSTmerge "true if merging after buildhull ('Cn' or 'An') ",
    scalar(boolT) pre_merge => PREmerge "true if merging during buildhull ('C-n' or 'A-n') ",
    scalar(boolT) print_centrums => PRINTcentrums "true 'Gc' if printing centrums",
    scalar(boolT) print_coplanar => PRINTcoplanar "true 'Gp' if printing coplanar points",
    scalar(int)   print_dim => PRINTdim "print dimension for Geomview output",
    scalar(boolT) print_dots => PRINTdots "true 'Ga' if printing all points as dots",
    scalar(boolT) print_good => PRINTgood "true 'Pg' if printing good facets PGood set if 'd', 'PAn', 'PFn', 'PMn', 'QGn', 'QG-n', 'QVn', or 'QV-n'",
    scalar(boolT) print_inner => PRINTinner "true 'Gi' if printing inner planes",
    scalar(boolT) print_neighbors => PRINTneighbors "true 'PG' if printing neighbors of good facets",
    scalar(boolT) print_no_planes => PRINTnoplanes "true 'Gn' if printing no planes",
    scalar(boolT) print_options_1st => PRINToptions1st "true 'FO' if printing options to stderr",
    scalar(boolT) print_outer => PRINTouter "true 'Go' if printing outer planes",
    scalar(boolT) print_precision => PRINTprecision "false 'Pp' if not reporting precision problems",
    array(qh_PRINT[sys::qh_PRINT_qh_PRINTEND as usize]) print_out => PRINTout "list of output formats to print",
    scalar(boolT) print_ridges => PRINTridges "true 'Gr' if print ridges",
    scalar(boolT) print_spheres => PRINTspheres "true 'Gv' if print vertices as spheres",
    scalar(boolT) print_statistics => PRINTstatistics "true 'Ts' if printing statistics to stderr",
    scalar(boolT) print_summary => PRINTsummary "true 's' if printing summary to stderr",
    scalar(boolT) print_transparent => PRINTtransparent "true 'Gt' if print transparent outer ridges",
    scalar(boolT) project_delaunay => PROJECTdelaunay "true if DELAUNAY, no readpoints() and need projectinput() for Delaunay in qh_init_B",
    scalar(int)   project_input => PROJECTinput "number of projected dimensions 'bn:0Bn:0'",
    scalar(boolT) random_dist => RANDOMdist "true 'Rn' if randomly change distplane and setfacetplane",
    scalar(realT) random_factor => RANDOMfactor "maximum random perturbation",
    scalar(realT) random_a => RANDOMa "qh_randomfactor is randr * RANDOMa + RANDOMb",
    scalar(realT) random_b => RANDOMb "boolT) RANDOMoutside \"true 'Qr' if select a random outside point",
    scalar(int)   report_freq => REPORTfreq "TFn' buildtracing reports every n facets",
    scalar(int)   report_freq_2 => REPORTfreq2 "tracemerging reports every REPORTfreq/2 facets",
    scalar(int)   rerun => RERUN "TRn' rerun qhull n times (qh.build_cnt)",
    scalar(int)   rotate_random => ROTATErandom "QRn' n<-1 random seed, n==-1 time is seed, n==0 random rotation by time, n>0 rotate input",
    scalar(boolT) scale_input => SCALEinput "true 'Qbk' if scaling input",
    scalar(boolT) scale_last => SCALElast "true 'Qbb' if scale last coord to max prev coord",
    scalar(boolT) set_roundoff => SETroundoff "true 'En' if qh.DISTround is predefined",
    scalar(boolT) skip_check_max => SKIPcheckmax "true 'Q5' if skip qh_check_maxout, qh_check_points may fail",
    scalar(boolT) skip_convex => SKIPconvex "true 'Q6' if skip convexity testing during pre-merge",
    scalar(boolT) split_thresholds => SPLITthresholds "true 'Pd/PD' if upper_/lower_threshold defines a region else qh.GOODthresholds set if qh.DELAUNAY (qh_initbuild) used  only for printing (!for qh.ONLYgood)",
    scalar(int)   stop_add => STOPadd "'TAn' 1+n for stop after adding n vertices",
    scalar(int)   stop_clone => STOPcone "'TCn' 1+n for stopping after cone for point n also used by qh_build_withresart for err exi",
    scalar(int)   stop_point => STOPpoint "'TVn' 'TV-n' 1+n for stopping after/before(-) adding point n",
    scalar(int)   test_point => TESTpoints "'QTn' num of test points after qh.num_points.  Test points always coplanar.",
    scalar(boolT) test_v_neighbors => TESTvneighbors " true 'Qv' if test vertex neighbors at end",
    scalar(int)   trace_level => TRACElevel "'Tn' conditional IStracing level",
    scalar(int)   trace_last_run => TRACElastrun " qh.TRACElevel applies to last qh.RERUN",
    scalar(int)   trace_point => TRACEpoint "'TPn' start tracing when point n is a vertex, use qh_IDunknown (-1) after qh_buildhull and qh_postmerge",
    scalar(realT) trace_dist => TRACEdist "'TWn' start tracing when merge distance too big",
    scalar(int)   trace_merge => TRACEmerge "'TMn' start tracing before this merge",
    scalar(boolT) triangulate => TRIangulate "true 'Qt' if triangulate non-simplicial facets",
    scalar(boolT) tri_normals => TRInormals "true 'Q11' if triangulate duplicates ->normal and ->center (sets Qt)",
    scalar(boolT) upper_delaunay => UPPERdelaunay "true 'Qu' if computing furthest-site Delaunay",
    scalar(boolT) use_stdout => USEstdout "true 'Tz' if using stdout instead of stderr",
    scalar(boolT) verify_output => VERIFYoutput "true 'Tv' if verify output at end of qhull",
    scalar(boolT) virtual_memory => VIRTUALmemory "true 'Q7' if depth-first processing in buildhull",
    scalar(boolT) voronoi => VORONOI "true 'v' if computing Voronoi diagram, also sets qh.DELAUNAY",

  /*--------input constants ---------*/
    scalar(realT) area_factor => AREAfactor "1/(hull_dim-1)! for converting det's to area",
    scalar(boolT) do_check_max => DOcheckmax "true if calling qh_check_maxout (!qh.SKIPcheckmax && qh.MERGING)",
    dyn_array(char*) feasible_string => feasible_string "feasible point 'Hn,n,n' for halfspace intersection",
    point(coordT*) feasible_point => feasible_point "   as coordinates, both malloc'd",
    scalar(boolT) get_area => GETarea "true 'Fa', 'FA', 'FS', 'PAn', 'PFn' if compute facet area/Voronoi volume in io_r.c",
    scalar(boolT) keep_near_inside => KEEPnearinside "true if near-inside points in coplanarset",
    //scalar(int)   hull_dim => hull_dim "dimension of hull, set by initbuffers",
    //scalar(int)   input_dim "dimension of input, set by initbuffers",
    scalar(int)   num_points => num_points "number of input points",
    // point(pointT*) first_point => first_point "array of input points, see POINTSmalloc",
    scalar(boolT) points_malloc => POINTSmalloc "  true if qh.first_point/num_points allocated",
    // TODO ???(pointT*) input_points => input_points "copy of original qh.first_point for input points for qh_joggleinput",
    scalar(boolT) input_malloc => input_malloc "true if qh.input_points malloc'd",
    array(char[256])  qhull_command => qhull_command "command line that invoked this program",
    scalar(int)   qhull_command_size_2 => qhull_commandsiz2 "size of qhull_command at qh_clear_outputflags",
    array(char[256]) rbox_command => rbox_command "command line that produced the input points",
    array(char[512]) qhull_options => qhull_options "descriptive list of options",
    scalar(int)  qhull_option_len => qhull_optionlen "length of last line",
    scalar(int)  qhull_option_size => qhull_optionsiz "size of qhull_options at qh_build_withrestart",
    scalar(int)  qhull_option_size_2 => qhull_optionsiz2 "size of qhull_options at qh_clear_outputflags",
    scalar(int)   run_id => run_id "non-zero, random identifier for this instance of qhull",
    scalar(boolT) vertex_neighbors => VERTEXneighbors "true if maintaining vertex neighbors",
    scalar(boolT) zero_centrum => ZEROcentrum "true if 'C-0' or 'C-0 Qx' and not post-merging or 'A-n'.  Sets ZEROall_ok",
    point(realT*) upper_threshold => upper_threshold "don't print if facet->normal\\[k\\]>=upper_threshold\\[k\\] must set either GOODthreshold or SPLITthreshold if qh.DELAUNAY, default is 0.0 for upper envelope (qh_initbuild)",
    point(realT*) lower_threshold => lower_threshold "don't print if facet->normal\\[k\\] <=lower_threshold\\[k\\]",
    point(realT*) upper_bound => upper_bound "scale point\\[k\\] to new upper bound",
    point(realT*) lower_bound => lower_bound "scale point\\[k\\] to new lower bound project if both upper_ and lower_bound == 0",

    /* precision constants */
    scalar(realT) angle_round => ANGLEround "max round off error for angles",
    scalar(realT) centrum_radius => centrum_radius "max centrum radius for convexity ('Cn' + 2*qh.DISTround)",
    scalar(realT) cos_max => cos_max "max cosine for convexity (roundoff added)",
    scalar(realT) dist_round => DISTround "max round off error for distances, qh.SETroundoff ('En') overrides qh_distround",
    scalar(realT) max_abs_coors => MAXabs_coord "max absolute coordinate",
    scalar(realT) max_last_coord => MAXlastcoord "max last coordinate for qh_scalelast",
    scalar(realT) max_outside => MAXoutside "max target for qh.max_outside/f.maxoutside, base for qh_RATIO... recomputed at qh_addpoint, unrelated to qh_MAXoutside",
    scalar(realT) max_sum_coord => MAXsumcoord "max sum of coordinates",
    scalar(realT) max_width => MAXwidth "max rectilinear width of point coordinates",
    scalar(realT) min_denom_1 => MINdenom_1 "min. abs. value for 1/x",
    scalar(realT) min_denom => MINdenom "   use divzero if denominator < MINdenom",
    scalar(realT) min_denom_1_2 => MINdenom_1_2 "min. abs. val for 1/x that allows normalization",
    scalar(realT) min_denom_2 => MINdenom_2 "   use divzero if denominator < MINdenom_2",
    scalar(realT) min_last_coord => MINlastcoord "min. last coordinate for qh_scalelast",
    point(realT*) near_zero => NEARzero "hull_dim array for near zero in gausselim",
    scalar(realT) near_inside => NEARinside "keep points for qh_check_maxout if close to facet",
    scalar(realT) one_merge => ONEmerge "max distance for merging simplicial facets",
    scalar(realT) outside_err => outside_err "application's epsilon for coplanar points qh_check_bestdist() qh_check_points() reports error if point outside",
    scalar(realT) wide_face => WIDEfacet "size of wide facet for skipping ridge in area computation and locking centrum",
    scalar(boolT) narrow_hull => NARROWhull "set in qh_initialhull if angle < qh_MAXnarrow", /**/
);

#[allow(non_camel_case_types)]
mod type_mapping {
    use crate::sys;
    pub type boolT = bool;
    pub type realT = f64;
    pub type int = i32;
    pub type pointT = f64;
    pub type coordT = f64;
    pub type char = core::ffi::c_char;
    pub type qh_PRINT = sys::qh_PRINT;
}
