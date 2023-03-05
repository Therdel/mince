use std::mem::size_of;

/// source: https://stackoverflow.com/a/70224634
fn _size_of_return_type<F, T, U>(_f: F) -> usize
where
    F: FnOnce(T) -> U
{
    std::mem::size_of::<U>()
}

pub type LocalplayerTeam = u32;
pub type LocalplayerPunch = [f32; 3];
// pub type LocalplayerTargetId = u32;

pub const LOCALPLAYER_TEAM: usize = 0x9C;
pub const LOCALPLAYER_PADDING_TEAM_PUNCH: usize = LOCALPLAYER_PUNCH_X - (LOCALPLAYER_TEAM + size_of::<LocalplayerTeam>());
pub const LOCALPLAYER_PUNCH_X: usize = 0xE48;
pub const LOCALPLAYER_PADDING_PUNCH_TARGET: usize = LOCALPLAYER_TARGET_ID - (LOCALPLAYER_PUNCH_X + size_of::<LocalplayerPunch>());
pub const LOCALPLAYER_TARGET_ID: usize = 0x14F0;
pub const PLAYERARRAY_PLAYERARRAY: usize = 0x28;