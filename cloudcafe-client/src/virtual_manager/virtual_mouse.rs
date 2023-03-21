use std::f32::consts::PI;
use glam::{Mat4, Quat, Vec3};
use stereokit::lifecycle::{StereoKitContext, StereoKitDraw};
use stereokit::material::{DEFAULT_ID_MATERIAL_UNLIT, Material};
use stereokit::model::Model;
use color_eyre::Result;
use glam::EulerRot::XYZ;
use stereokit::color_named::BLACK;
use stereokit::lines::{line_addv, LinePoint};
use stereokit::pose::Pose;
use stereokit::render::RenderLayer;
use stereokit::shader::Shader;
use stereokit::values::{Color32, Ray};
use crate::values::{cart_2_cyl, cart_2_sphere, cyl_2_cart, quat_lookat, sphere_2_cart};


const MOUSE_MODEL: &[u8] = include_bytes!("..\\..\\assets\\mouse.glb");

pub struct VMouse {
    shader: Shader,
    model: Model,
    material: Material,
    pos: Vec3,
    y_sensitivity: f32,
    x_sensitivity: f32,
}
impl VMouse {
    pub fn new(sk: &impl StereoKitContext, radius: f32) -> Result<Self> {
        let shader = Shader::default(sk);
        let model = Model::from_mem(sk, "mouse.glb", MOUSE_MODEL, Some(&shader))?;
        let material = Material::copy_from_id(sk, DEFAULT_ID_MATERIAL_UNLIT)?;
        Ok(
            Self {
                shader,
                model,
                material,
                pos: cyl_2_cart(Vec3::new(radius, 0.0, 0.0)),
                y_sensitivity: 200.0,
                x_sensitivity: 300.0,
            }
        )
    }
    pub fn update_pos(&mut self, dx: i32, dy: i32) {
        if self.pos.y > 0.0 {
            let mut cyl_pos = cart_2_cyl(self.pos);
            cyl_pos.y += dx as f32 / self.x_sensitivity;
            cyl_pos.z += -dy as f32 / self.y_sensitivity;
            while cyl_pos.y.to_degrees() > 360.0 {
                cyl_pos.y -= 2.0 * PI;
            }
            while cyl_pos.y.to_degrees() < 0.0 {
                cyl_pos.y += 2.0 * PI;
            }
            let pos = cyl_2_cart(cyl_pos);

            self.pos = pos;
        }
        else {
            let mut cyl_pos = cart_2_sphere(self.pos);
            cyl_pos.z += dx as f32 / self.x_sensitivity;
            cyl_pos.y += dy as f32 / self.y_sensitivity;
            while cyl_pos.z.to_degrees() > 360.0 {
                cyl_pos.z -= 2.0 * PI;
            }
            while cyl_pos.z.to_degrees() < 0.0 {
                cyl_pos.z += 2.0 * PI;
            }
            let pos = sphere_2_cart(cyl_pos);

            self.pos = pos;
        }
    }
    pub fn draw(&self, sk: &StereoKitDraw, center: Vec3) {
        let quat = quat_lookat(center, self.pos);
        let rotated_quat = Quat::from_euler(XYZ, 0.0, 90.0_f32.to_radians(), 0.0).mul_quat(quat);
        let mouse_matrix = Mat4::from_scale_rotation_translation(
            Vec3::new(0.03, 0.03, 0.03),
            rotated_quat,
            self.pos
        );
        let pose = Pose::new(self.pos, quat);
        let ray = Ray {
            pos: self.pos.into(),
            dir: Quat::from(pose.orientation).mul_vec3(Vec3::new(0.0, 0.0, -1.0)).into(),
        };
        line_addv(sk, &LinePoint {
            point: self.pos.into(),
            thickness: 0.003,
            color: Color32::new(200, 200, 200, 200),
        }, &LinePoint {
            point: (Vec3::from(ray.pos) + Vec3::from(ray.dir)).into(),
            thickness: 0.003,
            color: Color32::new(200, 200, 200, 200),
        });
        self.model.draw(sk, mouse_matrix.into(), BLACK, RenderLayer::Layer1);
    }
}