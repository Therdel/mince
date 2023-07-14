use std::fmt::Display;

use crate::memory::{MemVars, overlay_structs::Player};
use crate::hooks::ViewAnglesReadHandler;

use nalgebra_glm as glm;

#[derive(Clone, Copy)]
pub enum TargetSelect {
    ByAngle{ angle_deg: f32 },
    ByDistance
}

#[derive(Clone, Copy)]
pub enum AimPoint {
    HeadStanding,
    Belly,
    FootPosition,
}

pub struct AimbotConfig {
    pub friendly_fire: bool,
    pub target_select: TargetSelect,
    pub aim_point: AimPoint
}

pub struct Target {
    pub player: &'static Player,
    pub distance: f32,
    pub aim_point: glm::Vec3,
    pub eye_target_vec: glm::Vec3,
    pub angle_with_crosshair_rad: f32
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Target {{\n\tplayer: {},\n\tdistance: {},\n\taim_point: {},\n\teye_target_vec: {},\n\tangle_with_crosshair: {}\n}}",
                self.player.name_str().unwrap_or("ERROR GETTING NAME"), self.distance, self.aim_point, self.eye_target_vec, self.angle_with_crosshair_rad)
    }
}

pub struct Aimbot {
    mem_vars: MemVars,
    config: AimbotConfig,
}

impl Aimbot {
    const AIM_MIN_DISTANCE: f32 = 10.0;
	const AIM_TARGET_OFFSET_HEAD: [f32; 3] = [9.141, -3.828, 0.078];
	const AIM_TARGET_OFFSET_BELLY: [f32; 3] = [0.0, 0.0, -25.0];
    
    pub fn new(mem_vars: MemVars, config: AimbotConfig) -> Self {
        Self { mem_vars, config }
    }

    pub fn find_target(&self) -> Option<Target> {
        let Some(localplayer) = self.mem_vars.localplayer() else { return None };
        let players = &self.mem_vars.radar_struct().players;
        let crosshair_angles_unit_vec = Self::view_angles_to_unit_vector(self.mem_vars.angles());

        let possible_targets = players.iter()
            .filter(|player|
                // ensure current target is valid, alive and actively playing (not spectating)
                player.is_alive_and_playing() &&
                // if friendly fire is off, target mustn't be of same team
                (self.config.friendly_fire || player.team != localplayer.team))
            .map(|player| {
                let aim_point = Self::get_target_aim_point(player, self.config.aim_point);
                let distance = glm::distance(&aim_point, self.mem_vars.eye_pos());
                let eye_target_vec = aim_point - *self.mem_vars.eye_pos();
                let angle_to_crosshair_rad = eye_target_vec.angle(&crosshair_angles_unit_vec);

                Target {
                    player, distance, aim_point, eye_target_vec, angle_with_crosshair_rad: angle_to_crosshair_rad
                }
            })
            .filter(|target|
                // prevent aiming at ourselves
                target.distance >= Self::AIM_MIN_DISTANCE &&
                // sort out targets outside FOV when selecting ByAngle
                match self.config.target_select {
                    TargetSelect::ByAngle { angle_deg } => target.angle_with_crosshair_rad.to_degrees() < angle_deg,
                    TargetSelect::ByDistance => true
                }
            );

        let minimize_crosshair_angle = |lhs: &Target, rhs: &Target|
            // aim at target closest to crosshair
            lhs.angle_with_crosshair_rad.total_cmp(&rhs.angle_with_crosshair_rad);
        let minimize_distance = |lhs: &Target, rhs: &Target|
            lhs.distance.total_cmp(&rhs.distance);

        let target_selector_function = match self.config.target_select {
            TargetSelect::ByAngle{..} => minimize_crosshair_angle,
            TargetSelect::ByDistance => minimize_distance
        };
        // select target
        possible_targets.min_by(target_selector_function)
    }

    fn view_angles_to_unit_vector(angles: &glm::Vec3) -> glm::Vec3 {
		// TODO: Document Source-engine coordinate system, euler angle ranges, euler-sequence
            // Part Answer: "extrinsic Tait-Bryan rotations following the right-hand rule, offset from the cardinal Z axis"
            // source: https://developer.valvesoftware.com/wiki/QAngle
		let yaw_rad = (angles.y + 180.0).to_radians();
		let pitch_rad = (angles.x - 90.0).to_radians();
		glm::vec3(
            pitch_rad.sin() * yaw_rad.cos(),
            pitch_rad.sin() * yaw_rad.sin(),
            // has to be negated, somehow the axis is flipped using this calculation
            pitch_rad.cos() * -1.0
        )
	}

    fn unit_vector_to_view_angles(unit_vector: &glm::Vec3) -> glm::Vec3 {
            let yaw_degrees = f32::atan2(unit_vector.y, unit_vector.x).to_degrees();
            let pitch_degrees = f32::acos(unit_vector.z / glm::length(unit_vector)).to_degrees();
        
            // TODO: Clarify pitch angle conversion
                // z: No roll angle, since a unit vector excludes
            glm::vec3(pitch_degrees - 90.0, yaw_degrees, 0.0)
    }

    fn get_target_aim_point(target: &Player, aim_point: AimPoint) -> glm::Vec3 {
        match aim_point {
            AimPoint::HeadStanding => {
                // include the target offset (aim a little at the belly)
                let target_head_offset: glm::Vec3 = Self::AIM_TARGET_OFFSET_HEAD.into();
                // rotate static target offset with targets heading angle:
                // source: https://developer.valvesoftware.com/wiki/QAngle
                let target_heading = target.view_angles.y.to_radians();
                let oriented_target_head_offset: glm::Vec3 = glm::rotate_z_vec3(&target_head_offset, target_heading);
                target.pos + oriented_target_head_offset
            },
            AimPoint::Belly => {
                let target_belly_offset: glm::Vec3 = Self::AIM_TARGET_OFFSET_BELLY.into();
                target.pos + target_belly_offset
            },
            AimPoint::FootPosition => target.pos,
        }
    }

    pub fn look_at(&self, target: &Target) {
        let view_angles = self.mem_vars.angles();
        let new_view_angles = Self::unit_vector_to_view_angles(&target.eye_target_vec);
        *view_angles = new_view_angles;
    }
}

impl ViewAnglesReadHandler for Aimbot {
    fn on_view_angles_read(&self) {
        // TODO: Indirect through input system
        // TODO: Re-inject aimbot config every call for unsynchronized changes
            // TODO: But we'll also do 
        let is_firing = *self.mem_vars.do_attack_1() == 5;
        if is_firing {
            if let Some(target) = self.find_target() {
                self.look_at(&target);
            }
        }
    }
}