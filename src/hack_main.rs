use std::{ thread, time::Duration };

use anyhow::Result;

use crate::features::{aimbot::{Aimbot, AimbotConfig, AimPoint, TargetSelect}, bunnyhop::Bunnyhop};
use crate::gui;
use crate::StopToken;
use crate::memory::{MemVars, detour::DetourOrder};
use crate::hooks::{OnGroundHook, ViewAnglesReadHook};
use crate::menu::*;

fn run(stop_token: StopToken) -> Result<()> {
    let mem_vars = MemVars::get()?;
    let aimbot_config = AimbotConfig {
        friendly_fire: false,
        target_select: TargetSelect::ByAngle { angle_deg: 12.0 },
        aim_point: AimPoint::HeadStanding,
    };
    
    let menu = Menu::new(stop_token.clone());

    // TODO: Fix unsafe passing to another thread / hook thread
    let mut bhop = Bunnyhop::new(&mem_vars);
    let aimbot = Aimbot::new(mem_vars, aimbot_config);
    
    // TODO: Fix ownership problem
    //          - OnGroundHook should hold the instance alive rc/arc, or hold a reference
    //          - If using rc/arc: Solve deref problem, Solve mutex/refcell problem
    let _on_ground_hook = OnGroundHook::install(&mem_vars, &bhop, DetourOrder::DetourAfter)?;
    let _view_angles_read_hook = ViewAnglesReadHook::install(&mem_vars, &aimbot, DetourOrder::DetourBefore)?;
    
    let poll_sleep = Duration::from_millis(100);
    while !stop_token.stop_requested() {
        let do_bhop = menu.poll_command() == Command::Bhop;
        bhop.set_active(do_bhop);
        thread::sleep(poll_sleep);
    }

    Ok(())
}

pub fn hack_main(stop_token: StopToken) {
    if let Err(err) = run(stop_token) {
        gui::message_box::warn(err.to_string());
    }
}