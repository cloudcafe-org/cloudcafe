use glam::{Quat, Vec3};
use mint::Vector2;
use stereokit::values::{MQuat, MVec3};

pub type UVec2 = Vector2<u32>;
pub type IVec2 = Vector2<i32>;

pub fn cart_2_cyl(cord: Vec3) -> Vec3 {
    let thing = coord_transforms::prelude::Vector3::new(cord.x as f64, cord.z as f64, cord.y as f64);

    let thing = coord_transforms::d3::cartesian2cylindrical(&thing);

    Vec3::new(thing.x as f32, thing.y as f32, thing.z as f32)
}

pub fn cyl_2_cart(cord: Vec3) -> Vec3 {
    let thing = coord_transforms::prelude::Vector3::new(cord.x as f64, cord.y as f64, cord.z as f64);

    let thing = coord_transforms::d3::cylindrical2cartesian(&thing);

    Vec3::new(thing.x as f32, thing.z as f32, thing.y as f32)
}

pub fn sphere_2_cart(cord: Vec3) -> Vec3 {
    let thing = coord_transforms::prelude::Vector3::new(cord.x as f64, cord.y as f64, cord.z as f64);

    let thing = coord_transforms::d3::spherical2cartesian(&thing);

    Vec3::new(thing.x as f32, thing.z as f32, thing.y as f32)
}
pub fn cart_2_sphere(cord: Vec3) -> Vec3 {
    let thing = coord_transforms::prelude::Vector3::new(cord.x as f64, cord.z as f64, cord.y as f64);

    let thing = coord_transforms::d3::cartesian2spherical(&thing);

    Vec3::new(thing.x as f32, thing.y as f32, thing.z as f32)
}

pub fn quat_lookat(from: impl Into<MVec3>, at: impl Into<MVec3>) -> Quat {
    let from = from.into();
    let at = at.into();
    let quat = unsafe {
        let quat = stereokit::sys::quat_lookat(&stereokit::values::vec3_from(from), &stereokit::values::vec3_from(at));
        stereokit::values::quat_to(quat)
    };
    Quat::from(quat)
}