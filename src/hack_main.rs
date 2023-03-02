use std::{ thread, time::Duration };

use anyhow::Result;

use crate::features::Bunnyhop;
use crate::gui;
use crate::StopToken;
use crate::memory::{MemVars, detour::DetourOrder, hooks::OnGroundHook};
use crate::menu::*;

fn run(stop_token: StopToken) -> Result<()> {
    let menu = Menu::new(stop_token.clone());
    let mem_vars = MemVars::read()?;
    let mut bhop = Bunnyhop::new(&mem_vars);
    // TODO: Fix unsafe passing to another thread / hook thread
    // TODO: Fix ownership problem
    //          - OnGroundHook should hold the instance alive rc/arc, or hold a reference
    //          - If using rc/arc: Solve deref problem, Solve mutex/refcell problem
    let _on_ground_hook = OnGroundHook::install(&mem_vars, &bhop, DetourOrder::DetourAfter);
    
    let poll_sleep = Duration::from_millis(100);
    while !stop_token.stop_requested() {
        match menu.poll_command() {
            Command::Bhop => bhop.set_active(true),
            Command::NoBhop => bhop.set_active(false)
        }
        thread::sleep(poll_sleep);
    }
    
    Ok(())
}

pub fn hack_main(stop_token: StopToken) {
    if let Err(err) = run(stop_token) {
        gui::message_box::warn(&err.to_string());
    }
}