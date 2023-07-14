use crate::hack_main::hack_main;
use crate::stop_token::StopToken;
use std::panic::catch_unwind;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use self::linux as native;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use self::windows as native;

/// Library init hook.
/// Safe to call from C, as it catches rust panics.
/// Therefore, all possibly panicking native init code is injected in here.
extern "C" fn initialize_hook<F>(native_init: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    let init = || {
        native_init();

        // run in detached thread
        // the thread is going to be automatically destroyed when the library exits
        std::thread::spawn(run_main_and_free_after_finish);
    };

    let result = catch_unwind(init);
    if let Err(error) = result {
        let message = format!("Hack init panicked: {error:?}");
        gui::message_box::warn(message);
    }
}

/// Library deinit hook.
/// Safe to call from C, as it catches rust panics.
/// Therefore, all possibly panicking native deinit code is injected in here.
extern "C" fn deinitialize_hook<F>(native_deinit: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    let deinit = || {
        native_deinit();

        gui::message_box::info("deinitialize");
    };

    let result = catch_unwind(deinit);
    if let Err(error) = result {
        let message = format!("Hack deinit panicked: {error:?}");
        gui::message_box::warn(message);
    }
}

/// runs hack, frees library after finish
fn run_main_and_free_after_finish() {
    let stop_token = StopToken::default();
    hack_main(stop_token);
    free_library()
    // TODO: Employ states.
    //       Uninitialized,     | not started yet
    //       Running,           | initialize() started it
    //       StoppedFromOutside | either someone kicked our library or kicked the whole process
    //
    //       Either the state is still running
}

/// TODO
fn free_library() {
    native::free_library()
}

// end scenarios:
//- From inside (panic)
//  - set stop flag from hack thread
//  - hack main() ends
//  - hack thread determines process: inside
//  - join hack thread
//  - drop hack thread + drop Hack
//  - Win: FreeLibAndExitThread Linux: ComplictedExitProcedure
//  - deinitializer runs (dry)
//- From outside (Process termination or other FreeLibrary)
//  - deinitializer runs
//  - set stop flag from deinitializer
//  - join hack thread
//  - drop hack thread + drop Hack
//  - deinitializer finishes
//- Both A: From outside, then from inside
//  - from outside sets stop flag & from outside process
//  - Race condition#0: Setting stopflag & eject process should be atomic to keep precedence variant.
//  - Race condition#1: From inside during from-outside procedure:
//      - No problem, since from inside only happens bc hack main isn't finished yet - from-outside hasn't started yet
//  - from inside sees from outside process is set, does nothing
//  - hack main() ends
//  - from outside process takes over.
//  TEST Windows:
//      - block deinitialize on barrier
//  TEST Linux:
//- Both B: From inside, then from outside
//  - Race Condition: Outside call can come ANYTIME. Even during eject thread creation. Mutex?
//---TODO
//  - from inside process
//  - no problem?
//----TODO
//- Why the processes?
//  - Synchronization:
//    To avoid that a free library procedure is called when the library destructor is only waiting for hack main finish.
//  - Consent:
//    To come to clonclusion which way to exit
//
//    as the
// EXPERIMENTS FINDINGS
// - On Process terminate, Windows just kills our hack thread without destructors being run
//   no matter if detached or not
//   --> Join on hack thread panics
//   --> Check if we can catch signals
//   --> Check if this isn't too much of an edge case.
//   --> Also means we can't do cleanup properly.
// - On FreeLibraryAndExitThread, the deinitializer is executed in the calling thread.
// - Calling FreeLibraryAndExitThread twice - from hack and then in deinitializer - leeds to no freeing.
