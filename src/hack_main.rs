use std::{ thread, time::Duration };

use anyhow::{anyhow, Result};

use crate::features::Bunnyhop;
use crate::gui;
use crate::StopToken;
use crate::memory::{MemVars, detour::DetourOrder};
use crate::hooks::OnGroundHook;
use crate::menu::*;

fn fire_detection_thread(stop_token: StopToken) {
    let closure = || -> Result<()> {
        let mem_vars = MemVars::get()?;

        let mut last_target_id = 0;
        while !stop_token.stop_requested() {
            if let Some(localplayer) = mem_vars.localplayer() {
                if last_target_id != localplayer.target_id {
                    last_target_id = localplayer.target_id;
                    if last_target_id != 0 {
                        *mem_vars.do_attack_1() = 5;
                    } else {
                        *mem_vars.do_attack_1() = 4;
                    }
                }
            }
            
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    };
    if let Err(err) = closure() {
        gui::message_box::warn(&format!("FireDetectionThread failed:\n{err}"));
    }
}

fn run(stop_token: StopToken) -> Result<()> {
    let menu = Menu::new(stop_token.clone());
    let mem_vars = MemVars::get()?;
    let mut bhop = Bunnyhop::new(&mem_vars);
    // TODO: Fix unsafe passing to another thread / hook thread
    // TODO: Fix ownership problem
    //          - OnGroundHook should hold the instance alive rc/arc, or hold a reference
    //          - If using rc/arc: Solve deref problem, Solve mutex/refcell problem
    let _on_ground_hook = OnGroundHook::install(&mem_vars, &bhop, DetourOrder::DetourAfter);

    let stop_token_clone = stop_token.clone();
    let fire_detection_thread = std::thread::spawn(|| fire_detection_thread(stop_token_clone));
    
    let poll_sleep = Duration::from_millis(100);
    while !stop_token.stop_requested() {
        match menu.poll_command() {
            Command::Bhop => bhop.set_active(true),
            Command::NoBhop => bhop.set_active(false)
        }
        thread::sleep(poll_sleep);
    }

    fire_detection_thread.join()
        .map_err(|_| {
            anyhow!("Joining thread failed")
        })
}

pub fn hack_main(stop_token: StopToken) {
    if let Err(err) = run(stop_token) {
        gui::message_box::warn(&err.to_string());
    }
}