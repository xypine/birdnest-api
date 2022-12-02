use crate::reaktor::drones::Drone;

pub const NDZ_CENTER_X: f64 = 250000.0;
pub const NDZ_CENTER_Y: f64 = 250000.0;
pub const NDZ_MIN_ALLOWED_DISTANCE: f64 = 100_000.0;
pub fn get_drone_distance_to_ndz(drone: &Drone) -> f64 {
    ((drone.position_x - NDZ_CENTER_X).powi(2) + (drone.position_y - NDZ_CENTER_Y).powi(2)).sqrt()
}
