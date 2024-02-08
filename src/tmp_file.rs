
use std::io;

use crate::sys;


pub struct TmpFile {
    file: *mut sys::FILE,
}

impl TmpFile {
    pub fn new() -> io::Result<TmpFile> {
        unsafe {
            let mut file: *mut sys::FILE = std::mem::zeroed();

            // on windows
            #[cfg(windows)]
            {
                let err = sys::tmpfile_s(&mut file);

                if err != 0 {
                    if !file.is_null() {
                        sys::fclose(file);
                    }
                    return Err(io::Error::last_os_error());
                } else {
                    Ok(TmpFile { file })
                }
            }
            #[cfg(not(windows))]
            {
                file = sys::tmpfile();
                if file.is_null() {
                    Err(io::Error::last_os_error())
                } else {
                    Ok(TmpFile { file })
                }
            }
        }
    }

    pub fn file_handle(&self) -> *mut sys::FILE {
        self.file
    }

    pub fn read_and_close(self) -> Result<Vec<u8>, std::io::Error> {
        // flush the file
        let _ = unsafe { sys::fflush(self.file) };
        
        // TODO fix this: written by copilot but doesn't work, but should be more efficient
        /*
        // Seek to the beginning of the file
        //let _ = unsafe { sys::fseek(self.file, 0, sys::SEEK_SET as _) };
        let _ = unsafe { sys::rewind(self.file) };

        // Get the current size of the file
        let size = unsafe { sys::ftell(self.file) };
        println!("size: {:?}", size);

        // Create a buffer with the size of the file
        let mut buffer = vec![0u8; size as usize];

        // Read the file content into the buffer
        let _ = unsafe { sys::fread(buffer.as_mut_ptr() as *mut _, 1, size as _, self.file) };
        */

        let mut buffer = Vec::new();
        unsafe {
            sys::rewind(self.file);
            while sys::feof(self.file) == 0 {
                let c = sys::fgetc(self.file);
                if c == sys::EOF {
                    break;
                }
                buffer.push(c as u8);
            }
        }

        Ok(buffer)
    }

    pub fn read_as_string_and_close(self) -> Result<String, std::io::Error> {
        let buffer = self.read_and_close()?;
        Ok(String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
    }
}

impl Drop for TmpFile {
    fn drop(&mut self) {
        unsafe {
            sys::fclose(self.file);
        }
    }
}