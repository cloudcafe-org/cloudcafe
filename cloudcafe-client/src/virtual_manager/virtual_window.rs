use std::{mem, thread};
use color_eyre::eyre::Context;
use color_eyre::Report;
use dxcapture::{Capture, Device};
use stereokit::lifecycle::{StereoKitContext, StereoKitDraw};
use stereokit::material::{DepthTest, Material};
use stereokit::texture::{Texture, TextureAddress, TextureFormat, TextureType};
use crate::gamma_shader::gamma_shader;
use crate::internal_os::internal_window::IWindow;
use crate::windows_bindings::{Hwnd, is_window};
use color_eyre::Result;
use glam::{Vec2, Vec3};
use stereokit::mesh::Mesh;
use stereokit::model::Model;
use stereokit::pose::Pose;
use stereokit::render::RenderLayer;
use stereokit::values::Color128;
use crate::internal_os::FakeMonitor;
use crate::values::{IVec2, UVec2};

pub struct VWindow {
    pub window_capture: Option<WindowCapture>,
    pub internal_window: IWindow,
    hwnd: Hwnd,
    pose: Pose,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum IsWindowValid {
    Valid,
    Invalid,
}

impl VWindow {
    pub fn new(sk: &impl StereoKitContext, hwnd: Hwnd, fake_monitor: FakeMonitor, pose: Pose) -> Result<Self> {
        let internal_window = IWindow::new(hwnd, fake_monitor).wrap_err("internal window error")?;
        let window_capture = WindowCapture::new(sk, hwnd, internal_window.size().unwrap()).wrap_err("window capture error")?;
        Ok(Self {
            window_capture: Some(window_capture),
            internal_window,
            hwnd,
            pose,
        })
    }
    pub fn draw(&mut self, sk: &StereoKitDraw) -> IsWindowValid {
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
        if let Some(window_capture) = self.window_capture.as_ref() {
            let matrix = self.pose.as_matrix();
            window_capture.get_model().draw(sk, matrix, Color128::new_rgb(1.0, 1.0, 1.0), RenderLayer::Layer1);
        }
        return IsWindowValid::Valid;
    }
    fn send_msg_recapture_window() {
        let _ = thread::spawn(|| native_dialog::MessageDialog::new().set_text("unable to recapture changed window").show_alert().unwrap());
    }
}

pub struct WindowCapture {
    device: Device,
    capture: Capture,
    capture_texture: Texture,
    mesh: Mesh,
    model: Model,
    material: Material,
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
        let size = Vec2::new(window_size.x as f32 * 0.001, window_size.y as f32 * 0.001);
        let mesh = Mesh::gen_plane(sk, size, Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0), 1)?;
        let model = Model::from_mesh(sk, &mesh, &material).wrap_err("mesh")?;

        Ok(Self {
            device,
            capture,
            capture_texture,
            mesh,
            model,
            material,
        })
    }
    pub fn delete(self) {}
    pub fn get_model(&self) -> &Model {
        self.get_draw_texture();
        &self.model
    }
    pub fn get_draw_texture(&self) {
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