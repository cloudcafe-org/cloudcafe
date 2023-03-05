extern crate core;

mod popup_menu;
mod world;
//mod windows;
mod windows2;

use std::ffi::c_void;
use std::thread;
use std::time::Duration;
use dxcapture::Device;
use glam::{Mat4, Quat, Vec3};
use image::{ImageBuffer, Rgba};
use leftwm_layouts::geometry::Rect;
use leftwm_layouts::layouts::Layouts;
use stereokit::color_named::WHITE;
use stereokit::input::{Key, StereoKitInput};
use stereokit::lifecycle::LogFilter;
use stereokit::material::{DEFAULT_ID_MATERIAL_UNLIT_CLIP, Material};
use stereokit::mesh::Mesh;
use stereokit::model::Model;
use stereokit::render::{RenderLayer, StereoKitRender};
use stereokit::Settings;
use stereokit::shader::Shader;
use stereokit::texture::{Texture, TextureAddress, TextureFormat, TextureType};
use stereokit::ui::{MoveType, window, WindowType};
//use stereokit_sys::{color32, vec2};
use tokio::pin;
use ustr::ustr;
use win_screenshot::capture::capture_display;
use win_screenshot::prelude::window_list;
use win_screenshot::utils::find_window;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Win32::Graphics;
use windows::Win32::Graphics::Direct3D11::{D3D11_CPU_ACCESS_READ, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC, D3D11_USAGE_IMMUTABLE, D3D11_USAGE_STAGING, ID3D11Texture2D};
use crate::world::WorldModel;

fn main() {
    //windows2::run();
    second_main();
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

fn second_main() {
    let device = dxcapture::Device::default();
    let mut devices = vec![];
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
    //layout.increase_main_window_count();
    sk.run(|sk| {
        unsafe {
                for shared_tex in capture.rx.try_iter() {
                    let mut desc = D3D11_TEXTURE2D_DESC::default();
                    shared_tex.GetDesc(&mut desc);
                    texture.set_surface(std::mem::transmute(shared_tex), TextureType::ImageNoMips, desc.Format as i64, 0, 0, 1, true);
                }
            model.draw(sk, Mat4::from_scale_rotation_translation(Vec3::new(1.0, 1.0, 1.0), Quat::IDENTITY, Vec3::default()).into(), WHITE, RenderLayer::Layer1)
        }
        /*
        for i in &rects {
            let matrix = Mat4::from_scale_rotation_translation(
                Vec3::new(i.w as f32 / 100.0, i.h as f32 / 100.0, 1.0),
                Quat::IDENTITY,
                Vec3::new(i.x as f32 / 100.0, i.y as f32 / 100.0, 0.0)).into();
            model.draw(sk, matrix, WHITE, RenderLayer::Layer1);
        }
        */
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