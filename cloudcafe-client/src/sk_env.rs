use std::thread;
use std::time::Duration;
use stereokit::lifecycle::{StereoKitContext, StereoKitDraw};
use stereokit::render::{RenderLayer, SphericalHarmonics, StereoKitRender};
use stereokit::texture::{Texture, TextureAddress, TextureFormat, TextureType};
use color_eyre::{Report, Result};
use color_eyre::eyre::Context;
use dxcapture::{Capture, Device};
use glam::{Mat4, Quat, Vec2, Vec3};
use glam::EulerRot::XYZ;
use lerp::Lerp;
use native_dialog::MessageType;
use stereokit::color_named::WHITE;
use stereokit::material::{Cull, DEFAULT_ID_MATERIAL_UNLIT, DepthTest, Material, Transparency};
use stereokit::model::Model;
use stereokit::shader::Shader;
use stereokit::values::Color128;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{HDC, ReleaseDC};
use crate::gamma_shader::{gamma_shader, start_menu_shader};
use crate::internal_os::internal_mouse::IMouse;
use crate::values::{cart_2_cyl, cyl_2_cart};
use crate::virtual_manager::VDesktop;
use crate::virtual_manager::virtual_mouse::VMouse;
use crate::windows_bindings::{get_cursor_pos, get_dc, get_pixel, main_monitor_dimensions, set_cursor_pos};

pub struct SkEnv {
    pub shader: Shader,
    pub skybox: Model,
    pub bridge_material: Material,
    pub bridge_lip: Model,
    pub bridge: Model,
    pub second_lip: Model,
    capture_tex: Texture,
    capture_material: Material,
    device: Device,
    capture: Capture,
    dc: HDC,
}
impl SkEnv {
    pub fn new(sk: &impl StereoKitContext) -> Result<Self> {
        let shader = Shader::from_name(sk, "default/shader_unlit_clip")?;
        let skybox = Model::from_mem(sk, "skybox.glb", include_bytes!("..\\assets\\skybox.glb"), Some(&shader))?;
        let bridge_material: Material = Material::copy_from_id(sk, DEFAULT_ID_MATERIAL_UNLIT)?;
        let bridge_lip = Model::from_mem(sk, "top_base.glb", include_bytes!("..\\assets\\top_base.glb"), Some(&shader))?;
        let bridge = Model::from_mem(sk, "bottom_base.glb", include_bytes!("..\\assets\\bottom_base.glb"), Some(&shader))?;
        bridge_lip.set_material(sk, 0, &bridge_material);
        bridge.set_material(sk, 0, &bridge_material);
        bridge_material.set_transparency(sk, Transparency::Blend);
        bridge_material.set_parameter(sk, "color", &Color128::new(0.2, 0.2, 0.2, 1.0));

        let device = match Device::new_from_displays(None) {
            Ok(device) => device,
            Err(_) => {
                native_dialog::MessageDialog::new().set_type(MessageType::Error).set_text("restart Windows, api call for finding displays no longer functions").show_alert().unwrap();
                thread::sleep(Duration::from_secs(2));
                panic!("need to restart windows");
            }
        };
        let capture = Capture::new(&device).unwrap();
        let capture_texture = Texture::create(sk, TextureType::ImageNoMips, TextureFormat::None)
            .ok_or(Report::msg("unable to create texture for capture"))?;
        let material = Material::create(sk, start_menu_shader(sk)).wrap_err("material create")?;
        capture_texture.set_address_mode(TextureAddress::Clamp);
        //material.set_texture(sk, "diffuse", &capture_texture).context("unable to set capture texture")?;
        material.set_shader(start_menu_shader(sk));
        material.set_transparency(sk, Transparency::Blend);

        let second_bridge = Model::from_mem(sk, "second_top_base.glb", include_bytes!("..\\assets\\top_base_2.glb"), Some(&shader))?;
        second_bridge.set_material(sk, 0, &material);
        //material.set_depth_test(sk, DepthTest::Always);
        //material.set_queue_offset(sk, -1);
        material.set_texture(sk, "diffuse", &capture_texture).unwrap();
        //bridge_material.set_texture(sk, "diffuse", &capture_texture).unwrap();




        Ok(Self {
            shader,
            skybox,
            bridge_material,
            bridge_lip,
            bridge,
            second_lip: second_bridge,
            capture_tex: capture_texture,
            capture_material: material,
            device,
            capture,
            dc:  get_dc(HWND(0)),
        })
    }
    pub fn draw(&self, sk: &StereoKitDraw, mut radius: f32, v_desktop: &mut VDesktop, internal_mouse: &mut IMouse) {
        let bridge_matrix = Mat4::from_scale_rotation_translation(Vec3::new(radius, radius, radius), Quat::IDENTITY, Vec3::new(0.0, -0.9, 0.0));
        radius *= 1.3;
        let second_bridge_matrix =
            Mat4::from_scale_rotation_translation(Vec3::new(radius, radius, radius), Quat::from_euler(XYZ, 0.0, -90.0_f32.to_radians(), 0.0), Vec3::new(0.0, -0.6, 0.0));
        let ray = v_desktop.v_mouse.gen_ray(sk, v_desktop.center, &second_bridge_matrix);
        let intersect = ray.model_intersect(&self.second_lip, Cull::None);
        if v_desktop.lock_cursor {
            if let Some(ray) = intersect {
                let bounds = self.second_lip.get_bounds(sk);
                let mut pos = Vec3::from(ray.pos);
                //pos.x += bounds.dimensions.x / 2.0;
                //pos.y += bounds.dimensions.y / 2.0;
                //pos.z += bounds.dimensions.z / 2.0;
                //pos.x /= bounds.dimensions.x;
                pos.y /= bounds.dimensions.y;
                //pos.z /= bounds.dimensions.z;

                let val = pos.x.atan2(pos.z);

                //println!("angle: {:?}, pos: {:?}", val, pos);
                if pos.y < -0.15 {
                    v_desktop.lock_cursor = false;
                    internal_mouse.lock_cursor = false;
                    let monitor_dimensions = main_monitor_dimensions();
                    let x = 0.0.lerp(monitor_dimensions.x as f32, val / -3.2);
                    set_cursor_pos(x as i32, monitor_dimensions.y as i32 - 30);
                }
            }
        } else if v_desktop.captured_window.is_none() {
            let mouse_pos = get_cursor_pos();
            let color = get_pixel(self.dc, mouse_pos.x, mouse_pos.y);
            //println!("color: {}", color.0);
            if color.0 == 592137 {
                //println!("is black");
                v_desktop.lock_cursor = true;
                internal_mouse.lock_cursor = true;
                v_desktop.v_mouse.pos.y += 0.1;
                // let monitor_dimensions = main_monitor_dimensions();
                // let cursor_pos = get_cursor_pos();
                // let x = cursor_pos.x as f32;
                // let val_x = monitor_dimensions.x as f32 / x;
                // let val_x = val_x * -3.2;
                // let v_pos = v_desktop.v_mouse.pos;
                // let cyl_pos = cart_2_cyl(v_pos);
                // v_desktop.v_mouse.pos = cyl_2_cart(Vec3::new(cyl_pos.x, cyl_pos.y, val_x));
                // v_desktop.v_mouse.pos.y += 0.1;
            }
        }

        for shared_tex in self.capture.rx.try_iter() {
            unsafe {
                self.capture_tex.set_surface(
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

        let scale = 0.15;
        self.skybox.draw(sk, Mat4::from_scale_rotation_translation(Vec3::new(scale, scale, scale), Quat::IDENTITY, Vec3::new(0.0, 0.0, 0.0)).into(),
                    WHITE, RenderLayer::Layer1);
        self.bridge_lip.draw(sk,
                             bridge_matrix.into(),
                   Color128::new(0.2, 0.2, 0.2, 0.5), RenderLayer::Layer1);
        self.bridge.draw(sk,
                         bridge_matrix.into(),
                    Color128::new(0.3, 0.3, 0.3, 0.8), RenderLayer::Layer1);
        radius *= 1.335;
        self.second_lip.draw(sk,
                             second_bridge_matrix.into(), Color128::new(1.0, 1.0, 1.0, 0.8), RenderLayer::Layer1);
    }
}