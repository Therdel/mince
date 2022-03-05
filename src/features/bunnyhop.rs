use std::time::{Duration, Instant};

use module_maps::{find_module, ModuleMapping};

use crate::ModuleName::Client;

pub struct Bunnyhop {
    on_ground: *mut u8,
    do_jump: *mut u8
}

impl Bunnyhop {   
    pub fn new() -> Self {
        let is_client = |mapping: &ModuleMapping| {
            mapping.file_name == Client.file_name()
        };
        let client = find_module(is_client)
            .expect("module iteration failed")
            .expect("client module not found");

        
        let on_ground;
        let do_jump;
        unsafe {
            on_ground = client.base.offset(
                #[cfg(target_os="windows")]
                0x4F82AC,
                #[cfg(target_os="linux")]
                0xB9E7AC,
            );
            
            do_jump = client.base.offset(
                #[cfg(target_os="windows")]
                0x4F5D24,
                #[cfg(target_os="linux")]
                0xBEE4E8,
            );
        }

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