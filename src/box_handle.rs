use std::ops::Deref;

use windows::Win32::Foundation::{HANDLE, CloseHandle};

pub struct BoxHandle {
    handle: HANDLE
}

impl From<HANDLE> for BoxHandle {
    fn from(handle: HANDLE) -> Self {
        Self { handle }
    }
}

impl Deref for BoxHandle {
    type Target = HANDLE;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Drop for BoxHandle {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            unsafe {
                CloseHandle(self.handle);
            }
        }
    }
}