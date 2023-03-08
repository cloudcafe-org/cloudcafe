use std::collections::HashMap;
use std::f32::consts::PI;
use std::ops::AddAssign;
use glam::{Mat4, Quat, Vec2, Vec3};
use mint::Vector2;
use stereokit::lifecycle::{StereoKitContext, StereoKitDraw};
use stereokit::mesh::Mesh;
use stereokit::model::Model;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetWindowRect, IsIconic, MoveWindow, SetCursorPos};
use color_eyre::Result;
use dxcapture::enumerate_windows;
use dxcapture::window_finder::WindowInfo;
use glam::EulerRot::XYZ;
use stereokit::color_named::NAVY;
use stereokit::pose::Pose;
use stereokit::render::RenderLayer;
use windows::s;
use crate::window_management_2::{InternalPos, Mouse, RelativePos, Window, WindowManager, WindowsWindow};

pub struct SimpleMouse {
    position: Vec3,
    model: Model,
    sensitivity: f32,
    previous_position: InternalPos,
}

impl SimpleMouse {
    pub fn new(sk: &impl StereoKitContext, offset: Vec3) -> Result<Self> {
        let model = Model::from_file(sk, "assets/mouse.glb", None)?;
        let mut this = Self {
            position: offset,
            model,
            sensitivity: 100.0,
            previous_position: InternalPos(Vector2::from([0, 0])),
        };
        let prev_pos = this.internal_position();
        this.previous_position = prev_pos;
        Ok(this)
    }
}

impl Mouse for SimpleMouse {
    fn virtual_move(&mut self, change: Vec3) {
        self.position.add_assign(change);
    }

    fn virtual_position(&self) -> Vec3 {
        self.position
    }

    fn set_virtual_position(&mut self, virtual_position: Vec3) {
        self.position = virtual_position
    }

    fn internal_position(&self) -> InternalPos {
        let mut cursor_pos = POINT::default();
        unsafe {
            GetCursorPos(&mut cursor_pos)
        };
        InternalPos(Vector2::from([cursor_pos.x as u32, cursor_pos.y as u32]))
    }

    fn set_internal_position(&mut self, internal_position: InternalPos) {
        unsafe {
            SetCursorPos(internal_position.0.x as i32, internal_position.0.y as i32);
        }
    }

    fn draw(&mut self, sk: &StereoKitDraw) {
        let dx = self.internal_position().0.x as i32 - self.previous_position.0.x as i32;
        let dy = self.internal_position().0.y as i32 - self.previous_position.0.y as i32;

        self.virtual_move(Vec3::new(dx as f32 / self.sensitivity, -dy as f32 / self.sensitivity, 0.0));

        self.previous_position = self.internal_position();
        self.model.draw(sk,
                        Mat4::from_scale_rotation_translation(
                            Vec3::new(0.4, 0.4, 0.4),
                            Quat::from_euler(XYZ, 0.0, 90_f32.to_radians(), 0.0),
                            self.position - Vec3::new(-0.88, 0.88, 0.0),
                        ).into(), NAVY, RenderLayer::Layer1);
    }
}

pub struct SimpleWm {
    mouse: SimpleMouse,
    windows: HashMap<isize, SimpleWindow>,
    captured_window: Option<isize>,
    offset: Vec3,
}

impl SimpleWm {
    pub fn new(sk: &impl StereoKitContext) -> Self {
        let offset = Vec3::new(1.5, 1.5, -2.0);
        let mouse = SimpleMouse::new(sk, offset).unwrap();
        Self {
            mouse,
            windows: Default::default(),
            captured_window: None,
            offset,
        }
    }
}

impl WindowManager<SimpleMouse, SimpleWindow, isize> for SimpleWm {
    fn mouse_ref(&self) -> &SimpleMouse {
        &self.mouse
    }

    fn mouse_mut(&mut self) -> &mut SimpleMouse {
        &mut self.mouse
    }

    fn windows_ref(&self) -> &HashMap<isize, SimpleWindow> {
        &self.windows
    }

    fn windows_mut(&mut self) -> &mut HashMap<isize, SimpleWindow> {
        &mut self.windows
    }

    fn captured(&mut self) -> &mut Option<isize> {
        &mut self.captured_window
    }
}

pub struct SimpleWindow {
    virtual_pos: Vec3,
    virtual_size: Vec2,
    virtual_quat: Quat,
    windows_window: WindowsWindow,
    recapture: bool,
}

impl SimpleWindow {
    pub fn new(sk: &impl StereoKitContext, hwnd: HWND, offset_pos: Vec3) -> Result<Self> {
        let windows_window = WindowsWindow::new(sk, hwnd)?;
        Ok(
            Self {
                virtual_pos: offset_pos,
                virtual_size: Vec2::new(0.5, 0.5),
                virtual_quat: Quat::from_euler(XYZ, 0.0, PI, 0.0),
                windows_window,
                recapture: false,
            }
        )
    }
}

impl Window for SimpleWindow {
    fn virtual_position(&self) -> Vec3 {
        self.virtual_pos
    }

    fn virtual_size(&self) -> Vec2 {
        self.virtual_size
    }

    fn virtual_quat(&self) -> Quat {
        Quat::IDENTITY
    }

    fn set_virtual_position(&mut self, virtual_position: Vec3) {
        self.virtual_pos = virtual_position;
    }

    fn set_virtual_size(&mut self, virtual_size: Vec2) {
        self.virtual_size = virtual_size;
    }

    fn set_virtual_quat(&self, virtual_quat: Quat) {
        unimplemented!()
    }

    fn internal_position(&self) -> InternalPos {
        InternalPos(self.windows_window.position())
    }

    fn internal_size(&self) -> Vector2<u32> {
        self.windows_window.size()
    }

    fn set_internal_position(&mut self, internal_position: InternalPos) {
        self.windows_window.set_position(internal_position.0)
    }

    fn set_internal_size(&mut self, internal_size: Vector2<u32>) {
        self.windows_window.set_size(internal_size);
        self.recapture = true;
    }

    fn map_virtual_to_internal_pos(&self, virtual_pos: Vec3) -> Option<InternalPos> {
        let mut pos = self.virtual_pos;
        pos.x = pos.x - virtual_pos.x;
        pos.y -= virtual_pos.y;


        pos.x /= self.virtual_size.x;
        pos.y /= self.virtual_size.y;

        pos.x = 1.0 - pos.x;

        //println!("pos: {pos}");

        pos.x *= self.internal_size().x as f32;
        pos.y *= self.internal_size().y as f32;
        let ret_pos = Vector2::from([pos.x as i32, pos.y as i32]);
        if ret_pos.x < 0 || ret_pos.x > self.internal_size().x as i32 || ret_pos.y < 0 || ret_pos.y > self.internal_size().y as i32 {
            return None;
        }
        //println!("virtual: {}, mapped: {:?}", virtual_pos, ret_pos);
        Some(InternalPos(Vector2::from([self.internal_position().0.x + ret_pos.x as u32, self.internal_position().0.y + ret_pos.y as u32])))
    }

    fn map_virtual_to_relative_pos(&self, virtual_pos: Vec3) -> RelativePos {
        todo!()
    }

    fn map_relative_to_virtual_pos(&self, relative_pos: RelativePos) -> Vec3 {
        let mut pos = Vector2::from([(relative_pos.0.x - self.internal_position().0.x as i32) as f32, (relative_pos.0.y - self.internal_position().0.y as i32)as f32]);
        println!("new_pos: {:?}", pos);
        pos.x /= self.internal_size().x as f32;
        pos.y /= self.internal_size().y as f32;

        pos.x = 1.0 - pos.x;

        pos.x *= self.virtual_size.x;
        pos.y *= self.virtual_size.y;

        pos.x = self.virtual_pos.x + pos.x;
        pos.y = self.virtual_pos.y + pos.y;
        Vec3::new(pos.x, pos.y, self.virtual_pos.z)
    }

    fn is_valid(&self) -> bool {
        if self.internal_size().x == 0
            || self.internal_size().y == 0
        {
            return false;
        }
        if unsafe {IsIconic(self.windows_window.hwnd).as_bool()} {
            return false;
        }
        return true;
    }

    fn focus(&mut self) {
        todo!()
    }

    fn draw(&mut self, sk: &StereoKitDraw) {
        if self.recapture {
            self.recapture = false;
            self.windows_window = WindowsWindow::new(sk, self.windows_window.hwnd).unwrap();
        }
        let mut pos = self.virtual_pos;
        pos.x += self.virtual_size.x / 2.0;
        self.windows_window.draw(sk, self.virtual_size, Pose::new(pos, self.virtual_quat));
    }
}

pub fn main() {
    let sk = stereokit::Settings::default().log_filter(stereokit::lifecycle::LogFilter::Diagnostic).disable_unfocused_sleep(true).init().expect("Couldn't init stereokit");
    //let t = Texture::from_cubemap_equirectangular(&sk, "assets/skytex2.hdr", true, 0).unwrap();
    //sk.set_skytex(&t.0);
    //sk.set_skylight(&t.1);
    let mut simple_window_manager = SimpleWm::new(&sk);
    let mut offset_pos = Vec3::new(-1.5, 0.0, -2.0);
    for window in enumerate_windows() {
        if simple_window_manager.windows.len() == 2 {
            break;
        }
        let hwnd = unsafe {HWND(window.handle as isize)};
        let window = SimpleWindow::new(&sk, hwnd.clone(), offset_pos).unwrap();
        if window.is_valid() {
            offset_pos.x += 0.7;
            simple_window_manager.windows.insert(hwnd.0, window);
        }
    }
    let mut first_run = true;
    sk.run(|sk| {
        if first_run {
            first_run = false;
            //let matrix = Mat4::from_scale_rotation_translation(Vec3::new(1.0, 1.0, 1.0), Quat::from_euler(glam::EulerRot::XYZ, 0.0_f32.to_radians(), PI, 0.0_f32.to_radians()), Vec3::default());
            //stereokit::render::Camera::set_root(sk, matrix);
        }
        simple_window_manager.draw(sk);
    }, |_|{});

}