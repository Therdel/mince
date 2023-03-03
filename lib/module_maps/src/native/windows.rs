use std::mem::size_of_val;
use anyhow::{anyhow, Result};
use region::Region;
use windows::Win32::{
    System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot,
                                    Module32First,
                                    Module32Next,
                                    MODULEENTRY32,
                                    TH32CS_SNAPMODULE,
                                    TH32CS_SNAPMODULE32,},
    Foundation::{HANDLE, CloseHandle},
};

pub struct ModuleMap {
    handle: HANDLE
}

impl Drop for ModuleMap {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            unsafe {
                CloseHandle(self.handle);
            }
        }
    }
}

impl ModuleMap {
    pub fn snapshot() -> Result<ModuleMap> {
        let flags = TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32;
        let current_process = 0;
        let handle = unsafe {
            CreateToolhelp32Snapshot(flags, current_process)
        };
    
        if handle.is_invalid() {
            Err(anyhow!("ModuleMap enumeration failed: {}", windows::core::Error::from_win32().to_string()))
        } else {
            let snapshot = ModuleMap { handle };
            Ok(snapshot)
        }
    }
}

pub struct Module {
    memory: *const [u8],
    file_name: String,
}

impl Module {
    fn memory(&self) -> *const [u8] { self.memory }
    pub fn base(&self) -> *const u8 { unsafe { &*self.memory }.as_ptr() }
    pub fn file_name(&self) -> &str { &self.file_name }
    pub fn regions_snapshot(&self) -> Regions {
        Regions { module_memory: self.memory(), next_region_base: Some(self.base()) }
    }
}

impl From<MODULEENTRY32> for Module {
    fn from(module: MODULEENTRY32) -> Self {
        let memory = unsafe {
            std::slice::from_raw_parts(module.modBaseAddr, module.modBaseSize as usize)
        };
        let file_name = module.szModule.iter()
            .map_while(|c| -> Option<char> {
                match c.ok() {
                    Ok(c) => char::from_u32(c.0.into()),
                    Err(_) => None
                }
            })
            .collect();
        Module {
            memory: memory as _,
            file_name,
        }
    }
}

pub struct Modules {
    snapshot: ModuleMap,
    next: Option<Module>,
}

impl Iterator for Modules {
    type Item = Module;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.next.take() {
            let mut native_next = MODULEENTRY32::default();
            native_next.dwSize = size_of_val(&native_next) as _;
            let next_exists = unsafe {
                Module32Next(self.snapshot.handle, &mut native_next)
            };
            self.next = match next_exists.as_bool() {
                true => Some(native_next.into()),
                false => None
            };

            Some(current)
        } else {
            None
        }
    }
}

impl IntoIterator for ModuleMap {
    type Item = Module;

    type IntoIter = Modules;

    fn into_iter(self) -> Self::IntoIter {
        let mut first_native = MODULEENTRY32::default();
        first_native.dwSize = size_of_val(&first_native) as _;
        let first_exists = unsafe {
            Module32First(self.handle, &mut first_native)
        };

        let first = match first_exists.as_bool() {
            true => Some(first_native.into()),
            false => None
        };
    
        Self::IntoIter {
            snapshot: self,
            next: first
        }
    }
}

pub struct Regions {
    module_memory: *const [u8],
    next_region_base: Option<*const u8>
}

impl Iterator for Regions {
    type Item = Region;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(current_region_base) = self.next_region_base else {
            return None
        };

        // shouldn't fail - but if it does, we can't do better
        // the underlying VirtualQuery mechanism is just snapshotting: https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualquery
        // so if it fails, a module has been unloaded at runtime - which is entirely possible and we return no info about the unloaded range
        let Ok(region) = region::query(current_region_base) else {
            self.next_region_base = None;
            return None
        };

        let next_region_base: *const u8 = region.as_ptr_range().end;
        let module_end_address = unsafe { &*self.module_memory }.as_ptr_range().end;
        self.next_region_base = (next_region_base < module_end_address).then_some(next_region_base);

        Some(region)
    }
}