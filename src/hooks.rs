use anyhow::Result;
use crate::memory::{MemVars, detour::{DetourOrder, DetourToMethod}};

pub trait OnGroundHandler {
    fn on_ground_land(&self);
    fn on_ground_leave(&self);
}

pub struct OnGroundHook {
    /// holds detour alive
    #[allow(unused)] detour_on_ground_op_dec: DetourToMethod,
    /// holds detour alive
    #[allow(unused)] detour_on_ground_op_inc: DetourToMethod,
}

impl OnGroundHook {
    pub fn install<T: OnGroundHandler>(mem_vars: &MemVars, instance: &T, order: DetourOrder) -> Result<Self> {
        let on_ground_land = T::on_ground_land;
        let on_ground_leave = T::on_ground_leave;
        unsafe {
            let detour_on_ground_op_dec =
                DetourToMethod::install(mem_vars.on_ground_op_dec(), instance, on_ground_leave, order)?;
            let detour_on_ground_op_inc =
                DetourToMethod::install(mem_vars.on_ground_op_inc(), instance, on_ground_land, order)?;
            Ok(Self {
                detour_on_ground_op_dec,
                detour_on_ground_op_inc,
            })
        }
    }
}

pub trait ViewAnglesReadHandler {
    fn on_view_angles_read(&self);
}

pub struct ViewAnglesReadHook {
    #[allow(unused)] detour_on_view_angles_read: DetourToMethod
}

impl ViewAnglesReadHook {
    pub fn install<T: ViewAnglesReadHandler>(mem_vars: &MemVars, instance: &T, order: DetourOrder) -> Result<Self> {
        unsafe {
            let detour_on_view_angles_read =
                DetourToMethod::install(mem_vars.angles_op_read(), instance, T::on_view_angles_read, order)?;
            Ok(Self { detour_on_view_angles_read })
        }
    }
}