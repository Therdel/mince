use std::{ffi::{CStr, c_char}, fmt::Display};

use anyhow::Result;
use nalgebra_glm as glm;

use super::offsets;

#[repr(u32)]
#[derive(PartialEq, Debug)]
pub enum Team {
    Spectator = 1,
    Terrorist = 2,
    CounterTerrorist = 3,
}

#[repr(C)]
pub struct Localplayer {
    ____________________________padding0: [u8; offsets::localplayer::TEAM],
    pub team: Team,
    ____________________________padding1: [u8; offsets::localplayer::PADDING_TEAM_EYE_HEIGHT],
    pub eye_height: f32,
    ____________________________padding2: [u8; offsets::localplayer::PADDING_EYE_HEIGHT_POS],
    pub pos: glm::Vec3,
    ____________________________padding3: [u8; offsets::localplayer::PADDING_POS_PUNCH],
    pub punch_angles: glm::Vec3,
    ____________________________padding4: [u8; offsets::localplayer::PADDING_PUNCH_TARGET],
    pub target_id: u32,
}

impl Display for Localplayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Localplayer {{\n\tteam: {:?},\n\teye_height: {},\n\tpos: {},\n\tpunch_angles: {},\n\ttarget_id: {}\n}}",
                self.team, self.eye_height, self.pos, self.punch_angles, self.target_id)
    }
}

#[repr(C)]
pub struct Player {
    pub player_index: u32,
    valid: u32,
    _padding0: [u8; 8],
    // TODO: https://doc.rust-lang.org/std/ffi/struct.OsStr.html#method.to_str
    pub name: [c_char; 32],
    pub team: Team,
    pub health: u32,
    pub pos: glm::Vec3,
    pub view_angles: glm::Vec3,
    /// 30x8 bytes - getting updated one after another
    _footstep_history: [u64; 30]
}

impl Player {
    pub fn is_valid(&self) -> bool {
        self.valid > 0
    }

    pub fn name_cstr(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.name.as_ptr().cast()) }
    }

    pub fn name_str(&self) -> Result<&str> {
        self.name_cstr().to_str()
            .map_err(|err| err.into())
    }

    pub fn is_alive_and_playing(&self) -> bool {
        self.is_valid()
            && self.team != Team::Spectator
            && self.health > 0

    }
}

#[repr(C)]
pub struct RadarStruct {
    _padding: [u8; offsets::radar_struct::PLAYERARRAY],
    pub players: [Player; 64],
}