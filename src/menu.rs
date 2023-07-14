use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{JoinHandle, self};

use crate::{gui, StopToken};

#[derive(PartialEq, Clone, Copy)]
pub enum Command {
    Bhop = 0,
    NoBhop = 1,
}

#[derive(Default, Clone)]
struct SharedCommand {
    command: Arc<AtomicBool>,
}

impl SharedCommand {
    fn get(&self) -> Command {
        match self.command.load(Ordering::SeqCst) {
            true => Command::Bhop,
            false => Command::NoBhop,
        }
    }

    fn set(&self, command: Command) {
        self.command.store(command as isize == 0, Ordering::SeqCst)
    }
}

pub struct Menu {
    command: SharedCommand,
    _thread: JoinHandle<()>,
}

impl Menu {
    pub fn new(stop_token: StopToken) -> Self {
        let command = SharedCommand::default();
        let command_clone = command.clone();
        let thread = thread::spawn(|| Self::gui_thread(stop_token, command_clone));

        Self { command, _thread: thread }
    }

    pub fn poll_command(&self) -> Command {
        self.command.get()
    }

    fn gui_thread(stop_token: StopToken, command: SharedCommand) {
        while !stop_token.stop_requested() {
            let answer = gui::message_box::yes_no("yes: toggle bhop\nno: leave");
            match answer {
                true => match command.get() {
                    Command::Bhop => command.set(Command::NoBhop),
                    Command::NoBhop => command.set(Command::Bhop),
                },
                false => stop_token.request_stop(),
            }
        }
    }
}