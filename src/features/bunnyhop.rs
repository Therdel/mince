use std::time::{Duration, Instant};

use crate::ModuleName;

use module_maps::{find_module, ModuleMapping};

pub struct Bunnyhop {
    on_ground: *mut u32,
    do_jump: *mut u32
}

impl Bunnyhop {   
    pub fn new() -> Self {
        let by_file_name = |mapping: &ModuleMapping| mapping.file_name == ModuleName::Client.file_name();
        let client = find_module(by_file_name)
            .expect("module iteration failed")
            .expect("client module not found");

        let on_ground = unsafe { client.base.offset(0x4F82AC) };
        let do_jump = unsafe { client.base.offset(0x4F5D24) };

        let on_ground = on_ground as _;
        let do_jump = do_jump as _;

        Self { on_ground, do_jump }
    }

    pub fn auto_jump(&self, duration: Duration) {
        let begin = Instant::now();
        while Instant::now() < begin + duration {
            if self.is_on_ground() {
                unsafe { *self.do_jump = 5; }
            } else {
                unsafe { *self.do_jump = 4; }
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn is_on_ground(&self) -> bool {
        unsafe {
            let in_air = *self.on_ground == 0;
            !in_air
        }
    }
}