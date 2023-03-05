use anyhow::{anyhow, Result, Context};
use lazy_static::lazy_static;

use super::signatures::{SignatureScanner, Signatures, SignatureAreas};
use super::overlay_structs::{Localplayer, PlayerArray};

#[derive(Clone, Copy)]
pub struct MemVars {
    // TODO: why does *mut [u8] and &'static Option<&'static mut _> allow Clone, but &'static mut u8 doesn't?
    on_ground_op_dec:   (usize, usize),
    on_ground_op_inc:   (usize, usize),
    on_ground:          usize,
    do_jump:            usize,
    do_attack_1:        usize,
    angles_x_op_read:   (usize, usize),
    angles:             usize,
    localplayer_base:   usize,
    player_array_base:  usize,
}

impl MemVars {
    pub fn on_ground_op_dec(&self)  -> &'static mut [u8] { unsafe { std::slice::from_raw_parts_mut(self.on_ground_op_dec.0 as _, self.on_ground_op_dec.1) } }
    pub fn on_ground_op_inc(&self)  -> &'static mut [u8] { unsafe { std::slice::from_raw_parts_mut(self.on_ground_op_inc.0 as _, self.on_ground_op_inc.1) } }
    pub fn on_ground(&self)         -> &'static mut u8 { unsafe { &mut *(self.on_ground as *mut _ )} }
    pub fn do_jump(&self)           -> &'static mut u8 { unsafe { &mut *(self.do_jump as *mut _) } }
    pub fn do_attack_1(&self)       -> &'static mut u8 { unsafe { &mut *(self.do_attack_1 as *mut _) } }
    pub fn angles_x_op_read(&self)  -> &'static mut [u8] { unsafe { std::slice::from_raw_parts_mut(self.angles_x_op_read.0 as _, self.angles_x_op_read.1) } }
    pub fn angles(&self)            -> &'static mut [f32; 3] { unsafe { &mut *(self.angles as *mut _) } }
    pub fn localplayer(&self)       -> &'static Option<&'static mut Localplayer> { unsafe { &*(self.localplayer_base as *const _ ) } }
    pub fn player_array(&self)      -> &'static PlayerArray { unsafe { &*(self.player_array_base as *const _ ) }}

    pub fn get() -> Result<Self> {
        lazy_static! {
            static ref VARS: Result<MemVars> = MemVars::read();
        }       
        let mem_vars = match &*VARS {
            Ok(mem_vars) => Ok(*mem_vars),
            Err(err) => Err(anyhow!("Failed to read MemVars: {err}")),
        };
        mem_vars
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
            angles_x_op_read:   read_slice_parts("angles_x_op_read")?,
            angles:             read_address("angles")? as _,
            localplayer_base:   read_address("localplayer_base")? as _,
            player_array_base:  read_address("playerarray_base")? as _,
        };
        Ok(mem_vars)
    }
}