mod service;
mod windows_bindings;
mod values;
mod input;
mod sk_env;
mod asset_loader;
mod virtual_manager;
mod internal_os;
mod run_menu;

use std::{env, fs};
use std::ffi::c_int;
use std::ptr::null_mut;
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
use windows::core::HRESULT;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_LWIN, VK_RWIN};
use windows::Win32::UI::WindowsAndMessaging::{CallNextHookEx, DispatchMessageW, GetMessageW, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_KEYDOWN};
use crate::asset_loader::load_assets;
use crate::input::{Key, KeyboardMouseState};
use crate::internal_os::internal_mouse::IMouse;
use crate::run_menu::RunMenu;
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
    let mut mouse = VMouse::new(&sk, 0.85)?;
    let mut internal_mouse = IMouse::new(IVec2::from([30, 30]));
    let mut run_menu = RunMenu::new(&sk)?;
    let mut keyboard_mouse = KeyboardMouseState::new();
    sk.run(|sk| {
        sk_env.draw(sk);
        internal_mouse.tick();
        mouse.update_pos(internal_mouse.delta_pos.x, internal_mouse.delta_pos.y);
        mouse.draw(sk, Vec3::new(0.0, 0.0, 0.0));
        run_menu.draw(sk, &mut keyboard_mouse);
        if keyboard_mouse.get_input(Key::Windows).active {
            if keyboard_mouse.get_input(Key::Y).active {
                sk.quit();
            }
        }
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