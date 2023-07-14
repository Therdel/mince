use anyhow::{anyhow, Result, Context};
use lazy_static::lazy_static;
use nalgebra_glm as glm;

use super::signatures::{SignatureScanner, Signatures, SignatureAreas};
use super::overlay_structs::{Localplayer, Player, RadarStruct};


#[derive(Clone, Copy)]
pub struct MemVars {
    // TODO: why does *mut [u8] and &'static Option<&'static mut _> allow Clone, but &'static mut u8 doesn't?
    on_ground_op_dec:   (usize, usize),
    on_ground_op_inc:   (usize, usize),
    on_ground:          usize,
    do_jump:            usize,
    do_attack_1:        usize,
    eye_pos:            usize,
    angles_op_read:     (usize, usize),
    angles:             usize,
    localplayer_base:   usize,
    radar_struct_base:  usize,
}

impl MemVars {
    pub fn get() -> Result<Self> {
        lazy_static! {
            static ref VARS: Result<MemVars> = MemVars::read();
        }       
        #[allow(clippy::let_and_return)]
        let mem_vars = match &*VARS {
            Ok(mem_vars) => Ok(*mem_vars),
            Err(err) => Err(anyhow!("Failed to read MemVars: {err}")),
        };
        mem_vars
    }
    
    pub fn on_ground_op_dec(&self)  -> &'static mut [u8] {
        let slice = unsafe { std::slice::from_raw_parts_mut(self.on_ground_op_dec.0 as _, self.on_ground_op_dec.1) };
        slice
    }
    
    pub fn on_ground_op_inc(&self)  -> &'static mut [u8] {
        let slice = unsafe { std::slice::from_raw_parts_mut(self.on_ground_op_inc.0 as _, self.on_ground_op_inc.1) };
        slice
    }
    
    pub fn on_ground(&self)         -> &'static mut u8 {
        #[allow(clippy::let_and_return)]
        let reference = unsafe { &mut *(self.on_ground as *mut _ )};
        reference
    }
    
    pub fn do_jump(&self)           -> &'static mut u8 {
        #[allow(clippy::let_and_return)]
        let reference = unsafe { &mut *(self.do_jump as *mut _) };
        reference
    }
    
    pub fn do_attack_1(&self)       -> &'static mut u8 {
        #[allow(clippy::let_and_return)]
        let reference = unsafe { &mut *(self.do_attack_1 as *mut _) };
        reference
    }
    
    pub fn eye_pos(&self)            -> &'static mut glm::Vec3 {
        #[allow(clippy::let_and_return)]
        let reference = unsafe { &mut *(self.eye_pos as *mut _) };
        reference
    }
    
    pub fn angles_op_read(&self)  -> &'static mut [u8] {
        let slice = unsafe { std::slice::from_raw_parts_mut(self.angles_op_read.0 as _, self.angles_op_read.1) };
        slice
    }
    
    pub fn angles(&self)            -> &'static mut glm::Vec3 {
        #[allow(clippy::let_and_return)]
        let reference = unsafe { &mut *(self.angles as *mut _) };
        reference
    }
    
    pub fn localplayer(&self)       -> &'static Option<&'static mut Localplayer> {
        #[allow(clippy::let_and_return)]
        let reference = unsafe { &*(self.localplayer_base as *const _ ) };
        reference
    }
    
    pub fn radar_struct(&self)      -> &'static mut &'static mut RadarStruct {
        #[allow(clippy::let_and_return)]
        let reference = unsafe { &mut *(self.radar_struct_base as *mut _ ) };
        reference
    }
    
    // TODO: Is this a good place? Rest is just data, this is an operation on it
    pub fn poll_crosshair_player(&self) -> Option<&'static Player> {
        let Some(localplayer) = self.localplayer() else { return None };
        let is_aiming_at_object = localplayer.target_id > 0;
        if is_aiming_at_object {
            let player_array_index = localplayer.target_id - 1;
            self.radar_struct().players.get(player_array_index as usize)
        } else {
            None
        }
    }

    fn read() -> Result<Self> {
        let signatures = Signatures::new()?;
        let signature_areas = SignatureAreas::new(&signatures)?;

        let get_signature_area = |signature_area_name: &str| {
            signature_areas.areas.get(signature_area_name)
                .context(anyhow!("Signature area {signature_area_name} not found"))
        };

        let read_address = |signature_area_name: &str| -> Result<usize> {
            let signature_area = get_signature_area(signature_area_name)?;
            let scan_result = SignatureScanner::scan_signature(signature_area, &signatures)?;
            let scan_result: &[u8] = unsafe { &*scan_result};
            let address = usize::from_le_bytes(scan_result.try_into()?);
            Ok(address)
        };

        let read_slice_parts = |signature_area_name: &str| -> Result<(usize, usize)> {
            let signature_area = get_signature_area(signature_area_name)?;
            let slice_pointer = SignatureScanner::scan_signature(signature_area, &signatures)?;
            let slice = unsafe { &*slice_pointer };
            Ok((slice.as_ptr() as usize, slice.len()))
        };

        let mem_vars = MemVars {
            on_ground_op_dec:   read_slice_parts("on_ground_op_dec")?,
            on_ground_op_inc:   read_slice_parts("on_ground_op_inc")?,
            on_ground:          read_address("on_ground")?,
            do_jump:            read_address("do_jump")?,
            do_attack_1:        read_address("do_attack_1")?,
            eye_pos:            read_address("eye_pos")? as _,
            angles_op_read:     read_slice_parts("angles_op_read")?,
            angles:             read_address("angles")? as _,
            localplayer_base:   read_address("localplayer_base")? as _,
            radar_struct_base:  read_address("radar_struct_base")? as _,
        };
        Ok(mem_vars)
    }
}