use std::collections::HashMap;
use std::f32::consts::PI;
use std::ops::AddAssign;
use glam::{Mat4, Quat, Vec2, Vec3};
use mint::Vector2;
use stereokit::lifecycle::{StereoKitContext, StereoKitDraw};
use stereokit::mesh::{Ind, Mesh, Vertex};
use stereokit::model::Model;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetWindowRect, IsIconic, MoveWindow, SetCursorPos};
use color_eyre::Result;
use dxcapture::enumerate_windows;
use dxcapture::window_finder::WindowInfo;
use glam::EulerRot::XYZ;
use stereokit::color_named::{GREEN, NAVY, RED, WHITE};
use stereokit::input::{ButtonState, Key, StereoKitInput};
use stereokit::material::{DEFAULT_ID_MATERIAL, Material};
use stereokit::pose::Pose;
use stereokit::render::RenderLayer;
use stereokit::values::Color32;
use windows::s;
use crate::MESH;
use crate::window_management_2::{InternalPos, Mouse, RelativePos, Window, WindowManager, WindowsWindow};

pub struct SimpleMouse {
    position: Vec3,
    model: Model,
    mesh: Mesh,
    material: Material,
    sensitivity: f32,
    previous_position: InternalPos,
}

impl SimpleMouse {
    pub fn new(sk: &impl StereoKitContext, offset: Vec3) -> Result<Self> {
        //let model = Model::from_file(sk, "assets/mouse.glb", None)?;
        let mesh = Mesh::gen_cube(sk, Vec3::new(0.01, 0.1, 0.1), 1)?;
        let material = Material::copy_from_id(sk, DEFAULT_ID_MATERIAL)?;
        let model = Model::from_mesh(sk, &mesh, &material)?;
        let mut this = Self {
            position: offset,
            model,
            mesh,
            material,
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
        self.position = virtual_position;
        self.previous_position = self.internal_position();
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
        self.previous_position = internal_position;
    }

    fn draw(&mut self, sk: &StereoKitDraw) {
        let dx = self.internal_position().0.x as i32 - self.previous_position.0.x as i32;
        let dy = self.internal_position().0.y as i32 - self.previous_position.0.y as i32;

        self.virtual_move(Vec3::new(dx as f32 / self.sensitivity, -dy as f32 / self.sensitivity, 0.0));

        self.previous_position = self.internal_position();
        self.model.draw(sk,
                        Mat4::from_scale_rotation_translation(
                            Vec3::new(1.0, 1.0, 1.0),
                            Quat::from_euler(XYZ, 0.0, 90_f32.to_radians(), 0.0),
                            self.position - Vec3::new(-0.0, 0.0, 0.0),
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
        let virtual_size = match windows_window.size() {
            None => Vec2::new(0.5, 0.5),
            Some(size) => Vec2::new(size.x as f32 / 2000.0, size.y as f32 / 2000.0)
        };
        Ok(
            Self {
                virtual_pos: offset_pos,
                virtual_size,
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
        InternalPos(self.windows_window.position().unwrap())
    }

    fn internal_size(&self) -> Vector2<u32> {
        self.windows_window.size().unwrap()
    }

    fn set_internal_position(&mut self, internal_position: InternalPos) {
        self.windows_window.set_position(internal_position.0).unwrap()
    }

    fn set_internal_size(&mut self, internal_size: Vector2<u32>) {
        self.windows_window.set_size(internal_size);
        self.recapture = true;
    }

    fn map_virtual_to_internal_pos(&self, virtual_pos: Vec3) -> Option<InternalPos> {
        let test_window = TestWindow {
            virtual_position: self.virtual_pos,
            virtual_size: self.virtual_size,
            internal_size: self.internal_size(),
            internal_position: self.internal_position().0,
        };
        if let Some(pos) = test_window.virtual_to_internal(virtual_pos) {
            return Some(InternalPos(pos))
        }
        None
    }

    fn map_virtual_to_relative_pos(&self, virtual_pos: Vec3) -> RelativePos {
        todo!()
    }

    fn map_relative_to_virtual_pos(&self, relative_pos: RelativePos) -> Vec3 {
        let test_window = TestWindow {
            virtual_position: self.virtual_pos,
            virtual_size: self.virtual_size,
            internal_size: self.internal_size(),
            internal_position: self.internal_position().0,
        };
        let pos = Vector2::from([relative_pos.0.x as u32, relative_pos.0.y as u32]);
        test_window.internal_to_virtual(pos)
    }

    fn is_valid(&self) -> bool {
        if self.windows_window.size().is_none() {
            println!("size is invalid");
        }
        if self.windows_window.position().is_none() {
            println!("position is invalid");
        }
        if self.windows_window.size().is_none() || self.windows_window.position().is_none() {
            return false;
        }
        if unsafe {IsIconic(self.windows_window.hwnd).as_bool()} {
            println!("iconic");
            return false;
        }
        return true;
    }

    fn focus(&mut self) {
        self.windows_window.bring_to_top();
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


pub struct TestWindow {
    virtual_position: Vec3,
    virtual_size: Vec2,
    internal_size: Vector2<u32>,
    internal_position: Vector2<u32>,
}
impl TestWindow {
    pub fn virtual_to_internal(&self, mut mouse_pos: Vec3) -> Option<Vector2<u32>> {
        mouse_pos.x = mouse_pos.x - self.virtual_position.x;
        mouse_pos.y = mouse_pos.y - self.virtual_position.y;

        mouse_pos.x /= self.virtual_size.x;
        mouse_pos.y /= self.virtual_size.y;

        mouse_pos.y *= -1.0;

        if mouse_pos.x < 0.0 || mouse_pos.x > 1.0 || mouse_pos.y < 0.0 || mouse_pos.y > 1.0 {
            //panic!("{:?}", mouse_pos);
            return None;
        }

        mouse_pos.x *= self.internal_size.x as f32;
        mouse_pos.y *= self.internal_size.y as f32;
        Some(Vector2::from([mouse_pos.x as u32 + self.internal_position.x, mouse_pos.y as u32 + self.internal_position.y]))
    }
    pub fn internal_to_virtual(&self, mut mouse_pos: Vector2<u32>) -> Vec3 {
        let mut mouse_pos = Vector2::from([mouse_pos.x as i32, mouse_pos.y as i32]);
        mouse_pos.x = mouse_pos.x - self.internal_position.x as i32;
        mouse_pos.y = mouse_pos.y - self.internal_position.y as i32;

        let mut mouse_pos = Vec3::new(mouse_pos.x as f32, mouse_pos.y as f32, self.virtual_position.z);

        mouse_pos.x /= self.internal_size.x as f32;
        mouse_pos.y /= self.internal_size.y as f32;

        mouse_pos.y *= -1.0;

        mouse_pos.x *= self.virtual_size.x;
        mouse_pos.y *= self.virtual_size.y;

        mouse_pos.x = self.virtual_position.x + mouse_pos.x;
        mouse_pos.y = self.virtual_position.y + mouse_pos.y;

        mouse_pos
    }
}
#[test]
pub fn test_things() {
    let test_window = TestWindow {
        virtual_position: Vec3::new(0.0, 0.0, 0.0),
        virtual_size: Vec2::new(1.0, 1.0),
        internal_size: Vector2::from([500, 500]),
        internal_position: Vector2::from([0, 0]),
    };
    let mouse_pos = Vec3::new(0.1, -0.1, 0.0);
    let pos = test_window.virtual_to_internal(mouse_pos).unwrap();
    assert_eq!(pos, Vector2::from([50, 50]));
    let pos = test_window.internal_to_virtual(pos);
    assert_eq!(mouse_pos, pos);
    let test_window = TestWindow {
        virtual_position: Vec3::new(0.0, 0.0, 0.0),
        virtual_size: Vec2::new(1.0, 1.0),
        internal_size: Vector2::from([500, 500]),
        internal_position: Vector2::from([50, 0]),
    };
    let mouse_pos = Vec3::new(0.1, -0.1, 0.0);
    let pos = test_window.virtual_to_internal(mouse_pos).unwrap();
    assert_eq!(pos, Vector2::from([100, 50]));
    let pos = test_window.internal_to_virtual(pos);
    assert_eq!(mouse_pos, pos);
    let test_window = TestWindow {
        virtual_position: Vec3::new(0.0, 0.1, 0.0),
        virtual_size: Vec2::new(1.0, 1.0),
        internal_size: Vector2::from([500, 500]),
        internal_position: Vector2::from([50, 0]),
    };
    let mouse_pos = Vec3::new(0.1, -0.1, 0.0);
    let pos = test_window.virtual_to_internal(mouse_pos).unwrap();
    assert_eq!(pos, Vector2::from([100, 100]));
    let pos = test_window.internal_to_virtual(pos);
    assert_eq!(mouse_pos, pos);
}

#[test]
fn second_test() {
    let test_window = TestWindow {
        virtual_position: Vec3::new(0.0, 0.1, 0.0),
        virtual_size: Vec2::new(0.3, 2.5),
        internal_size: Vector2::from([500, 500]),
        internal_position: Vector2::from([50, 0]),
    };
    let mouse_pos = Vec3::new(0.1, -0.1, 0.0);
    let pos = test_window.virtual_to_internal(mouse_pos).unwrap();
    assert_eq!(pos, Vector2::from([216, 40]));
    let pos = test_window.internal_to_virtual(pos);
    assert_eq!(mouse_pos, pos);
}

#[test]
pub fn test_inverse() {
    let test_window = TestWindow {
        virtual_position: Vec3::new(0.0, 0.1, 0.0),
        virtual_size: Vec2::new(1.0, 1.0),
        internal_size: Vector2::from([500, 500]),
        internal_position: Vector2::from([50, 0]),
    };
    let mouse_pos = Vec3::new(0.1, -0.1, 0.0);
    let pos = test_window.virtual_to_internal(mouse_pos).unwrap();
    assert_eq!(pos, Vector2::from([100, 100]));
    let pos = test_window.internal_to_virtual(pos);
    assert_eq!(mouse_pos, pos);
}

pub fn main() {
    let sk = stereokit::Settings::default().flatscreen_pos_x(1500 as u32).log_filter(stereokit::lifecycle::LogFilter::Diagnostic).disable_unfocused_sleep(true).init().expect("Couldn't init stereokit");
    let mesh = Mesh::create(&sk).unwrap();

    let mut verts = vec![];
    let mut indices = vec![];

    let max = 36;

    let depth = 0.3;
    let curvature = 0.5;


    for x in 0..max {
        let mut val = ((x as f32 / max as f32) * curvature * PI + (PI * curvature / 2.0)).sin() * depth;
        val -= depth;
        if x % 2 != 0 {
            verts.push(
                Vertex {
                    pos: Vec3::new(x as f32 / max as f32, 1.0, val).into(),
                    norm: Vec3::new(0.0, 0.0, 1.0).into(),
                    uv: Vec2::new(1.0 - (x as f32 / max as f32), 0.0).into(),
                    col: Color32::from(WHITE),
                }
            );
            if x >= 2 {
                indices.push(x);
                indices.push(x - 1);
                indices.push(x - 2);
            }
        } else {
            verts.push(
                Vertex {
                    pos: Vec3::new(x as f32 / max as f32, 0.0, val).into(),
                    norm: Vec3::new(0.0, 0.0, 1.0).into(),
                    uv: Vec2::new(1.0 - (x as f32 / max as f32), 1.0).into(),
                    col: Color32::from(WHITE),
                }
            );
            if x >= 2 {
                indices.push(x - 2);
                indices.push(x - 1);
                indices.push(x);
            }
        }
    }


    let indices: Vec<_> = indices.iter().map(|a| Ind::new(*a)).collect();
    mesh.set_verts(&sk, verts.as_slice(), false);
    mesh.set_indices(&sk, indices.as_slice());

    unsafe {
        MESH.replace(mesh);
    }


    let mut simple_window_manager = SimpleWm::new(&sk);
    let mut offset_pos = Vec3::new(-1.5, 0.0, -2.0);
    for window_info in enumerate_windows() {
        let hwnd = unsafe {HWND(window_info.handle as isize)};
        let mut window = SimpleWindow::new(&sk, hwnd.clone(), offset_pos).unwrap();
        if window_info.title.contains("sk")
        || window_info.title.contains("steam")
        || window_info.title.contains("Steam") {
            continue;
        }
        if !window.is_valid() {
            println!("invalid window: {}", window_info.title);
            continue;
        }
        if offset_pos.x >= 3.0 {
            offset_pos.x = -1.5;
            offset_pos.y += 0.8;
        }
        if window.is_valid() {
            window.set_internal_position(InternalPos(Vector2::from([20, 20])));
            window.set_internal_size(Vector2::from([1280, 720]));
            window.focus();
            offset_pos.x += 1.0;
            simple_window_manager.windows.insert(hwnd.0, window);
        }
    }
    sk.run(|sk| {
        simple_window_manager.draw(sk);
        if simple_window_manager.captured_window.is_none() {
            //simple_window_manager.mouse_mut().set_internal_position(InternalPos(Vector2::from([10, 10])));
        }
        if sk.input_key(Key::KeyQ).contains(ButtonState::Active) {
            if sk.input_key(Key::Ctrl).contains(ButtonState::Active) {
                sk.quit();
            }
        }
    }, |_|{});

}