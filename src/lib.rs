mod box_handle;
mod module_name;
use module_name::*;
mod features;
// mod hook;

use std::{os::raw::{c_ulong, c_void, c_int}, time::Duration};
use std::sync::{Arc, atomic::AtomicIsize, atomic::Ordering};
use windows::{
    Win32::Foundation::HINSTANCE,
    Win32::System::LibraryLoader::{DisableThreadLibraryCalls, FreeLibraryAndExitThread},
    Win32::{System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH}},
    Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_OK, MB_YESNOCANCEL, MB_ICONQUESTION, IDYES, IDNO, MESSAGEBOX_RESULT},
};

type HMODULE = HINSTANCE;
type DWORD = c_ulong;
type LPVOID = *mut c_void;
type BOOL = c_int;

fn message(message: &str) -> MESSAGEBOX_RESULT{
    unsafe {
        MessageBoxA(None,
                    message,
                    None,
                    MB_OK)
    }
}

fn hopper_thread(exit: Arc<AtomicIsize>) {
    let loop_duration = Duration::from_millis(100);
    let bhop = features::Bunnyhop::new();
    while exit.load(Ordering::SeqCst) >=  0 {
        if exit.load(Ordering::SeqCst).is_positive() {
            bhop.auto_jump(loop_duration);
        } else {
            std::thread::sleep(loop_duration);
        }
    }
}

fn thread(h_module: HMODULE) {
    let exit = std::sync::Arc::from(std::sync::atomic::AtomicIsize::from(0));
    let exit_clone = exit.clone();
    let hopper = std::thread::spawn(||hopper_thread(exit_clone));
    loop {
        let answer = unsafe {
            MessageBoxA(None,
                        "Jump? Stand? Leave?",
                        None,
                        MB_YESNOCANCEL | MB_ICONQUESTION)
            };
        let exit_next = match answer {
            IDYES => 1,
            IDNO => 0,
            _ => -1
        };
        exit.store(exit_next, Ordering::SeqCst);
        if exit.load(Ordering::SeqCst).is_negative() {
            break;
        }
    }
    hopper.join().unwrap();

    unsafe {
        FreeLibraryAndExitThread(h_module, 0)
    }
}

#[no_mangle]
extern "stdcall" fn DllMain(h_module: HMODULE,
                            ul_reason_for_call: DWORD,
                            lp_reserved: LPVOID) -> BOOL {
    match ul_reason_for_call {
        DLL_PROCESS_ATTACH => {
            unsafe { DisableThreadLibraryCalls(h_module); }
            message("Hello you bastard!");

            std::thread::spawn(move ||thread(h_module));
        },
        DLL_PROCESS_DETACH => {
            if lp_reserved.is_null() {
                // reason: FreeLibrary was called
                message("Detach: FreeLibrary");
            } else {
                // reason: Process is terminating
                message("Detach: Process terminating");
            }
        }
        _ => {}
    }

    true as BOOL
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
