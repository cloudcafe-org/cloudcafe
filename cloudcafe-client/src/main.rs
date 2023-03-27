mod service;
mod windows_bindings;
mod values;
mod input;
mod sk_env;
mod asset_loader;
mod virtual_manager;
mod internal_os;
mod run_menu;
mod gamma_shader;

use std::{env, fs};
use std::ffi::c_int;
use std::ptr::null_mut;
use color_eyre::{Report, Result};
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
use crate::virtual_manager::VDesktop;
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
    let console_hwnd = service::init()?.ok_or(Report::msg("no console hwnd"))?;
    load_assets();
    let mut radius = 1.2;
    let sk = Settings::default().render_scaling(6.0).app_name("Cloudcafe XR Desktop").disable_unfocused_sleep(true).init()?;
    let sk_env = SkEnv::new(&sk)?;
    let mut internal_mouse = IMouse::new(IVec2::from([300, 300]));
    let mut run_menu = RunMenu::new(&sk)?;
    let mut keyboard_mouse = KeyboardMouseState::new();
    let mut virtual_desktop = VDesktop::new(&sk, console_hwnd, radius)?;
    internal_mouse.tick();
    sk.run(|sk| {
        sk_env.draw(sk, radius);
        internal_mouse.tick();
        run_menu.draw(sk, &mut keyboard_mouse, radius);
        if keyboard_mouse.get_input(Key::Windows).active {
            if keyboard_mouse.get_input(Key::Q).active {
                sk.quit();
            }
        }
        virtual_desktop.draw(sk, &mut internal_mouse, &mut keyboard_mouse, &mut radius);
        keyboard_mouse.reset_active();
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