use crate::features::Bunnyhop;
use crate::gui;
use crate::StopToken;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

#[derive(PartialEq, Clone, Copy)]
enum Command {
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

fn menu(stop_token: StopToken, command: SharedCommand) {
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

pub fn hack_main(stop_token: StopToken) {
    let command = SharedCommand::default();
    let (stop_clone, command_clone) = (stop_token.clone(), command.clone());
    let menu_thread = thread::spawn(|| menu(stop_clone, command_clone));

    let loop_duration = Duration::from_millis(100);
    let bhop = Bunnyhop::new();
    while !stop_token.stop_requested() {
        match command.get() {
            Command::Bhop => bhop.auto_jump(loop_duration),
            Command::NoBhop => thread::sleep(loop_duration),
        }
    }

    menu_thread.join().unwrap();
}
