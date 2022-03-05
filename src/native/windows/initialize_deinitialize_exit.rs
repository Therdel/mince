use std::os::raw::{c_ulong, c_void, c_int};
use windows::{
    Win32::Foundation::HINSTANCE,
    Win32::System::LibraryLoader::{DisableThreadLibraryCalls, FreeLibraryAndExitThread},
    Win32::{System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH}},
};
use once_cell::sync::OnceCell;
use crate::gui::message_box;

static H_MODULE: OnceCell<HMODULE> = OnceCell::new();

type HMODULE = HINSTANCE;
type DWORD = c_ulong;
type LPVOID = *mut c_void;
type BOOL = c_int;

#[no_mangle]
extern "stdcall" fn DllMain(h_module: HMODULE,
                            ul_reason_for_call: DWORD,
                            lp_reserved: LPVOID) -> BOOL {
    match ul_reason_for_call {
        DLL_PROCESS_ATTACH => {
            // SAFETY: Cannot fail, as process attach happens only once
            H_MODULE.set(h_module).unwrap();
            unsafe { DisableThreadLibraryCalls(h_module); }
            crate::initialize();
        },
        DLL_PROCESS_DETACH => {
            if lp_reserved.is_null() {
                // reason: FreeLibrary was called
                message_box::info("Detach: FreeLibrary");
            } else {
                // reason: Process is terminating
                message_box::info("Detach: Process terminating");
            }
            crate::deinitialize();
        }
        _ => {}
    }

    true as BOOL
}

pub fn exit() -> ! {
    // SAFETY: Always initialized, as exit can only be called after or within initialize,
    //         which sets H_MODULE first thing.
    let h_module = H_MODULE.get().unwrap();
    unsafe {
        FreeLibraryAndExitThread(h_module, 0)
    }
}