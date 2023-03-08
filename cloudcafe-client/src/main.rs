extern crate core;

mod popup_menu;
mod world;
mod window_management;
mod window_management_2;
mod simple_wm;

use std::f32::consts::PI;
use std::ffi::c_void;
use std::ops::{MulAssign, Sub};
use std::ptr::null;
use std::thread;
use std::time::Duration;
use dxcapture::{Capture, Device, enumerate_windows};
use glam::{EulerRot, Mat4, Quat, Vec2, Vec3};
use image::{ImageBuffer, Rgba};
use leftwm_layouts::geometry::Rect;
use leftwm_layouts::layouts::Layouts;
use stereokit::color_named::WHITE;
use stereokit::input::{ButtonState, Key, StereoKitInput};
use stereokit::lifecycle::{LogFilter, StereoKitContext, StereoKitDraw};
use stereokit::material::{DEFAULT_ID_MATERIAL, DEFAULT_ID_MATERIAL_FONT, DEFAULT_ID_MATERIAL_PBR, DEFAULT_ID_MATERIAL_UI, DEFAULT_ID_MATERIAL_UNLIT, DEFAULT_ID_MATERIAL_UNLIT_CLIP, Material};
use stereokit::mesh::Mesh;
use stereokit::model::Model;
use stereokit::render::{RenderLayer, StereoKitRender};
use stereokit::{color_named, Settings};
use stereokit::shader::Shader;
use stereokit::texture::{Texture, TextureAddress, TextureFormat, TextureType};
use stereokit::ui::{MoveType, window, WindowType};
//use stereokit_sys::{color32, vec2};
use tokio::pin;
use ustr::ustr;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Input::KeyboardAndMouse::{MOUSE_EVENT_FLAGS, TME_HOVER, TME_LEAVE, TRACKMOUSEEVENT};
use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageW, GetCursorPos, GetMessageW, HWND_TOPMOST, MoveWindow, MSG, PeekMessageW, PM_REMOVE, SetCursorPos, TranslateMessage, WM_INPUT, WM_MOUSEFIRST};
use crate::world::WorldModel;
use color_eyre::Result;
use stereokit::pose::Pose;
use stereokit_sys::tex_set_surface;


pub struct Window {
    hwnd: HWND,
    title: String,
    class_name: String,
    polar_pos: Vec3,
    texture: Texture,
    material: Material,
    device: Device,
    capture: Capture,
    mirror_mesh: Mesh,
    blit_shader: Shader,
    blit_material: Material,
    duplication_tex: Texture,
}

impl Window {
    pub fn from(sk: &impl StereoKitContext, window_info: dxcapture::window_finder::WindowInfo, polar_pos: Vec3) -> Result<Self> {
        //let shader = Shader::from_name(DEFAULT_ID)
        //let material = Material::create(sk, &Shader::from_file(sk, "assets/desktop_blit.sks").unwrap()).unwrap();
        let material = Material::copy_from_id(sk, DEFAULT_ID_MATERIAL_UNLIT)?;
        let texture = Texture::create(sk, TextureType::ImageNoMips, TextureFormat::None).unwrap();
        let blit_shader = Shader::from_file(sk, "assets/desktop_blit.sks")?;
        let blit_material = Material::create(sk, &blit_shader)?;
        let duplication_tex = Texture::create(sk, TextureType::ImageNoMips, TextureFormat::None).unwrap();
        blit_material.set_texture(sk, "source", &duplication_tex)?;
        texture.set_address_mode(TextureAddress::Clamp);
        material.set_texture(sk, "diffuse", &texture).unwrap();
        let device = Device::new_from_window(window_info.title.clone()).unwrap();
        let capture = Capture::new(&device).unwrap();
        Ok(Self {
            hwnd: unsafe { HWND(window_info.handle as isize)},
            title: window_info.title,
            class_name: window_info.class_name,
            polar_pos,
            texture,
            material,
            device,
            capture,
            mirror_mesh: Mesh::find(sk, "default/mesh_quad")?,
            blit_shader,
            blit_material,
            duplication_tex,
        })
    }
    pub fn get_dimensions(&self) -> (u32, u32) {
        let dim = self.texture.get_dimensions();
        (dim.0 as u32, dim.1 as u32)
    }
    pub fn draw(&mut self, sk: &StereoKitDraw, center_point: Vec3) {
        for shared_tex in self.capture.rx.try_iter() {
            unsafe {
                self.texture.set_surface(std::mem::transmute(shared_tex), TextureType::ImageNoMips, 87, 0, 0, 1, true);
            }
        }
        let dimensions = self.texture.get_dimensions();
        let scale_factor = 1000.0;
        let size = Vec2::new(dimensions.0 as f32 / scale_factor, dimensions.1 as f32 / scale_factor);
        let position = spherical_to_cartesian(&self.polar_pos);
        let rotation = look_at_quat(position, center_point, Vec3::new(0.0, 1.0, 0.0));
        let mut pose = Pose::new(position, rotation);
       // sk.render_blit(&self.texture, &self.blit_material);
        window(sk, self.title.as_str(), &mut pose, size.into(), WindowType::WindowNormal, MoveType::MoveNone, |ui| {
            unsafe { stereokit_sys::ui_layout_reserve(stereokit_sys::vec2 { x: size.x, y: size.y}, 0, 0.0) };
            sk.add_mesh(&self.mirror_mesh, &self.material, Mat4::from_scale_rotation_translation(Vec3::new(size.x, size.y, 1.0), Quat::IDENTITY, Vec3::new(0.0, -size.y/2.0, -0.01)).into(), WHITE, RenderLayer::LayerAll)
        });
    }
}


fn main() {
    //windows2::run();
    //second_main();
    //window_management::main();
    simple_wm::main();
    return;
    let sk = Settings::default().log_filter(LogFilter::Diagnostic).disable_unfocused_sleep(true).init().expect("Couldn't init stereokit");


    let mut windows = Vec::new();
    let mut polar_pos = Vec3::new(2.0, 0.1, PI / 2.0);
    for info in enumerate_windows() {
        windows.push(Window::from(&sk, info, polar_pos).unwrap());
        polar_pos.y += windows.last().unwrap().polar_pos.y + 0.12;
        if polar_pos.y >= PI {
            polar_pos.y = 0.01;
            polar_pos.z -= 0.6;
            println!("A");
        }
    }

    let mouse_model = Model::from_file(&sk, "assets/mouse.glb", None).unwrap();

    let t = Texture::from_cubemap_equirectangular(&sk, "assets/skytex2.hdr", true, 0).unwrap();
    sk.set_skytex(&t.0);
    sk.set_skylight(&t.1);


    let mut mouse_polar = Vec3::new(2.0, 0.1, 0.1);
    unsafe {
        SetCursorPos(600, 600);
    }
    let mut last_pos = POINT {
        x: 0,
        y: 0,
    };
    unsafe {
        GetCursorPos(&mut last_pos);
    }
    let mut pos_difference = (0, 0);
    let mut mouse_sensitivity = 200.0;

    let mut clicked_window: Option<usize> = None;
    let mut offset = Vec2::default();

    let mut first_run = true;
    sk.run(|sk| {
        if first_run {
            for window in &mut windows {
                window.draw(&sk, Vec3::new(0.0, 0.0, 0.0));
            }
            first_run = false;
        }
        {
            let mut current_pos = POINT::default();
            unsafe {
                GetCursorPos(&mut current_pos);
            }
            pos_difference.0 = (last_pos.x - current_pos.x);
            pos_difference.1 = (last_pos.y - current_pos.y);
            last_pos.x = current_pos.x;
            last_pos.y = current_pos.y;
        }

        mouse_polar.z -= pos_difference.1 as f32 / mouse_sensitivity;
        mouse_polar.y -= pos_difference.0 as f32 / mouse_sensitivity;
        if mouse_polar.z < 0.0 {
            mouse_polar.z = 0.001;
        }
        if mouse_polar.z > PI {
            mouse_polar.z = PI - 0.001;
        }
        let position = spherical_to_cartesian(&mouse_polar);
        let mut rotation = look_at_quat(position, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        rotation *= Quat::from_euler(EulerRot::XYZ, 0.0, 90.0, 0.0);
        let matrix = Mat4::from_scale_rotation_translation(Vec3::new(0.03, 0.03, 0.03), rotation, position);

        mouse_model.draw(sk, matrix.into(), color_named::BURLY_WOOD, RenderLayer::Layer1);
        if let Some(clicked_window) = clicked_window.as_ref() {
            let window = windows.get_mut(*clicked_window).unwrap();
            window.polar_pos.y = mouse_polar.y - offset.y;
            window.polar_pos.z = mouse_polar.z - offset.x;
        }
        for (i, window) in windows.iter_mut().enumerate() {
            if window.get_dimensions().0 == 0 || window.get_dimensions().1 == 0 {
                continue;
            }
            if sk.input_key(Key::MouseLeft).contains(ButtonState::JustActive) {
                if (mouse_polar.z - window.polar_pos.z).abs() < 0.1 {
                    if (mouse_polar.y - window.polar_pos.y).abs().rem_euclid(2.0*PI) < 0.25 * (window.get_dimensions().0 as f32 / 1000.0) {
                        if clicked_window.is_none() {
                            offset.x = mouse_polar.z - window.polar_pos.z;
                            offset.y = mouse_polar.y - window.polar_pos.y;
                            clicked_window.replace(i);
                        }
                    }
                }
            }
            let mouse_cart = spherical_to_cartesian(&mouse_polar);
            let window_cart = spherical_to_cartesian(&window.polar_pos);
            let dim_polar = Vec3::new(2.0, window.get_dimensions().1 as f32 / 1000.0, window.get_dimensions().0 as f32 / 1000.0);
            let window_polar = window.polar_pos;
            if mouse_polar.z > window_polar.z {
                if mouse_polar.z < window_polar.z + (dim_polar.z / 2.0) {
                    println!("{}", window.title);
                }
            }
            // if mouse_cart.y < window_cart.y {
            //     if mouse_cart.y > window_cart.y - (window.get_dimensions().1 as f32 / 1000.0) {
            //         if Vec2::new(mouse_cart.x, mouse_cart.z).distance(Vec2::new(window_cart.x, window_cart.z)) < (window.get_dimensions().0 as f32 / 2000.0) {
            //             println!("{}", window.title);
            //         }
            //     }
            // }
            // if (mouse_polar.z - window.polar_pos.z).abs() < (window.get_dimensions().1 as f32 / (1000.0 * 2.0)) {
            //     if mouse_polar.z > window.polar_pos.z {
            //         if (mouse_polar.y - window.polar_pos.y).abs() < (window.get_dimensions().0 as f32 / (1000.0 * 2.0)) {
            //             if mouse_polar.y > window.polar_pos.y {
            //                 println!("within the bounds of: {}", window.title);
            //             }
            //         }
            //     }
            // }
            if sk.input_key(Key::MouseLeft).contains(ButtonState::JustInactive) {
                clicked_window.take();
            }
            window.draw(&sk, Vec3::new(0.0, 0.0, 0.0));
        }

    }, |_|{});
    return;
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();
    let sk = Settings::default().log_filter(LogFilter::Diagnostic).disable_unfocused_sleep(true).init().expect("Couldn't init stereokit");
    let mut world = WorldModel::new(&sk);
    // let mut windows = windows::main(&sk);
    // let mirror_mesh = Mesh::find(&sk, "default/mesh_quad").unwrap();
    // for window1 in &windows {
    //     println!("dimesnions: {:?}", window1.texture.get_dimensions());
    // }
    sk.run(|sk| {
        world.draw(sk);
        // for window1 in &mut windows {
        //     let size = [window1.texture.get_dimensions().0 as f32 * 0.0004, window1.texture.get_dimensions().1 as f32 *0.0004].into();
        //
        //     window(sk, window1.name.as_str(), &mut window1.pose, size, WindowType::WindowNormal, MoveType::MoveExact, |ui| {
        //         unsafe { stereokit_sys::ui_layout_reserve(vec2 { x: size.x, y: size.y }, 0, 0.0) };
        //         sk.add_mesh(&mirror_mesh, &window1.material, Mat4::from_scale_rotation_translation(Vec3::new(size.x, size.y, 1.0), Quat::IDENTITY, Vec3::new(0.0, -size.y/2.0, -0.01)).into(), WHITE, RenderLayer::LayerAll)
        //     });
        // }
    }, |_| {});
}

fn cartesian_to_spherical(cartesian: &Vec3) -> Vec3 {
    let r: f32 = cartesian.length();
    if cartesian.x == 0.0 && cartesian.y == 0.0 {
        return Vec3::new(r, 0.0, 0.0);
    }
    let mut theta: f32 = (cartesian.y / cartesian.x).atan();
    let phi: f32 = (Vec2::new(cartesian.x, cartesian.y).length() / cartesian.z).atan();
    if cartesian.x < 0.0 && cartesian.y >= 0.0 && theta == 0.0 {
        theta = std::f32::consts::PI;
    } else if cartesian.x < 0.0 && cartesian.y < 0.0 && theta.signum() > 0.0 {
        theta -= std::f32::consts::PI;
    } else if cartesian.x < 0.0 && cartesian.y > 0.0 && theta.signum() < 0.0 {
        theta += std::f32::consts::PI;
    }
    Vec3::new(r, theta, phi)
}

fn spherical_to_cartesian(spherical: &Vec3) -> Vec3 {
    let (r, theta, phi) = (spherical.x, spherical.y, spherical.z);
    let x = r * phi.sin() * theta.cos();
    let y = r * phi.cos();
    let z = r * phi.sin() * theta.sin();
    Vec3::new(x, y, z)
}

fn look_at_quat(look_from_point: Vec3, look_at_point: Vec3, up_direction: Vec3) -> Quat {
    let forward_direction = (look_at_point - look_from_point).normalize();
    let right_direction = forward_direction.cross(up_direction).normalize();
    let up_direction = right_direction.cross(forward_direction);
    Quat::from_mat3(&glam::Mat3::from_cols(right_direction, up_direction, (-forward_direction)))
}

fn second_main() {
    let device = dxcapture::Device::default();
    let mut devices = vec![];
    let mut hwnd = dxcapture::enumerate_windows().get(0).unwrap().handle;
    for window in dxcapture::enumerate_windows() {
        if let Ok(device) = Device::new_from_window(window.title) {
            devices.push(device);
        }
    }
    let mut captures = vec![];
    for device in &devices {
        captures.push(dxcapture::Capture::new(device).unwrap());
    }
    let mut capture = dxcapture::Capture::new(devices.get(0).unwrap()).unwrap();

    let sk = Settings::default().log_filter(LogFilter::Diagnostic).disable_unfocused_sleep(true).init().expect("Couldn't init stereokit");
    let material = Material::copy_from_id(&sk, DEFAULT_ID_MATERIAL_UNLIT_CLIP).unwrap();
    let texture = Texture::create(&sk, TextureType::ImageNoMips, TextureFormat::None).unwrap();
    texture.set_address_mode(TextureAddress::Clamp);
    material.set_texture(&sk, "diffuse", &texture).unwrap();
    //let context = Device::get_immediate_context(&device.d3d_device).unwrap();
    //texture.set_surface(shared_tex as *mut c_void, TextureType::ImageNoMips, format as i64, 0, 0, 1, true);
    //texture.set_address_mode(TextureAddress::Clamp);
    //material.set_texture(sk, "diffuse", &texture).unwrap();

    //DirectXPixelFormat::B8G8R8A8UIntNormalized
    //let first = capture.get_raw_frame().unwrap();
    //let mut thing = capture.get_raw_frame().unwrap();
    //println!("thing_size: {}, {}", thing.width, thing.height);
    let mesh = Mesh::gen_cube(&sk, [0.5, 0.5, 0.5], 1).unwrap();
    let model = Model::from_mesh(&sk, &mesh, &material).unwrap();
    //let mut do_change = true;
    let defaults = Layouts::default();
    let mut layout = defaults.layouts.get(3).unwrap();
    let rects = leftwm_layouts::apply(layout, 5, &Rect::new(0, 0, 300, 300));

    let mouse_model = Model::from_file(&sk, "assets/mouse.glb", None).unwrap();

    let hwnd = unsafe {
         HWND(hwnd as isize)
    };

    //layout.increase_main_window_count();
    let mut amount = 0;
    let mut last_pos = POINT {
        x: 0,
        y: 0,
    };
    let mut fixed_pos = POINT {
        x: 0,
        y: 0,
    };
    unsafe {
        GetCursorPos(&mut last_pos);
    }
    let mut pos_difference = (0, 0);
    let mut mouse_pos = Vec3::new(0.0, 0.0, 0.0);
    let mut is_captured = true;
    let t = Texture::from_cubemap_equirectangular(&sk, "assets/skytex2.hdr", false, 0).unwrap();
    sk.set_skytex(&t.0);
    sk.set_skylight(&t.1);
    sk.run(|sk| {
        unsafe {
                for shared_tex in capture.rx.try_iter() {
                    //shared_tex.GetDesc(&mut desc);
                    texture.set_surface(std::mem::transmute(shared_tex), TextureType::ImageNoMips, 87, 0, 0, 1, true);
                }
            model.draw(sk, Mat4::from_scale_rotation_translation(Vec3::new(1.0, 1.0, 1.0), Quat::IDENTITY, Vec3::default()).into(), WHITE, RenderLayer::Layer1)
        }
        unsafe {
            MoveWindow(hwnd, 1, 1, 1920, 1080, true);
            let mut pos: POINT = POINT::default();
            GetCursorPos(&mut pos);
            pos_difference = (-pos.x + last_pos.x, -pos.y + last_pos.y);
            last_pos = pos;
            println!("pos: {:?}", pos);
            println!("pos_difference: {:?}", pos_difference);
        }
        if !is_captured {
            mouse_pos.x += sk.input_mouse().scroll_change / 50.0;
            mouse_pos.z -= pos_difference.0 as f32 / 200.0;
            mouse_pos.y += pos_difference.1 as f32 / 200.0;
            let mouse_rotation = Quat::IDENTITY;
            let mouse_matrix = Mat4::from_scale_rotation_translation(Vec3::new(0.1, 0.1, 0.1), mouse_rotation, mouse_pos).into();
            mouse_model.draw(sk, mouse_matrix, WHITE, RenderLayer::Layer1);
            if mouse_pos.z < 0.5 && mouse_pos.z > -0.5 {
                if mouse_pos.y < 0.5 && mouse_pos.y > -0.5 {
                    is_captured = true;
                    //unsafe { SetCursorPos(((mouse_pos.x + 0.5) * 1920.0) as i32, ((mouse_pos.y + 0.5) * 1920.0) as i32); }
                    mouse_pos.z = 0.0;
                    mouse_pos.y = 0.0;
                } else {
                    last_pos = fixed_pos;
                    unsafe { SetCursorPos(fixed_pos.x, fixed_pos.y); }
                }
            } else {
                last_pos = fixed_pos;
                unsafe { SetCursorPos(fixed_pos.x, fixed_pos.y); }
            }
        }
        if is_captured {
            if last_pos.x <= 4 {
                mouse_pos.z = -0.51; is_captured = false;
                fixed_pos = last_pos;
                fixed_pos.x = 4;
            }
            if last_pos.x >= 1921 {
                mouse_pos.z = 0.51; is_captured = false;
                fixed_pos = last_pos;
            }
            if last_pos.y <= 4 {
                mouse_pos.y = 0.51; is_captured = false;
                fixed_pos = last_pos;
                fixed_pos.y = 4;
            }
            if last_pos.y >= 1081 {
                mouse_pos.y = -0.51; is_captured = false;
                fixed_pos = last_pos;
            }
        }
    }, |_| {});

    // let mut buf2: Vec<color32> = vec![];
    // for i in buf.pixels {
    //     buf2.push(i);
    // }
    // unsafe {
    //     texture.set_surface(buf.pixels.as_mut_ptr() as *mut c_void, TextureType::ImageNoMips, TextureFormat::RGBA32 as i64, buf.width as i32, buf.height as i32, 1, true);
    // }
    // let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
    //     ImageBuffer::from_raw(buf.width, buf.height, buf.pixels).unwrap();
    // //img.save("screenshot.jpg").unwrap();
    // sk.run(|sk| {
    //
    // }, |_| {});
}



// extern "stdcall" {
//     fn LoadLibraryA(name: *const u8) ->
// }
//
// pub struct Library {
//
// }
// impl Library {
//     pub fn new(name: &str) -> Option<Self> {
//         let name = ustr(name);
//         let module = unsafe { LoadLibraryA(name.as_ptr()) };
//         todo!()
//     }
// }