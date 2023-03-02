use std::ffi::{CStr, OsStr};
use std::os::unix::prelude::OsStrExt;
use std::path::Path;
use libc::{dl_iterate_phdr, dl_phdr_info, size_t, c_void};

use crate::ModuleMapping;
use crate::error::Result;

#[cfg(feature = "expose_native_module_types")]
pub type NativeModuleMapping = dl_phdr_info;

struct Snapshot {
    mappings: Vec<ModuleMapping>
}

fn snapshot() -> Result<Snapshot> {
    let mut mappings = Vec::new();
    let mappings_ptr: *mut c_void = &mut mappings as *mut _ as _;
    extern "C" fn callback(module_ptr: *mut dl_phdr_info,
                           _: size_t,
                           mappings_ptr: *mut c_void) -> i32 {
        let mappings_ref: &mut Vec<ModuleMapping> = unsafe {
            let typed_ptr = mappings_ptr as *mut Vec<ModuleMapping>;
            &mut *typed_ptr
        };
        let module = unsafe {
            &*module_ptr
        };
        let mapping = native_module_to_mapping(module);
        mappings_ref.push(mapping);
        return 0
    }
    unsafe {
        dl_iterate_phdr(Some(callback), mappings_ptr);
    }
    let snapshot = Snapshot {
        mappings
    };
    Ok(snapshot)
}

fn get_module_file_name(module: &dl_phdr_info) -> String {
    // SAFETY: These come from the OS and are therefore c-strings
    let chars = unsafe {
        CStr::from_ptr(module.dlpi_name).to_bytes()
    };
    let os_str = OsStr::from_bytes(chars);
    let path = Path::new(os_str);
    
    let path_empty = path.components().count() == 0;
    if path_empty {
        String::new()
    } else {
        // unwrap: These paths don't end in '...', so unwrapping is safe
        let file_name = path.file_name().unwrap();
        let file_name = file_name.to_string_lossy();
        file_name.into()
    }
}

fn native_module_to_mapping(module: &dl_phdr_info) -> ModuleMapping {
    let base_addr = (*module).dlpi_addr;
    let base = base_addr as *const u8;
    let size = todo!();
    let memory = unsafe { std::slice::from_raw_parts(base, size) };
    
    let mapping = ModuleMapping {
        memory: memory as _,
        file_name: get_module_file_name(module),
        #[cfg(feature = "expose_native_module_types")]
        native_module: module.clone()
    };
    mapping
}

type IntoIter = std::vec::IntoIter<ModuleMapping>;

pub fn iterate_mappings() -> Result<IntoIter> {
    let iterator = snapshot()?
        .mappings
        .into_iter();
    Ok(iterator)
}