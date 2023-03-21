mod service;
mod windows_bindings;
mod values;
mod input;
mod sk_env;
mod asset_loader;
mod virtual_manager;
mod internal_os;

use std::{env, fs};
use color_eyre::Result;
use glam::{Mat4, Quat, Vec3};
use native_dialog::MessageType;
use stereokit::color_named::WHITE;
use stereokit::lifecycle::StereoKitContext;
use stereokit::material::{DEFAULT_ID_MATERIAL, Material, Transparency};
use stereokit::mesh::Mesh;
use stereokit::model::Model;
use stereokit::render::{RenderLayer, SphericalHarmonics, StereoKitRender};
use stereokit::Settings;
use stereokit::shader::Shader;
use stereokit::texture::Texture;
use stereokit::values::Color128;
use crate::asset_loader::load_assets;
use crate::internal_os::internal_mouse::IMouse;
use crate::sk_env::SkEnv;
use crate::values::IVec2;
use crate::virtual_manager::virtual_mouse::VMouse;

fn main() {
    match main2() {
        Ok(_) => {}
        Err(err) => {
            let err_msg = err.to_string();
            native_dialog::MessageDialog::new().set_type(MessageType::Error)
                .set_text(&err_msg)
                .show_alert().unwrap();
        }
    }
}
fn main2() -> Result<()> {
    //service::init()?;
    load_assets();
    let sk = Settings::default().init()?;
    let sk_env = SkEnv::new(&sk)?;
    let mut mouse = VMouse::new(&sk, 1.0)?;
    let mut internal_mouse = IMouse::new(IVec2::from([30, 30]));
    sk.run(|sk| {
        sk_env.draw(sk);
        internal_mouse.tick();
        mouse.update_pos(internal_mouse.delta_pos.x, internal_mouse.delta_pos.y);
        mouse.draw(sk, Vec3::new(0.0, 0.0, 0.0));
    }, |_| {});
    Ok(())
}

/*
let search_term = "chr";

    if let Ok(entries) = fs::read_dir("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs") {
        for entry in entries {
            if let Ok(entry) = entry {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains(search_term) {
                    println!("{}", entry.path().display());
                }
            }
        }
    }
 */