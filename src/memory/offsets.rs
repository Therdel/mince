pub mod localplayer {
    use std::mem::size_of;
    use nalgebra_glm as glm;
    use crate::memory::overlay_structs;

    pub type Team = overlay_structs::Team;
    pub type Punch = glm::Vec3;
    pub type EyeHeight = f32;
    pub type Pos = glm::Vec3;
    
    pub const TEAM: usize = 0x9C;
    pub const PADDING_TEAM_EYE_HEIGHT: usize = EYE_HEIGHT - (TEAM + size_of::<Team>());
    pub const EYE_HEIGHT: usize = 0xF0;
    pub const PADDING_EYE_HEIGHT_POS: usize = POS - (EYE_HEIGHT + size_of::<EyeHeight>());
    pub const POS: usize = 0x260;
    pub const PADDING_POS_PUNCH: usize = PUNCH - (POS + size_of::<Pos>());
    pub const PUNCH: usize = 0xE48;
    pub const PADDING_PUNCH_TARGET: usize = TARGET_ID - (PUNCH + size_of::<Punch>());
    pub const TARGET_ID: usize = 0x14F0;
}

pub mod radar_struct {
    pub const PLAYERARRAY: usize = 0x28;
}