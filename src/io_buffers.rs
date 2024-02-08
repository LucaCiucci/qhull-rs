use crate::{sys, tmp_file::TmpFile};


pub struct IOBuffers {
    pub out_file: Option<TmpFile>,
    pub err_file: Option<TmpFile>,
}

impl IOBuffers {
    // from C:
    //#define stdin  (__acrt_iob_func(0))
    //#define stdout (__acrt_iob_func(1))
    //#define stderr (__acrt_iob_func(2))
    const STD_IN_IDX: usize = 0;
    const STD_OUT_IDX: usize = 1;
    const STD_ERR_IDX: usize = 2;

    pub fn new(
        capture_stdout: bool,
        capture_stderr: bool,
    ) -> Self {
        Self {
            out_file: capture_stdout.then(|| TmpFile::new().expect("failed to create temporary file for stdout")),
            err_file: capture_stderr.then(|| TmpFile::new().expect("failed to create temporary file for stderr")),
        }
    }

    pub fn in_file(&self) -> *mut sys::FILE {
        Self::default_file(Self::STD_IN_IDX)
    }

    pub fn out_file(&self) -> *mut sys::FILE {
        self.out_file.as_ref().map_or_else(
            || Self::default_file(Self::STD_OUT_IDX),
            |f| f.file_handle() as *mut _,
        )
    }

    pub fn err_file(&self) -> *mut sys::FILE {
        self.err_file.as_ref().map_or_else(
            || Self::default_file(Self::STD_ERR_IDX),
            |f| f.file_handle() as *mut _,
        )
    }

    fn default_file(idx: usize) -> *mut sys::FILE {
        unsafe { sys::__acrt_iob_func(idx as _) }
    }
}
