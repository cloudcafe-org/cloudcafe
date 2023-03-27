use std::f32::consts::PI;
use glam::{Mat4, Quat, Vec3};
use stereokit::lifecycle::{StereoKitContext, StereoKitDraw};
use stereokit::material::{DEFAULT_ID_MATERIAL_UNLIT, DepthTest, Material};
use stereokit::model::Model;
use color_eyre::Result;
use glam::EulerRot::XYZ;
use stereokit::color_named::{BLACK, DARK_GREY};
use stereokit::lines::{line_addv, LinePoint};
use stereokit::pose::Pose;
use stereokit::render::RenderLayer;
use stereokit::shader::Shader;
use stereokit::values::{Color128, Color32, MMatrix, Ray};
use crate::values::{cart_2_cyl, cart_2_sphere, cyl_2_cart, quat_lookat, sphere_2_cart};

const POINT_MODEL: &[u8] = include_bytes!("..\\..\\assets\\mouse.glb");
const RESIZE_MODEL: &[u8] = include_bytes!("..\\..\\assets\\resize_cursor.glb");

pub struct VMouse {
    mouse_cursor: MouseCursor,
    pub(crate) pos: Vec3,
    y_sensitivity: f32,
    x_sensitivity: f32,
}
#[derive(Debug, Copy, Clone)]
pub enum CursorType {
    Point,
    Resize(ResizeType),
}
#[derive(Debug, Copy, Clone)]
pub enum ResizeType {
    Vertical,
    Horizontal,
    MixedLeft,
    MixedRight,
}
pub struct MouseCursor {
    point_model: PointModel,
    resize_model: ResizeModel,
    color: Color128,
    cursor_type: CursorType,
}
impl MouseCursor {
    pub fn new(sk: &impl StereoKitContext) -> Result<Self> {
        Ok(Self{
            point_model: PointModel::new(sk)?,
            resize_model: ResizeModel::new(sk)?,
            color: DARK_GREY,
            cursor_type: CursorType::Resize(ResizeType::Vertical),
        })
    }
    pub fn draw(&self, sk: &StereoKitDraw, matrix: impl Into<MMatrix>) {
        let matrix = matrix.into();
        match self.cursor_type {
            CursorType::Point => {
                self.point_model._model.draw(sk, matrix, self.color, RenderLayer::Layer1);
            }
            CursorType::Resize(resize_type) => {
                let (_, mut r, t) = Mat4::from(matrix).to_scale_rotation_translation();
                r.z = 0_f32.to_radians();
                r.x = 0.0_f32.to_radians();
                r = r.mul_quat(Quat::from_euler(XYZ, 0.0f32.to_radians(), 0.0f32.to_radians(), 90.0f32.to_radians()));
                match resize_type {
                    ResizeType::Vertical => {
                        r = r.mul_quat(Quat::from_euler(XYZ, 0.0, 0.0f32.to_radians(), 0.0))
                    }
                    ResizeType::Horizontal => {
                        r = r.mul_quat(Quat::from_euler(XYZ, 0.0, 90.0f32.to_radians(), 0.0))
                    }
                    ResizeType::MixedLeft => {
                        r = r.mul_quat(Quat::from_euler(XYZ, 0.0, 45.0f32.to_radians(), 0.0))
                    }
                    ResizeType::MixedRight => {
                        r = r.mul_quat(Quat::from_euler(XYZ, 0.0, 225.0f32.to_radians(), 0.0))
                    }
                }
                let mut matrix = Mat4::from_scale_rotation_translation(Vec3::new(0.4, 0.4, 0.4), r, t).into();
                self.resize_model._model.draw(sk, matrix, self.color, RenderLayer::Layer1);
            }
        }
    }
}
struct PointModel {
    _shader: Shader,
    _model: Model,
    _material: Material,
}
impl PointModel {
    pub fn new(sk: &impl StereoKitContext) -> Result<Self> {
        let shader = Shader::default(sk);
        let model = Model::from_mem(sk, "mouse.glb", POINT_MODEL, Some(&shader))?;
        let material = Material::copy_from_id(sk, DEFAULT_ID_MATERIAL_UNLIT)?;
        material.set_depth_test(sk, DepthTest::Always);
        material.set_queue_offset(sk, 500);
        Ok(Self{
            _shader: shader,
            _model: model,
            _material: material,
        })
    }
}
struct ResizeModel {
    _shader: Shader,
    _model: Model,
    _material: Material,
}
impl ResizeModel {
    pub fn new(sk: &impl StereoKitContext) -> Result<Self> {
        let shader = Shader::default(sk);
        let model = Model::from_mem(sk, "resize_cursor.glb", RESIZE_MODEL, Some(&shader))?;
        let material = Material::copy_from_id(sk, DEFAULT_ID_MATERIAL_UNLIT)?;
        material.set_depth_test(sk, DepthTest::Always);
        material.set_queue_offset(sk, 500);
        Ok(Self{
            _shader: shader,
            _model: model,
            _material: material,
        })
    }
}

impl VMouse {
    pub fn new(sk: &impl StereoKitContext, radius: f32) -> Result<Self> {
        Ok(Self {
                mouse_cursor: MouseCursor::new(sk)?,
                pos: cyl_2_cart(Vec3::new(radius, 0.0, 0.0)),
                y_sensitivity: 300.0,
                x_sensitivity: 500.0,
            })
    }
    pub fn update_pos(&mut self, dx: i32, dy: i32) {
        if dx == 0 && dy == 0 {
            return;
        }
        if self.pos.y > -0.7 {
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
            self.pos.y += 0.7;
            let mut cyl_pos = cart_2_sphere(self.pos);
            cyl_pos.z += dx as f32 / self.x_sensitivity;
            cyl_pos.y += dy as f32 / self.y_sensitivity;
            while cyl_pos.z.to_degrees() > 360.0 {
                cyl_pos.z -= 2.0 * PI;
            }
            while cyl_pos.z.to_degrees() < 0.0 {
                cyl_pos.z += 2.0 * PI;
            }
            if cyl_pos.y >= PI - (PI * 0.35) {
                cyl_pos.y = PI - (PI * 0.35);
            }
            let mut pos = sphere_2_cart(cyl_pos);
            pos.y -= 0.7;
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
            color: Color32::new(0, 0, 0, 200),
        }, &LinePoint {
            point: (Vec3::from(ray.pos) + Vec3::from(ray.dir)).into(),
            thickness: 0.003,
            color: Color32::new(10, 10, 10, 150),
        });
        self.mouse_cursor.draw(sk, mouse_matrix);
    }
    pub fn gen_ray(&self, sk: &StereoKitDraw, center: Vec3, matrix: &Mat4) -> Ray {
        let quat = quat_lookat(center, self.pos);
        let pose = Pose::new(self.pos, quat);
        let ray = Ray {
            pos: self.pos.into(),
            dir: Quat::from(pose.orientation).mul_vec3(Vec3::new(0.0, 0.0, -1.0)).into(),
        };

        let mat = matrix.inverse();
        let ray = Ray {
            pos: mat.transform_vector3(ray.pos.into()).into(),
            dir: mat.transform_point3(ray.dir.into()).into(),
        };
        ray
    }
    pub fn set_cursor_type(&mut self, cursor_type: CursorType) {
        self.mouse_cursor.cursor_type = cursor_type;
    }
}