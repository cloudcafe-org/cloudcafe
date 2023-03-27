use std::{mem, thread};
use std::collections::HashMap;
use color_eyre::eyre::Context;
use color_eyre::Report;
use dxcapture::{Capture, Device};
use stereokit::lifecycle::{StereoKitContext, StereoKitDraw};
use stereokit::material::{DEFAULT_ID_MATERIAL_UNLIT, DepthTest, Material};
use stereokit::texture::{Texture, TextureAddress, TextureFormat, TextureType};
use crate::gamma_shader::gamma_shader;
use crate::internal_os::internal_window::IWindow;
use crate::windows_bindings::{Hwnd, is_window};
use color_eyre::Result;
use glam::{Mat4, Quat, Vec2, Vec3};
use stereokit::color_named;
use stereokit::color_named::WHITE;
use stereokit::mesh::{Mesh, Vertex};
use stereokit::model::Model;
use stereokit::pose::Pose;
use stereokit::render::RenderLayer;
use stereokit::values::{Color128, Color32};
use windows::Win32::Foundation::HWND;
use crate::internal_os::FakeMonitor;
use crate::values::{IVec2, UVec2};
use crate::virtual_manager::desktop_capture::CaptureDesktop;

pub struct VWindow {
    pub window_capture: Option<WindowCapture>,
    pub capture_desktop: CaptureDesktop,
    pub internal_window: IWindow,
    pub(crate) grab_bar: GrabBar,
    hwnd: Hwnd,
    pub(crate) pose: Pose,
    pub z_depth: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum IsWindowValid {
    Valid,
    Invalid,
}

impl VWindow {
    pub fn new(sk: &impl StereoKitContext, hwnd: Hwnd, fake_monitor: FakeMonitor, pose: Pose, z_depth: u32, capture_desktop: CaptureDesktop) -> Result<Self> {
        let internal_window = IWindow::new(hwnd, fake_monitor).wrap_err("internal window error")?;
        let window_capture = WindowCapture::new(sk, hwnd, internal_window.size().unwrap()).wrap_err("window capture error")?;
        let grab_bar = GrabBar::new(sk)?;
        Ok(Self {
            window_capture: Some(window_capture),
            capture_desktop,
            internal_window,
            grab_bar,
            hwnd,
            pose,
            z_depth,
        })
    }
    pub fn draw(&mut self, sk: &StereoKitDraw, radius: f32, focused: bool) -> IsWindowValid {
        if !is_window(self.hwnd) {
            return IsWindowValid::Invalid
        }
        if self.internal_window.size_changed() {
            if self.internal_window.size().unwrap().x == 0 || self.internal_window.size().unwrap().y == 0 {
                return IsWindowValid::Invalid;
            }
            match self.internal_window.update_size() {
                None => return IsWindowValid::Invalid,
                _ => {}
            }
            drop(self.window_capture.take());
            if let Ok(window_capture) = WindowCapture::new(sk, self.hwnd, self.internal_window.size().unwrap()) {
                self.window_capture.replace(window_capture);
            } else {
                return IsWindowValid::Invalid;
            }
        }
        self.update_radius(radius);
        if let Some(window_capture) = self.window_capture.as_ref() {
            window_capture.material.set_queue_offset(sk, self.z_depth as i32 - 200);
            window_capture.get_model(focused, &self.capture_desktop).draw(sk, self.matrix().unwrap().into(), Color128::new_rgb(1.0, 1.0, 1.0), RenderLayer::Layer1);
            self.grab_bar.draw(sk, self.grab_bar_matrix(), self.z_depth as i32 - 200);
        }
        return IsWindowValid::Valid;
    }
    pub fn grab_bar_matrix(&self) -> Mat4 {
        let size = self.internal_window.size().unwrap();
        self.grab_bar.gen_matrix(self.pose, size.x as f32 * 0.0005, size.y as f32 * 0.0005)
    }
    pub fn update_radius(&mut self, radius: f32) {

    }
    pub fn matrix(&self) -> Option<Mat4> {
        let scale = self.window_capture.as_ref()?.scale();
        Some(self.pose.pose_matrix(Vec3::new(scale.x * 0.5, scale.y * 0.5, 1.0)).into())
    }
    fn send_msg_recapture_window() {
        let _ = thread::spawn(|| native_dialog::MessageDialog::new().set_text("unable to recapture changed window").show_alert().unwrap());
    }
}

pub struct GrabBar {
    pub(crate) model: Model,
    mesh: Mesh,
    material: Material,
}
impl GrabBar {
    pub fn new(sk: &impl StereoKitContext) -> Result<Self> {
        let material = Material::copy_from_id(sk, DEFAULT_ID_MATERIAL_UNLIT)?;
        let mesh = Mesh::gen_plane(sk, [1.0, 1.0], Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0), 1)?;
        let model = Model::from_mesh(sk, &mesh, &material).wrap_err("mesh")?;
        material.set_depth_test(sk, DepthTest::Always);
        Ok(Self {
            model,
            mesh,
            material,
        })
    }
    pub fn draw(&self, sk: &StereoKitDraw, matrix: Mat4, queue_offset: i32) {
        self.material.set_queue_offset(sk, queue_offset);
        self.model.draw(sk, matrix.into(), color_named::BURLY_WOOD, RenderLayer::Layer1);
    }
    pub fn gen_matrix(&self, mut pose: Pose, window_width: f32, window_height: f32) -> Mat4 {
        let height = 0.025;
        let pos = Mat4::from_translation(Quat::from(pose.orientation).mul_vec3(Vec3::new(0.0, height, 0.0)));

        pose.position.y += window_height / 2.0;
        pose.position.y -= height / 2.0;
        let new_pos = (pos.transform_point3(pose.position.into()));
        Mat4::from_scale_rotation_translation(Vec3::new(window_width, height, 1.0), pose.orientation.into(), new_pos)
    }
}

pub struct WindowCapture {
    device: Device,
    capture: Capture,
    capture_texture: Texture,
    pub mesh: Mesh,
    pub model: Model,
    material: Material,
    size: Vec2,
}

impl WindowCapture {
    pub fn new(sk: &impl StereoKitContext, hwnd: Hwnd, window_size: UVec2) -> Result<Self> {
        let device = Device::new_from_hwnd(unsafe { mem::transmute(hwnd.0) }).map_err(|e| Report::msg(e.to_string()))
            .wrap_err("device from hwnd")?;
        let capture = Capture::new(&device).map_err(|e| Report::msg(e.to_string()))
            .wrap_err("capture from device")?;
        let capture_texture = Texture::create(sk, TextureType::ImageNoMips, TextureFormat::None)
            .ok_or(Report::msg("unable to create texture for capture"))?;
        let material = Material::create(sk, gamma_shader(sk)).wrap_err("material create")?;
        capture_texture.set_address_mode(TextureAddress::Clamp);
        material.set_texture(sk, "diffuse", &capture_texture).context("unable to set capture texture")?;
        material.set_shader(gamma_shader(sk));
        material.set_depth_test(sk, DepthTest::Always);
        material.set_queue_offset(sk, -1);
        let size = Vec2::new(window_size.x as f32 * 0.001, window_size.y as f32 * 0.001);
        //let mesh = Mesh::gen_plane(sk, size, Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0), 1)?;
        let mesh = Self::gen_mesh(sk, 0.0, 0.0, 1.0, 1.0)?;
        let model = Model::from_mesh(sk, &mesh, &material).wrap_err("mesh")?;

        Ok(Self {
            device,
            capture,
            capture_texture,
            mesh,
            model,
            material,
            size,
        })
    }
    pub fn delete(self) {}
    pub fn get_model(&self, focused: bool, capture_desktop: &CaptureDesktop) -> &Model {
        self.get_draw_texture(focused, capture_desktop);
        &self.model
    }
    pub fn scale(&self) -> Vec2 {
        self.size
    }
    pub fn gen_mesh(sk: &impl StereoKitContext, bottom_left: f32, bottom_right: f32, top_left: f32, top_right: f32) -> Result<Mesh> {
        let mesh = Mesh::create(sk)?;
        let mut verts = vec![];
        let mut inds = vec![];

        verts.push(Vertex {
            pos: Vec3::new(-0.5, -0.5, 0.0).into(),
            norm: Vec3::new(0.0, 0.0, 1.0).into(),
            uv: Vec2::new(bottom_left, top_right).into(),
            col: Color32::from(WHITE),
        });
        verts.push(Vertex {
            pos: Vec3::new(-0.5, 0.5, 0.0).into(),
            norm: Vec3::new(0.0, 0.0, 1.0).into(),
            uv: Vec2::new(bottom_left, bottom_right).into(),
            col: Color32::from(WHITE),
        });
        verts.push(Vertex {
            pos: Vec3::new(0.5, 0.5, 0.0).into(),
            norm: Vec3::new(0.0, 0.0, 1.0).into(),
            uv: Vec2::new(top_left, bottom_right).into(),
            col: Color32::from(WHITE),
        });
        inds.push(2);
        inds.push(1);
        inds.push(0);
        verts.push(Vertex {
            pos: Vec3::new(0.5, -0.5, 0.0).into(),
            norm: Vec3::new(0.0, 0.0, 1.0).into(),
            uv: Vec2::new(top_left, top_right).into(),
            col: Color32::from(WHITE),
        });
        inds.push(3);
        inds.push(2);
        inds.push(0);
        mesh.set_verts(sk, &verts, true);
        mesh.set_indices(sk, &inds);
        Ok(mesh)
    }
    pub fn get_mesh(&self) -> &Mesh {
        &self.mesh
    }
    pub fn get_draw_texture(&self, focused: bool, desktop_capture: &CaptureDesktop) {
        match focused {
            true => {
                for shared_tex in desktop_capture.0.lock().unwrap().capture.rx.try_iter() {
                    unsafe {
                        self.capture_texture.set_surface(
                            std::mem::transmute(shared_tex),
                            TextureType::ImageNoMips,
                            87,
                            0,
                            0,
                            1,
                            true,
                        )
                    }
                }
            }
            false => {
                for shared_tex in self.capture.rx.try_iter() {
                    unsafe {
                        self.capture_texture.set_surface(
                            std::mem::transmute(shared_tex),
                            TextureType::ImageNoMips,
                            87,
                            0,
                            0,
                            1,
                            true,
                        )
                    }
                }
            }
        }
    }
}