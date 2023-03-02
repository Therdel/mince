use std::{mem::size_of_val, ops::Deref};
use windows::Win32::{
    System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot,
                                    Module32First,
                                    Module32Next,
                                    MODULEENTRY32,
                                    TH32CS_SNAPMODULE,
                                    TH32CS_SNAPMODULE32,},
};
use crate::error::Result;
mod box_handle;
use box_handle::BoxHandle;
use crate::ModuleMapping;

#[cfg(feature = "expose_native_module_types")]
pub type NativeModuleMapping = MODULEENTRY32;

struct Snapshot {
    handle: BoxHandle
}

fn snapshot() -> Result<Snapshot> {
    let flags = TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32;
    let current_process = 0;
    let handle = unsafe {
        CreateToolhelp32Snapshot(flags, current_process)
    };

    if handle.is_invalid() {
        Err(windows::core::Error::from_win32().into())
    } else {
        let handle = BoxHandle::from(handle);
        let snapshot = Snapshot { handle };
        Ok(snapshot)
    }
}

pub struct IntoIter {
    snapshot: Snapshot,
    next: Option<ModuleMapping>,
}

impl Iterator for IntoIter {
    type Item = ModuleMapping;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.next.take() {
            let mut native_next = MODULEENTRY32::default();
            native_next.dwSize = size_of_val(&native_next) as _;
            let next_exists = unsafe {
                Module32Next(*self.snapshot.handle, &mut native_next)
            };
            self.next = match next_exists.as_bool() {
                true => Some(native_module_to_mapping(native_next)),
                false => None
            };

            Some(current)
        } else {
            None
        }
    }
}

impl IntoIterator for Snapshot {
    type Item = ModuleMapping;

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let mut first_native = MODULEENTRY32::default();
        first_native.dwSize = size_of_val(&first_native) as _;
        let first_exists = unsafe {
            Module32First(self.handle.deref(), &mut first_native)
        };

        let first = match first_exists.as_bool() {
            true => Some(native_module_to_mapping(first_native)),
            false => None
        };
    
        Self::IntoIter {
            snapshot: self,
            next: first
        }
    }
}

fn get_module_file_name(module: &MODULEENTRY32) -> String {
    let chars = module.szModule.iter()
        .map_while(|c| -> Option<char> {
            match c.ok() {
                Ok(c) => char::from_u32(c.0.into()),
                Err(_) => None
            }
        });
    String::from_iter(chars)
}

fn native_module_to_mapping(module: MODULEENTRY32) -> ModuleMapping {
    let memory = unsafe { std::slice::from_raw_parts(module.modBaseAddr, module.modBaseSize as usize) };
    ModuleMapping {
        memory: memory as _,
        file_name: get_module_file_name(&module),
        
        #[cfg(feature = "expose_native_module_types")]
        native_module: module
    }
}

pub fn iterate_mappings() -> Result<IntoIter> {
    let iterator = snapshot()?
        .into_iter();
    Ok(iterator)
}