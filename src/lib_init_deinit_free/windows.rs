use std::os::raw::{c_ulong, c_void, c_int};
use windows::{
    Win32::Foundation::HINSTANCE,
    Win32::System::LibraryLoader::{DisableThreadLibraryCalls, FreeLibraryAndExitThread},
    Win32::{System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH}},
};
use once_cell::sync::OnceCell;

static H_MODULE: OnceCell<HMODULE> = OnceCell::new();

type HMODULE = HINSTANCE;
type DWORD = c_ulong;
type LPVOID = *mut c_void;
type BOOL = c_int;

#[no_mangle]
extern "stdcall" fn DllMain(h_module: HMODULE,
                            ul_reason_for_call: DWORD,
                            _lp_reserved: LPVOID) -> BOOL {
    match ul_reason_for_call {
        DLL_PROCESS_ATTACH => {
            let native_init = || {
                // SAFETY: Cannot fail, as process attach happens only once
                H_MODULE.set(h_module).unwrap();
                unsafe { DisableThreadLibraryCalls(h_module); }
            };
            super::initialize_hook(native_init);
        }
        DLL_PROCESS_DETACH => {
            // if lp_reserved.is_null() {
            //     // reason: FreeLibrary was called
            // } else {
            //     // reason: Process is terminating
            // }
            let native_deinit = || {};
            super::deinitialize_hook(native_deinit);
        }
        _ => {}
    }

    true as BOOL
}

pub fn free_library() -> ! {
    // SAFETY: Always initialized, as exit can only be called after or within initialize,
    //         which sets H_MODULE first thing.
    let h_module = H_MODULE.get().unwrap();
    unsafe {
        FreeLibraryAndExitThread(h_module, 0)
    }
}