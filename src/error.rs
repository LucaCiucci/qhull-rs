use std::{error::Error, fmt::Display};

use crate::{sys, tmp_file::TmpFile};

macro_rules! define_error_kinds {
    (
        $(
            $(#[$attr:meta])*
            $name:ident => $code:literal,
        ),*$(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum QhErrorKind {
            $(
                $(#[$attr])*
                ///
                #[doc = concat!("Error code ", $code)]
                $name,
            )*

            /// An error code that is not part of the enum.
            Other(i32),
        }

        impl QhErrorKind {
            pub fn from_code(code: i32) -> Self {
                match code {
                    0 => panic!("0 is not an error code"),
                    $(
                        $code => Self::$name,
                    )*
                    _ => Self::Other(code),
                }
            }
            pub fn error_code(&self) -> i32 {
                match self {
                    $(
                        Self::$name => $code,
                    )*
                    Self::Other(code) => *code,
                }
            }
        }
    };
}

define_error_kinds!{
    // TODO ...
}

#[derive(Debug, Clone)]
pub struct QhError {
    pub kind: QhErrorKind,
    pub error_message: Option<String>,
}

impl Display for QhError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Qhull error: {:?} (#{})", self.kind, self.kind.error_code())?;
        if let Some(msg) = &self.error_message {
            write!(f, "\n{}", msg)?;
        }
        Ok(())
    }
}

impl Error for QhError {}

impl QhError {
    pub unsafe fn try_on_raw<R, F>(
        qh: &mut sys::qhT,
        err_file: &mut Option<TmpFile>,
        f: F,
    ) -> Result<R, QhError>
    where
        F: FnOnce(&mut sys::qhT) -> R,
    {
        unsafe extern "C" fn cb<F2>(
            qh: *mut sys::qhT,
            data: *mut std::ffi::c_void,
        )
        where
            F2: FnOnce(&mut sys::qhT),
        {
            assert!(qh.is_null() == false, "qh is null");
            assert!(data.is_null() == false, "data is null");
            let qh = &mut *qh;
            let f: &mut Option<F2> = &mut *(data as *mut _);
            f.take().unwrap()(qh);
        }
    
        fn get_cb<F>(_: &mut Option<F>) -> unsafe extern "C" fn(*mut sys::qhT, *mut std::ffi::c_void)
        where
            F: FnOnce(&mut sys::qhT),
        {
            cb::<F>
        }

        let mut result = None;

        let mut f = Some(|qh: &mut sys::qhT| result = Some(f(qh)));
    
        let err_code = unsafe { sys::qhull_sys__try_on_qh(
            &mut *qh,
            Some(get_cb(&mut f)),
            &mut f as *mut _ as *mut std::ffi::c_void,
        )};

        if err_code == 0 {
            Ok(result.unwrap())
        } else {
            let kind = QhErrorKind::from_code(err_code);
            let file = err_file.replace(TmpFile::new().expect("Failed to create a replacement temporary file"));
            qh.ferr = err_file.as_ref().unwrap().file_handle();
            let msg = file.map(|file| file.read_as_string_and_close().unwrap());
            Err(QhError {
                kind,
                error_message: msg,
            })
        }
    }
}