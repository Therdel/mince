use std::fmt::Result;

use region::Region;

pub struct Hook {
    patched_region: Region,
    original_code: Vec<u8>
}

impl Hook {
    pub unsafe fn hook(code_to_patch: Region, jump_destination: isize) -> region::Result<Hook> {
        // this error handling sucks.
        if code_to_patch.len() < Self::min_patch_size() {
            Err(todo!())?
        }
        let hook = Self {
            patched_region: code_to_patch,
            original_code: todo!()
        };
        Ok(hook)
    }

    fn copy_original_code(region_to_patch: Region) -> region::Result<Hook> {
        todo!()
    }

    // minimum amount of bytes required for a patch
    const fn min_patch_size() -> usize {
        #[cfg(target_pointer_width = "32")]
        return 5;
        #[cfg(target_pointer_width = "64")]
        return 12;
    }

    pub fn unhook(self) {}
}

impl Drop for Hook {
    fn drop(&mut self) {
        todo!("Unhook")
    }
}

enum ExecuteOriginalCode {
    BeforeCallback,
    AfterCallback,
    Dont
}

// fn insert_callback()

// struct Dispatcher {

// }