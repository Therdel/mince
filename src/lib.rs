mod gui;
mod module_name;
use module_name::*;
mod features;
mod native;
// mod hook;

use std::{time::Duration,
          sync::{Arc,
                 atomic::{AtomicIsize, Ordering}
                }
          };

fn initialize() {
    gui::message_box::warn("Hello!");
    std::thread::spawn(main);
}

fn deinitialize() {
    gui::message_box::warn("Bye!");
}

fn exit_by_user() {
    gui::message_box::info("exit by user");
    native::initialize_deinitialize_exit::exit()
}

fn main() {
    #[derive(PartialEq, Clone, Copy)]
    enum Command {
        Bhop = 0,
        NoBhop = 1,
        Exit = -1
    }
    impl From<isize> for Command {
        fn from(command: isize) -> Self {
            match command {
                0 => Command::Bhop,
                1 => Command::NoBhop,
                _ => Command::Exit
            }
        }
    }
    // impl From<Command> for isize {
    //     fn from(command: Command) -> Self {
    //         command as _
    //     }
    // }
    let command = std::sync::Arc::from(std::sync::atomic::AtomicIsize::from(0));
    let command_clone = command.clone();
    let hopper = std::thread::spawn(||bhop_thread(command_clone));
    loop {
        let answer = gui::message_box::yes_no("yes: toggle bhop\nno: leave");
        let last_command: Command = command.load(Ordering::SeqCst).into();
        let next_command = match answer {
            true => {
                match last_command {
                    Command::Bhop => Command::NoBhop,
                    Command::NoBhop => Command::Bhop,
                    Command::Exit => unreachable!()
                }
            }
            false => {
                Command::Exit
            }
        };
        command.store(next_command as _, Ordering::SeqCst);
        if next_command == Command::Exit {
            break;
        }
    }
    hopper.join().unwrap();

    exit_by_user()
}

fn bhop_thread(exit: Arc<AtomicIsize>) {
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}