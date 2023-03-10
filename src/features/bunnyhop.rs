use crate::memory::MemVars;
use crate::hooks::OnGroundHandler;

pub struct Bunnyhop {
    is_active: bool,
    mem_vars: MemVars,
}

impl Bunnyhop {
    pub fn new(mem_vars: &MemVars) -> Self {
        Self {
            is_active: false,
            mem_vars: *mem_vars
        }
    }

    pub fn set_active(&mut self, is_active: bool) {
        self.is_active = is_active;
        if is_active {
            // the fake jumps are only triggered by landing on the ground, not while being on ground
            // so if we're currently standing on the ground...
            if self.is_on_ground() {
                // we're manually faking the first jump
                *self.mem_vars.do_jump() = 5;
            }
        } else {
            // prevent bhop not starting
            // when jump key is down when you land on the ground and bhop is enabled
            *self.mem_vars.do_jump() = 4;
        }
    }

    fn is_on_ground(&self) -> bool {
        let in_air = *self.mem_vars.on_ground() == 0;
        !in_air
    }
}

impl OnGroundHandler for Bunnyhop {
    fn on_ground_land(&self) {
        if self.is_active {
            *self.mem_vars.do_jump() = 5;
        }
    }
    
    fn on_ground_leave(&self) {
        if self.is_active {
            *self.mem_vars.do_jump() = 4;
        }
    }
}