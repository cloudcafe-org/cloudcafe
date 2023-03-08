use std::collections::HashMap;
use std::f32::consts::PI;
use std::hash::Hash;
use std::mem;
use std::mem::transmute;
use dxcapture::{Capture, Device, enumerate_windows};
use dxcapture::window_finder::WindowInfo;
use mint::Vector2;
use stereokit::lifecycle::{LogFilter, StereoKitContext, StereoKitDraw};
use stereokit::material::{DEFAULT_ID_MATERIAL_UNLIT, Material};
use stereokit::mesh::Mesh;
use stereokit::texture::{Texture, TextureFormat, TextureType};
use windows::Win32::Foundation::{HWND, POINT};
use color_eyre::Result;
use glam::{EulerRot, Mat4, Quat, Vec2, Vec3};
use glam::EulerRot::XYZ;
use stereokit::color_named::{BLACK, WHITE};
use stereokit::input::{ButtonState, Key, StereoKitInput};
use stereokit::model::Model;
use stereokit::pose::Pose;
use stereokit::render::{Camera, RenderLayer, StereoKitRender};
use stereokit::Settings;
use stereokit::ui::{MoveType, WindowType};
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::{BringWindowToTop, GetCursorPos, HWND_TOP, HWND_TOPMOST, MoveWindow, SetCursorPos, SetWindowPos};
use crate::Window;

pub trait WindowManager<Window: TWindow, WindowId: Clone + Hash + std::cmp::Eq, Mouse: TMouse> {

    /// where the user is placed
    fn set_user_center(&mut self, user_center: Vec3);
    /// distance from the user, for a sphere for example this would be a radius
    fn set_offset(&mut self, offset: f32);

    fn windows_ref(&self) -> &HashMap<WindowId, Window>;
    /// do not add or remove windows using this, just use this to get mutable access to the windows themselves
    /// instead use the `remove_window` and `add_window` functions.
    fn windows_mut(&mut self) -> &mut HashMap<WindowId, Window>;

    fn get_mouse(&self) -> &Mouse;
    fn get_mouse_mut(&mut self) -> &mut Mouse;


    fn mouse_move(&mut self, dx: i32, dy: i32);
    fn set_focus(&mut self, window_id: &WindowId);
    fn resize_window(&mut self, window_id: &WindowId, size: mint::Vector2<u32>);
    fn remove_window(&mut self, window_id: &WindowId);
    fn add_window(&mut self, window: Window, window_id: &WindowId);
    /// is the mouse where whatever titlebar you have, such that it can be grabbed and dragged around.
    fn within_grabbable_range(&mut self, window_id: &WindowId) -> bool;
    /// has the mouse entered the bounds of the actual window content.
    fn mouse_in_window(&self, window_id: &WindowId) -> bool;
    /// gets the equivalent x and y the virtual mouse would be in mapped to the real window
    fn mouse_window_pos_mapping(&self, window_id: &WindowId) -> Option<mint::Vector2<u32>>;

    fn get_grabbed_window_id(&self) -> Option<WindowId>;

    fn set_grabbed(&mut self, window_id: &WindowId);
    fn unset_grabbed(&mut self);

    fn draw(&mut self, sk: &StereoKitDraw) {
        if sk.input_key(Key::MouseLeft).contains(ButtonState::JustActive) {
            /*
           if self.get_grabbed_window_id().is_none() {
               let mut optional_window_id = None;
               for window_id in self.get_window_ids() {
                   if self.mouse_in_window(window_id) {
                       optional_window_id.replace(window_id.clone());
                       break;
                   }
               }
               if let Some(window_id) = optional_window_id {
                   self.set_grabbed(&window_id);
               }
           }
             */
        }
        if sk.input_key(Key::MouseLeft).contains(ButtonState::JustInactive) {
            /*
            if self.get_grabbed_window_id().is_some() {
                self.unset_grabbed();
            }
             */
        }
        {
            let calc_mouse_pos = self.get_mouse().get_calc_mouse_position();
            /*
            if let Some(grabbed_window_id) = self.get_grabbed_window_id() {
                let grabbed_window = self.windows_mut().get_mut(grabbed_window_id).unwrap();
                grabbed_window.set_calc_position(calc_mouse_pos);
            }
             */
        }
        for window in self.windows_ref().values() {
            window.draw(sk);
        }
        for id in self.windows_ref().keys() {
            if self.mouse_in_window(id) {
                if let Some(pos) = self.mouse_window_pos_mapping(id) {
                    //let window = self.windows_ref().get(id).unwrap();
                    //println!("{}, pos: {:?}", window.get_title(), pos);
                }
                let window = self.windows_ref().get(id).unwrap();
                self.set_focus(&id.clone());
                break;
                //println!("{} is within mouse", window.get_title());
            }
        }
    }
}

pub trait TMouse {
    fn mouse_move(&mut self, dx: i32, dy: i32);
    fn get_draw_mouse_position(&self) -> Vec3;
    fn get_draw_mouse_rotation(&self) -> Quat;
    fn get_calc_mouse_position(&self) -> Vec3;
    fn get_calc_mouse_rotation(&self) -> Quat;
    fn get_mouse_model(&self) -> &stereokit::model::Model;
    fn get_mouse_scale(&self) -> Vec3;

    fn draw(&self, sk: &StereoKitDraw) {
        let mouse_model = self.get_mouse_model();
        let mouse_matrix = Mat4::from_scale_rotation_translation(
            self.get_mouse_scale(),
            self.get_draw_mouse_rotation(),
            self.get_draw_mouse_position()
        );
        mouse_model.draw(sk, mouse_matrix.into(), BLACK, RenderLayer::Layer1);
    }
}

pub trait TWindow {
    fn get_internal_size(&self) -> mint::Vector2<u32> {
        let dimensions = self.get_window_texture().get_dimensions();
        Vector2::from([dimensions.0 as u32, dimensions.1 as u32])
    }
    fn set_internal_size(&mut self, size: mint::Vector2<u32>);
    /// focuses this window
    fn set_focused(&mut self);
    fn get_window_mesh(&self) -> &stereokit::mesh::Mesh;
    fn get_window_material(&self) -> &stereokit::material::Material;
    fn get_window_texture(&self) -> &stereokit::texture::Texture;
    fn get_title(&self) -> &str;


    fn set_calc_position(&mut self, position: Vec3);

    fn get_draw_rotation(&self) -> Quat;
    fn get_draw_position(&self) -> Vec3;
    fn get_draw_size(&self) -> Vec2;

    fn get_calc_position(&self) -> Vec3;

    fn setup_init(&mut self, sk: &impl StereoKitContext) {
        let win_mat = self.get_window_material();
        let win_tex = self.get_window_texture();
        win_tex.set_address_mode(stereokit::texture::TextureAddress::Clamp);
        win_mat.set_texture(sk, "diffuse", &win_tex).unwrap();
    }
    /// gets called inside the draw call before any drawing occurs
    fn draw_hook(&self);
    fn draw(&self, sk: &StereoKitDraw) {
        self.draw_hook();
        let rotation = self.get_draw_rotation();
        let position = self.get_draw_position();
        let size = self.get_draw_size();
        let mut pose = Pose::new(position, rotation);
        stereokit::ui::window(sk, self.get_title(), &mut pose, size.into(), WindowType::WindowNormal, MoveType::MoveNone, |ui| {
            unsafe { stereokit_sys::ui_layout_reserve(stereokit_sys::vec2 { x: size.x, y: size.y}, 0, 0.0) };
            sk.add_mesh(&self.get_window_mesh(), &self.get_window_material(), Mat4::from_scale_rotation_translation(Vec3::new(size.x, size.y, 1.0), Quat::IDENTITY, Vec3::new(0.0, -size.y/2.0, -0.001)).into(), WHITE, RenderLayer::LayerAll)
        });
    }
}

pub struct SimpleWindowManager {
    windows: HashMap<isize, SimpleWindow>,
    mouse: SimpleMouse,
    user_center: Vec3,
    offset: f32,
}
pub struct SimpleMouse {
    mouse_model: Model,
    sensitivity: f32,
    position: Vec3,
}
impl SimpleMouse {
    pub fn new(sk: &impl StereoKitContext, startup_offset: Vec3) -> Result<Self> {
        let mouse_model = Model::from_file(sk, "assets/mouse.glb", None)?;
        Ok(Self {
            mouse_model,
            sensitivity: 100.0,
            position: startup_offset,
        })
    }
}
impl SimpleWindowManager {
    pub fn new(sk: &impl StereoKitContext) -> Result<Self> {
        let offset = 2.0;
        let startup_offset = Vec3::new(1.5, -3.0, offset);
        let mouse = SimpleMouse::new(sk, startup_offset)?;
        Ok(
            Self {
                windows:HashMap::default(),
                mouse,
                user_center: Vec3::default(),
                offset,
            }
        )
    }
}
impl WindowManager<SimpleWindow, isize, SimpleMouse> for SimpleWindowManager {
    fn set_user_center(&mut self, user_center: Vec3) {
        self.user_center = user_center;
    }

    fn set_offset(&mut self, offset: f32) {
        self.offset = offset;
    }

    fn windows_ref(&self) -> &HashMap<isize, SimpleWindow> {
        &self.windows
    }

    fn windows_mut(&mut self) -> &mut HashMap<isize, SimpleWindow> {
        &mut self.windows
    }

    fn get_mouse(&self) -> &SimpleMouse {
        &self.mouse
    }

    fn get_mouse_mut(&mut self) -> &mut SimpleMouse {
        &mut self.mouse
    }

    fn mouse_move(&mut self, dx: i32, dy: i32) {
        self.mouse.mouse_move(dx, dy);
    }

    fn set_focus(&mut self, window_id: &isize) {
        self.windows.get_mut(window_id).unwrap().set_focused();
    }

    fn resize_window(&mut self, window_id: &isize, size: Vector2<u32>) {
        self.windows.get_mut(window_id).unwrap().set_internal_size(size);
    }

    fn remove_window(&mut self, window_id: &isize) {
        todo!()
    }

    fn add_window(&mut self, window: SimpleWindow, window_id: &isize) {
        self.windows.insert(window_id.clone(), window);
    }

    fn within_grabbable_range(&mut self, window_id: &isize) -> bool {
        todo!()
    }

    fn mouse_in_window(&self, window_id: &isize) -> bool {
        let mouse_position = self.mouse.position;
        let window = self.windows_ref().get(window_id).unwrap();
        let window_pos = window.get_calc_position();
        let window_size = window.get_draw_size();
        if window_pos.x - window_size.x / 2.0 < mouse_position.x && window_pos.y > mouse_position.y {
            if window_pos.x + window_size.x / 2.0 > mouse_position.x {
                if window_pos.y - window_size.y < mouse_position.y {
                    return true;
                }
            }
        }
        return false;
    }

    fn mouse_window_pos_mapping(&self, window_id: &isize) -> Option<Vector2<u32>> {
        let window = self.windows_ref().get(window_id)?;
        let window_pos = window.get_calc_position();
        let mouse_pos = self.get_mouse().get_calc_mouse_position();
        let window_size = window.get_draw_size();
        let diff_x = mouse_pos.x - (window_pos.x - window_size.x / 2.0);
        let diff_y = window_pos.y - mouse_pos.y;

        let diff_x = diff_x / window_size.x;
        let diff_y = diff_y / window_size.y;

        let diff_x = 1.0 - diff_x;

        let diff_x = (diff_x * window.get_internal_size().x as f32) as u32;
        let diff_y = (diff_y * window.get_internal_size().y as f32) as u32;
        Some(Vector2::from([diff_x, diff_y]))
    }

    fn get_grabbed_window_id(&self) -> Option<isize> {
        todo!()
    }

    fn set_grabbed(&mut self, window_id: &isize) {
        todo!()
    }

    fn unset_grabbed(&mut self) {
        todo!()
    }
}
impl TMouse for SimpleMouse {
    fn mouse_move(&mut self, dx: i32, dy: i32) {
        self.position.x -= dx as f32 / self.sensitivity;
        self.position.y += dy as f32 / self.sensitivity;
    }

    fn get_draw_mouse_position(&self) -> Vec3 {
        self.position
    }

    fn get_draw_mouse_rotation(&self) -> Quat {
        Quat::from_euler(EulerRot::XYZ, 0.0, 90.0_f32.to_radians() as f32, 0.0)
    }

    fn get_calc_mouse_position(&self) -> Vec3 {
        self.position
    }

    fn get_calc_mouse_rotation(&self) -> Quat {
        Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0)
    }

    fn get_mouse_model(&self) -> &Model {
        &self.mouse_model
    }

    fn get_mouse_scale(&self) -> Vec3 {
        Vec3::new(0.2, 0.2, 0.2)
    }
}
pub struct SimpleWindow {
    win_material: Material,
    win_texture: Texture,
    win_mesh: Mesh,
    device: Device,
    capture: Capture,
    window_info: WindowInfo,
    hwnd: HWND,
    position: Vec3,
}
impl SimpleWindow {
    pub fn new(sk: &impl StereoKitContext, window_info: WindowInfo, position: Vec3) -> Result<Self> {
        let win_material = Material::copy_from_id(sk, DEFAULT_ID_MATERIAL_UNLIT)?;
        let win_texture = Texture::create(sk, TextureType::ImageNoMips, TextureFormat::None).unwrap();
        let win_mesh = Mesh::find(sk, "default/mesh_quad")?;
        let hwnd = unsafe { HWND( window_info.handle as isize) };
        unsafe {
            MoveWindow(hwnd, 20, 20, 1920, 1080, true);
        }
        let device = Device::new_from_hwnd(unsafe { mem::transmute_copy(&hwnd) }).unwrap();
        let capture = Capture::new(&device).unwrap();
        let mut this =
        Self {
            win_material,
            win_texture,
            win_mesh,
            device,
            capture,
            hwnd,
            window_info,
            position,
        };
        this.setup_init(sk);
        Ok(this)
    }
}
impl TWindow for SimpleWindow {

    fn set_internal_size(&mut self, size: Vector2<u32>) {
        todo!()
    }

    fn set_focused(&mut self) {
        unsafe {
            MoveWindow(self.hwnd, 20, 20, 1920, 1080, true);
            BringWindowToTop(self.hwnd);
            SetWindowPos(self.hwnd, HWND_TOP, 20, 20, 1920, 1080, Default::default());
        }
    }

    fn get_window_mesh(&self) -> &Mesh {
        &self.win_mesh
    }

    fn get_window_material(&self) -> &Material {
        &self.win_material
    }

    fn get_window_texture(&self) -> &Texture {
        &self.win_texture
    }

    fn get_title(&self) -> &str {
        self.window_info.title.as_str()
    }

    fn set_calc_position(&mut self, position: Vec3) {
        self.position = position;
    }

    fn get_draw_rotation(&self) -> Quat {
        Quat::IDENTITY
    }

    fn get_draw_position(&self) -> Vec3 {
        self.position
    }

    fn get_draw_size(&self) -> Vec2 {
        Vec2::new(self.get_internal_size().x as f32 / 1000.0, self.get_internal_size().y as f32 / 1000.0)
    }

    fn get_calc_position(&self) -> Vec3 {
        self.position
    }

    fn draw_hook(&self) {
        for shared_tex in self.capture.rx.try_iter() {
            unsafe {
                self.win_texture.set_surface(std::mem::transmute(shared_tex), TextureType::ImageNoMips, 87, 0, 0, 1, true);
            }
        }
    }
}

pub struct WindowHandler {
    window_manager: SimpleWindowManager,
    previous_mouse_pos: POINT,
    mouse_captured: Option<isize>,
}

impl WindowHandler {
    fn new(sk: &impl StereoKitContext) -> Result<Self> {
        let window_manager = SimpleWindowManager::new(sk)?;
        let mut previous_mouse_pos = POINT::default();
        unsafe { GetCursorPos(&mut previous_mouse_pos)};
        Ok(Self {
            window_manager,
            previous_mouse_pos,
            mouse_captured: None,
        })
    }
    fn init(&mut self, sk: &impl StereoKitContext) {
        let mut setup_window_position = Vec3::new(-1.0, 0.0, self.window_manager.offset);
        for window_info in enumerate_windows() {
            if window_info.title.contains("sk") {
                continue;
            }
            let mut window = SimpleWindow::new(sk, window_info.clone(), setup_window_position).unwrap();
            setup_window_position.x += 2.0;
            if setup_window_position.x >= 4.0 {
                setup_window_position.x = 0.0;
                setup_window_position.y += 1.5;
            }
            //window.position.x -= window.get_draw_size().x;
            self.window_manager.add_window(window, &(window_info.handle as isize))
        }
    }
    fn draw(&mut self, sk: &StereoKitDraw) {
        let mut current_mouse_pose = POINT::default();
        unsafe { GetCursorPos(&mut current_mouse_pose)};
        if self.mouse_captured.is_some() {
            if current_mouse_pose.x < 20
                || current_mouse_pose.x > 1940
                || current_mouse_pose.y < 20
                || current_mouse_pose.y > 1120 {
                let window_id = self.mouse_captured.take().unwrap();
                let window = self.window_manager.windows.get(&window_id).unwrap();
                //window.get_draw_position().x * current_mouse_pose.x
            }
        }
        else {
            let dx = current_mouse_pose.x - self.previous_mouse_pos.x;
            let dy = current_mouse_pose.y - self.previous_mouse_pos.y;
            self.previous_mouse_pos = current_mouse_pose;
            self.window_manager.mouse_move(dx, dy);
            unsafe {
                //SetCursorPos(10, 10);
            }
            for window in self.window_manager.windows.keys() {
                if self.window_manager.mouse_in_window(window) {
                    self.mouse_captured = None;
                    let mut new_mouse_pos = self.window_manager.mouse_window_pos_mapping(window).unwrap();
                    new_mouse_pos.x += 20;
                    new_mouse_pos.y += 20;
                    unsafe {
                        SetCursorPos(new_mouse_pos.x as i32, new_mouse_pos.y as i32);
                    }
                }
            }
            self.window_manager.get_mouse().draw(sk);
        }
        self.window_manager.draw(sk);
    }
}

pub fn main() {
    let sk = Settings::default().log_filter(LogFilter::Diagnostic).disable_unfocused_sleep(true).init().expect("Couldn't init stereokit");
    let t = Texture::from_cubemap_equirectangular(&sk, "assets/skytex2.hdr", true, 0).unwrap();
    sk.set_skytex(&t.0);
    sk.set_skylight(&t.1);
    let mut window_handler = WindowHandler::new(&sk).unwrap();
    window_handler.init(&sk);
    let mut first_run = true;
    sk.run(|sk| {
        if first_run {
            first_run = false;
            let matrix = Mat4::from_scale_rotation_translation(Vec3::new(1.0, 1.0, 1.0), Quat::from_euler(EulerRot::XYZ, 0.0_f32.to_radians(),  PI, 0.0_f32.to_radians()), Vec3::default());
            Camera::set_root(sk, matrix);
        }
        window_handler.draw(sk);
    }, |_|{});

}