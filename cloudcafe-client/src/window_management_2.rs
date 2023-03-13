use std::collections::HashMap;
use std::hash::Hash;
use std::mem;
use dxcapture::{Capture, Device, enumerate_windows};
use dxcapture::window_finder::WindowInfo;
use glam::{Mat4, Quat, Vec2, Vec3};
use mint::Vector2;
use stereokit::lifecycle::{StereoKitContext, StereoKitDraw};
use stereokit::material::{DEFAULT_ID_MATERIAL_UNLIT, Material};
use stereokit::mesh::Mesh;
use stereokit::texture::{Texture, TextureFormat, TextureType};
use windows::Win32::Foundation::{BOOL, HWND, POINT, RECT};

pub trait WindowManager<MouseT: Mouse, WindowT: Window, WindowId: Hash + Eq + Clone> {
    fn mouse_ref(&self) -> &MouseT;
    fn mouse_mut(&mut self) -> &mut MouseT;

    fn windows_ref(&self) -> &HashMap<WindowId, WindowT>;
    fn windows_mut(&mut self) -> &mut HashMap<WindowId, WindowT>;

    fn captured(&mut self) -> &mut Option<WindowId>;

    fn draw(&mut self, sk: &StereoKitDraw) {
        let mut invalid_windows = vec![];
        for (window_id, window) in self.windows_ref() {
            if !window.is_valid() {
                invalid_windows.push(window_id.clone());
                continue;
            }
        }
        if let Some(captured_id) = self.captured().take() {
            let window = self.windows_ref().get(&captured_id).unwrap();
            let internal_mouse_pos = self.mouse_ref().internal_position();
            if window.internal_position().0.x > internal_mouse_pos.0.x
                || window.internal_position().0.y > internal_mouse_pos.0.y
                || window.internal_position().0.x + window.internal_size().x < internal_mouse_pos.0.x
                || window.internal_position().0.y + window.internal_size().y < internal_mouse_pos.0.y {
                let new_virtual_mouse_pos = window.map_relative_to_virtual_pos(RelativePos(Vector2::from([internal_mouse_pos.0.x as i32, internal_mouse_pos.0.y as i32])));
                self.mouse_mut().set_virtual_position(new_virtual_mouse_pos);
            }
            else {
                self.captured().replace(captured_id);
            }
        } else {
            let virtual_mouse_pos = self.mouse_ref().virtual_position();
            let mut optional_internal_pos: Option<InternalPos> = None;
            let mut optional_window_id: Option<WindowId> = None;
            for (window_id, window) in self.windows_ref() {
                if invalid_windows.contains(window_id) {
                    continue;
                }
                if let Some(internal_pos) = window.map_virtual_to_internal_pos(virtual_mouse_pos) {
                    optional_internal_pos.replace(internal_pos);
                    optional_window_id.replace(window_id.clone());
                    break;
                }
            }
            if let Some(internal_pos) = optional_internal_pos {
                if let Some(window_id) = optional_window_id {
                    self.captured().replace(window_id.clone());
                    self.mouse_mut().set_internal_position(internal_pos);
                    self.windows_mut().get_mut(&window_id).unwrap().focus();
                } else {
                    self.mouse_mut().draw(sk);
                }
            } else {
                self.mouse_mut().draw(sk);
            }
        }
        for invalid_window_id in invalid_windows {
            self.windows_mut().remove(&invalid_window_id);
        }
        for window in self.windows_mut().values_mut() {
            window.draw(sk);
        }
    }
}

pub trait Mouse {
    fn virtual_move(&mut self, change: Vec3);
    fn virtual_position(&self) -> Vec3;
    fn set_virtual_position(&mut self, virtual_position: Vec3);

    fn internal_position(&self) -> InternalPos;
    fn set_internal_position(&mut self, internal_position: InternalPos);

    fn draw(&mut self, sk: &StereoKitDraw);
}

pub trait Window {
    /// top left corner
    fn virtual_position(&self) -> Vec3;
    fn virtual_size(&self) -> Vec2;
    fn virtual_quat(&self) -> Quat;

    fn set_virtual_position(&mut self, virtual_position: Vec3);
    fn set_virtual_size(&mut self, virtual_size: Vec2);
    fn set_virtual_quat(&self, virtual_quat: Quat);

    fn internal_position(&self) -> InternalPos;
    fn internal_size(&self) -> Vector2<u32>;

    fn set_internal_position(&mut self, internal_position: InternalPos);
    fn set_internal_size(&mut self, internal_size: Vector2<u32>);

    /// if the virtual pos is within the bounds of the virtual window, then it has the equivalent position it should be on the internal window.
    /// if this exists, then calling `mouse.set_internal_position` on it should put it exactly where it should be.
    fn map_virtual_to_internal_pos(&self, virtual_pos: Vec3) -> Option<InternalPos>;
    /// the point can be in or outside the virtual window, whatever the equivalent for the internal window would be.
    fn map_virtual_to_relative_pos(&self, virtual_pos: Vec3) -> RelativePos;

    fn map_relative_to_virtual_pos(&self, relative_pos: RelativePos) -> Vec3;


    fn is_valid(&self) -> bool;

    fn focus(&mut self);

    fn draw(&mut self, sk: &StereoKitDraw);
}

use color_eyre::Result;
use stereokit::color_named::WHITE;
use stereokit::pose::Pose;
use stereokit::render::{RenderLayer, StereoKitRender};
use stereokit::ui::{MoveType, WindowType};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::{GetClientRect, GetForegroundWindow, GetParent, GetWindowRect, GetWindowTextA, GetWindowTextW, GetWindowThreadProcessId, HWND_TOP, MoveWindow, SetForegroundWindow, SwitchToThisWindow};
use crate::MESH;

pub struct WindowsWindow {
    pub(crate) hwnd: HWND,
    win_material: Material,
    win_texture: Texture,
    win_mesh: Mesh,
    device: Device,
    capture: Capture,
}
impl WindowsWindow {
    pub(crate) fn new(sk: &impl StereoKitContext, hwnd: HWND) -> Result<Self> {
        let win_material = Material::copy_from_id(sk, DEFAULT_ID_MATERIAL_UNLIT)?;
        let win_texture = Texture::create(sk, TextureType::ImageNoMips, TextureFormat::None).unwrap();
        let win_mesh = unsafe { MESH.as_ref().unwrap().clone() };
        //let win_mesh = Mesh::find(sk, "default/mesh_sphere")?;
        win_texture.set_address_mode(stereokit::texture::TextureAddress::Clamp);
        win_material.set_texture(sk, "diffuse", &win_texture).unwrap();
        let device = Device::new_from_hwnd(unsafe { mem::transmute_copy(&hwnd.0)}).unwrap();
        let capture = Capture::new(&device).unwrap();
        Ok(Self {
            hwnd,
            win_material,
            win_texture,
            win_mesh,
            device,
            capture,
        })
    }
    pub(crate) fn draw(&self, sk: &StereoKitDraw, size: Vec2, mut pose: Pose) {
        for shared_tex in self.capture.rx.try_iter() {
            unsafe {
                self.win_texture.set_surface(std::mem::transmute(shared_tex), TextureType::ImageNoMips, 87, 0, 0, 1, true);
            }
        }
        stereokit::ui::window(sk, get_window_title(self.hwnd).as_str(), &mut pose, size.into(), WindowType::WindowNormal, MoveType::MoveNone, |ui| {
            unsafe { stereokit_sys::ui_layout_reserve(stereokit_sys::vec2 { x: size.x, y: size.y}, 0, 0.0) };
            sk.add_mesh(&self.win_mesh, &self.win_material, Mat4::from_scale_rotation_translation(Vec3::new(size.x, size.y, 1.0), Quat::IDENTITY, Vec3::new(0.0, -size.y/2.0, -0.001)).into(), WHITE, RenderLayer::LayerAll)
        });
    }
    pub fn size(&self) -> Option<Vector2<u32>> {
        let mut rect = RECT::default();
        unsafe {
            GetWindowRect(self.hwnd, &mut rect);
        }
        let mut w = rect.right - rect.left;
        let mut h = rect.bottom - rect.top;
        if w < 0 || h < 0 {
            return None;
        }
        Some(Vector2::from([w as u32, h as u32]))
    }
    pub fn set_size(&mut self, size: Vector2<u32>) -> Option<()> {
        if let Some(position) = self.position() {
            unsafe {
                MoveWindow(self.hwnd, position.x as i32, position.y as i32, size.x as i32, size.y as i32, true)
            };
            return Some(())
        }
        None
    }
    pub fn position(&self) -> Option<Vector2<u32>> {
        let mut rect = RECT::default();
        unsafe {
            GetWindowRect(self.hwnd, &mut rect);
        }
        if rect.left < 0 && rect.left > -10 {
            rect.left = 0;
        }
        if rect.left < 0 || rect.top < 0 {
            return None;
        }
        Some(Vector2::from([rect.left as u32, rect.top as u32]))
    }
    pub fn set_position(&mut self, position: Vector2<u32>) -> Option<()> {
        if let Some(size) = self.size() {
            unsafe {
                MoveWindow(self.hwnd, position.x as i32, position.y as i32, size.x as i32, size.y as i32, true)
            };
            return Some(())
        }
        return None;
    }
    pub fn bring_to_top(&mut self) {
        let position = self.position().unwrap();
        let size = self.size().unwrap();
        unsafe {
            for window in enumerate_windows() {
                if window.class_name.contains("sk") {
                    SetFocus(HWND({window.handle as isize}));
                }
            }
            windows::Win32::UI::WindowsAndMessaging::BringWindowToTop(self.hwnd);
            SetForegroundWindow(self.hwnd);
            SetFocus(self.hwnd);
            //SwitchToThisWindow(self.hwnd, BOOL::from(false));
            windows::Win32::UI::WindowsAndMessaging::SetWindowPos(self.hwnd, HWND_TOP, position.x as i32, position.y as i32, size.x as i32, size.y as i32, Default::default());
            windows::Win32::UI::WindowsAndMessaging::MoveWindow(self.hwnd, position.x as i32, position.y as i32, size.x as i32, size.y as i32, true);
        }
    }
    pub fn title(&self) -> String {
        get_window_title(self.hwnd)
    }
}

fn get_window_title(hwnd: HWND) -> String {
    let mut bytes = [0; 256];
    unsafe {
        GetWindowTextW(hwnd, &mut bytes);
    }
    let title = String::from_utf16(&bytes).unwrap();
    title
}


#[derive(Copy, Clone)]
pub struct RelativePos(pub Vector2<i32>);
#[derive(Copy, Clone)]
pub struct InternalPos(pub Vector2<u32>);
