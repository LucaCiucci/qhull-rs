use crate::{sys, tmp_file::TmpFile};


pub struct IOBuffers {
    pub out_file: Option<TmpFile>,
    pub err_file: Option<TmpFile>,
}

impl IOBuffers {
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
        unsafe { sys::qhull_sys__stdin() }
    }

    pub fn out_file(&self) -> *mut sys::FILE {
        self.out_file.as_ref().map_or_else(
            || unsafe { sys::qhull_sys__stdout() },
            |f| f.file_handle() as *mut _,
        )
    }

    pub fn err_file(&self) -> *mut sys::FILE {
        self.err_file.as_ref().map_or_else(
            || unsafe { sys::qhull_sys__stderr() },
            |f| f.file_handle() as *mut _,
        )
    }
}
