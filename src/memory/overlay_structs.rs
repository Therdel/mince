use std::ffi::CStr;

use anyhow::Result;

use super::offsets;

#[repr(u32)]
#[derive(PartialEq)]
pub enum Team {
    Spectator = 1,
    Terrorist = 2,
    CounterTerrorist = 3,
}

#[repr(C)]
pub struct Localplayer {
    _padding0: [u8; offsets::LOCALPLAYER_TEAM],
    pub team: Team,
    _padding1: [u8; offsets::LOCALPLAYER_PADDING_TEAM_PUNCH],
    pub punch_angles: [f32; 3],
    _padding2: [u8; offsets::LOCALPLAYER_PADDING_PUNCH_TARGET],
    pub target_id: u32,
}

#[repr(C)]
pub struct Player {
    pub player_index: u32,
    valid: u32,
    _padding0: [u8; 8],
    // TODO: https://doc.rust-lang.org/std/ffi/struct.OsStr.html#method.to_str
    pub name: [u8; 32],
    pub team: Team,
    pub health: u32,
    pub pos: [f32; 3],
    pub view_angles: [f32; 3],
    /// 30x8 bytes - getting updated one after another
    _footstep_history: [u8; 60]
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

    pub fn is_playing_and_alive(&self) -> bool {
        self.is_valid()
            && self.team != Team::Spectator
            && self.health > 0

    }
}

#[repr(C)]
pub struct PlayerArray {
    _padding: [u8; offsets::PLAYERARRAY_PLAYERARRAY],
    pub players: [Player; 64],
}