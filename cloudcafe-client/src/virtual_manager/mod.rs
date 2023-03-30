use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use dxcapture::{enumerate_displays, enumerate_windows};
use glam::{Mat4, Quat, Vec2, Vec3};
use stereokit::lifecycle::{StereoKitContext, StereoKitDraw};
use stereokit::pose::Pose;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::HMONITOR;
use crate::internal_os::FakeMonitor;
use crate::internal_os::internal_window::IWindow;
use crate::virtual_manager::virtual_window::{IsWindowValid, VWindow, WindowCapture};
use crate::windows_bindings::{class_name, get_real_window_rect, get_real_window_size, get_window_rect, Hwnd, is_window, Rect, window_enabled, window_visible};
use color_eyre::{Report, Result};
use color_eyre::owo_colors::OwoColorize;
use stereokit::material::Cull;
use crate::input::{Key, KeyboardMouseState};
use crate::input::Key::MouseLeft;
use crate::internal_os::internal_mouse::IMouse;
use crate::values::{cart_2_cyl, cyl_2_cart, IVec2, quat_lookat, UVec2};
use crate::virtual_manager::desktop_capture::CaptureDesktop;
use crate::virtual_manager::virtual_mouse::{CursorType, ResizeType, VMouse};

pub mod virtual_mouse;
pub mod virtual_window;
pub mod desktop_capture;

pub const INVALID_WINDOW_TITLES: [&'static str; 1] = ["Cloudcafe XR Desktop" /*"SteamVR",*/ /*"OBS",*/ /*"Mixed Reality Portal"*/];

pub struct VDesktop {
    capture_desktop: CaptureDesktop,
    windows: HashMap<isize, VWindow>,
    current_list_windows_thread: JoinHandle<()>,
    current_list_windows: Arc<Mutex<Vec<Hwnd>>>,
    fake_monitor: FakeMonitor,
    pub(crate) v_mouse: VMouse,
    grabbed_window: Option<(isize, Vec3)>,
    resize_window: Option<(isize, ResizeType, Vec3)>,
    pub(crate) captured_window: Option<isize>,
    skip_windows: Vec<isize>,
    console_hwnd: Hwnd,
    pub(crate) center: Vec3,
    radius: f32,
    tick_counter: u32,
    pub lock_cursor: bool,
}
fn is_invalid_window(window_title: &str) -> bool {
    for invalid_title in INVALID_WINDOW_TITLES {
        if window_title.contains(invalid_title) {
            return true;
        }
    }
    return false;
}
impl VDesktop {
    pub fn new(sk: &impl StereoKitContext, console_hwnd: Hwnd, radius: f32) -> Result<Self> {
        let mut fake_monitor = None;
            let displays = enumerate_displays();
            for display in displays {
                if (display.monitor_rect.right - display.monitor_rect.left) != 4000 {
                    continue;
                }
                fake_monitor.replace(FakeMonitor::new(HMONITOR(display.handle as isize),Rect {
                    left: display.monitor_rect.left,
                    top: display.monitor_rect.top,
                    right: display.monitor_rect.right,
                    bottom: display.monitor_rect.bottom,
                })?);
            }
        let fake_monitor = fake_monitor.ok_or(Report::msg("unable to locate monitor"))?;

        let capture_desktop = CaptureDesktop::new(sk, fake_monitor)?;

        let mut hwnds = Vec::new();
        for window_info in enumerate_windows() {
            if is_invalid_window(&window_info.title) {
                continue;
            }
            if window_info.handle as isize == console_hwnd.0 {
                continue;
            }
            hwnds.push(HWND(window_info.handle as isize));
        }
        let mut i_windows = Vec::new();
        for hwnd in hwnds {
            if let Ok(i_window) = IWindow::new(hwnd, fake_monitor) {
                if i_window.size().unwrap().x != 0 && i_window.size().unwrap().y != 0 {
                    i_windows.push(i_window);
                }
            }
        }
        let mut windows = HashMap::new();
        let mut z_depth = 0;
        for i_window in i_windows {
            if let Ok(mut v_window) = VWindow::new(sk, i_window.hwnd, fake_monitor, Pose::new([0.0, 0.0, -radius], Quat::IDENTITY), z_depth, capture_desktop.clone()) {
                v_window.internal_window.set_aspect_ratio(1.7);
                v_window.internal_window.move_to_inactive();
                windows.insert(i_window.hwnd.0, v_window);
                z_depth += 1;
            }
        }

        let current_list_windows = Arc::new(Mutex::new(Vec::new()));
        let c_l_w = current_list_windows.clone();
        let current_list_windows_thread = thread::spawn(move || {
            let c_l_w = c_l_w;
            loop {
                thread::sleep(Duration::from_millis(5));
                let mut windows = Vec::new();
                for window_info in enumerate_windows() {
                    let hwnd = HWND(window_info.handle as isize);
                    if is_invalid_window(&window_info.title) {
                        continue;
                    }
                    if window_info.handle as isize == console_hwnd.0 {
                        continue;
                    }
                    let rect = get_window_rect(hwnd);
                    let w = rect.right - rect.left;
                    let h = rect.bottom - rect.top;
                    if w < 0 || h < 0 {
                        continue;
                    }
                    if w == 0 || h == 0 {
                        continue;
                    }
                    if !is_window(hwnd) {
                        continue;
                    }
                    if !window_visible(hwnd) {
                        continue;
                    }
                    if !window_enabled(hwnd) {
                        continue;
                    }
                    windows.push(hwnd);
                }
                let mut clw = c_l_w.lock().unwrap();
                clw.clear();
                for i in windows {
                    clw.push(i);
                }
            }
        });
        Ok(Self {
            capture_desktop,
            windows,
            current_list_windows_thread,
            current_list_windows,
            fake_monitor,
            v_mouse: VMouse::new(sk, radius)?,
            grabbed_window: None,
            resize_window: None,
            captured_window: None,
            skip_windows: vec![],
            console_hwnd,
            center: Vec3::new(0.0, 0.0, 0.0),
            radius,
            tick_counter: 0,
            lock_cursor: true,
        })
    }
    fn get_current_list_of_windows(&self) -> Vec<HWND> {
        let mut windows = Vec::new();
        for window in self.current_list_windows.lock().unwrap().iter() {
            windows.push(*window);
        }
        windows
    }
    pub fn highest_z_depth(&mut self) -> u32 {
        let mut highest = 0;
        for window in self.windows.values() {
            if window.z_depth > highest {
                highest = window.z_depth;
            }
        }
        highest
    }
    pub fn bring_to_top(&mut self, id: isize) -> Option<()> {
        let highest_z = self.highest_z_depth();
        let curr_z = self.windows.get(&id)?.z_depth;
        for (_, window) in &mut self.windows {
            if window.z_depth > curr_z {
                window.z_depth -= 1;
            }
        }
        self.windows.get_mut(&id)?.z_depth = highest_z;
        Some(())
    }
    pub fn delay_run(&mut self, sk: &StereoKitDraw) {
        self.tick_counter += 1;
        if self.tick_counter != 10 {
            return;
        }
        self.tick_counter = 0;
    }
    pub fn is_focused(&self, id: isize) -> bool {
        match self.captured_window.as_ref() {
            None => false,
            Some(window_id) => *window_id == id,
        }
    }
    pub fn draw(&mut self, sk: &StereoKitDraw, internal_mouse: &mut IMouse, keyboard_mouse: &mut KeyboardMouseState, radius: &mut f32) {
        self.delay_run(sk);
        let mut focus_changed = None;
        *radius = self.radius;
        self.v_mouse.update_pos(internal_mouse.delta_pos.x, internal_mouse.delta_pos.y);
        internal_mouse.lock_cursor = false;
        let mut invalid_windows = Vec::new();
        let windows = self.get_current_list_of_windows();
        for window in windows.iter() {
            if self.skip_windows.contains(&window.0) {
                continue;
            }
            if self.windows.contains_key(&window.0) {
                continue;
            }
            let z_depth = self.highest_z_depth() + 1;
            match VWindow::new(sk, *window, self.fake_monitor, Pose::new([0.0, 0.0, -self.radius], Quat::IDENTITY), z_depth, self.capture_desktop.clone()) {
                Ok(mut v_window) => {
                    v_window.internal_window.move_to_inactive();
                    v_window.internal_window.set_aspect_ratio(1.7);
                    self.windows.insert(window.0, v_window);
                }
                Err(err) => {
                    self.skip_windows.push(window.0);
                    println!("new_window_err: {err}")
                }
            }
        }
        let ids = self.windows.keys().map(|a| *a).collect::<Vec<_>>();
        for id in ids {
            let focused = self.is_focused(id);
            if IsWindowValid::Invalid == self.windows.get_mut(&id).unwrap().draw(sk, self.radius, focused) {
                println!("window is invalid: {}", id);
                invalid_windows.push(id);
            }
        }
        for invalid_window in invalid_windows {
            drop(self.windows.remove(&invalid_window));
        }

        if !self.lock_cursor {
            internal_mouse.lock_cursor = false;
            return;
        } else {
            //internal_mouse.lock_cursor = true;
        }

        if let Some((id, offset)) = self.grabbed_window.take() {
            self.v_mouse.draw(sk, Vec3::new(0.0, 0.0, 0.0));
            if keyboard_mouse.get_input(Key::MouseLeft).active {
                let mut position = cart_2_cyl(self.v_mouse.pos + offset);
                position.x = self.radius;
                let position = cyl_2_cart(position);
                self.windows.get_mut(&id).unwrap().pose.position = position.into();
                let face_user_quat = {
                    let mut quat = quat_lookat(self.center, position);
                    if self.v_mouse.pos.y > -0.7 {
                        quat.x = 0.0; quat.z = 0.0;
                    }
                    quat
                };
                self.windows.get_mut(&id).unwrap().pose.orientation = face_user_quat.into();
                self.grabbed_window.replace((id, offset));
                internal_mouse.lock_cursor = true;
            }
        } else {
            if let Some(id) = self.captured_window.take() {
                if let Some(window) = self.windows.get_mut(&id) {
                    if let Some(size) = window.internal_window.size() {
                        let position = window.internal_window.pos();

                        let mouse_pos = internal_mouse.pos();

                        let border = 8;

                        if mouse_pos.x - border < position.x || mouse_pos.y - border < position.y
                            || mouse_pos.x + border > position.x + size.x as i32 || mouse_pos.y + border + 2 > position.y + size.y as i32 {
                            let mut pos = Vec2::new(mouse_pos.x as f32, mouse_pos.y as f32);
                            pos.x -= position.x as f32;
                            pos.y -= position.y as f32;


                            pos.x /= size.x as f32;
                            pos.y /= size.y as f32;

                            pos.y = 1.0 - pos.y;


                            let bounds = window.window_capture.as_ref().unwrap().get_mesh().get_bounds(sk);
                            pos.y *= bounds.dimensions.y;
                            pos.x *= bounds.dimensions.x;


                            pos.x = pos.x - bounds.dimensions.x / 2.0;
                            pos.y = pos.y - bounds.dimensions.y / 2.0;

                            let pos = Vec3::new(pos.x, pos.y, 0.0);
                            let mat = Mat4::from(window.matrix().unwrap());
                            let pos = mat.transform_point3(pos);
                            let mut cyl_mouse_pos = cart_2_cyl(Vec3::new(pos.x, pos.y, pos.z));
                            cyl_mouse_pos.x = self.radius;
                            self.v_mouse.pos = cyl_2_cart(cyl_mouse_pos);
                            focus_changed = Some(id);
                        } else {
                            self.captured_window.replace(id);
                        }
                    }
                }
            } else {
                internal_mouse.lock_cursor = true;
                self.resize_or_capture_check(sk, keyboard_mouse, internal_mouse, &mut focus_changed);
                self.try_grab_window(sk, keyboard_mouse);
                self.v_mouse.draw(sk, Vec3::new(0.0, 0.0, 0.0));
            }
        }
        if let Some(changed_id) = focus_changed {
            match self.captured_window.is_some() {
                false => {
                    if let Some(window) = self.windows.get_mut(&changed_id) {
                        let size = window.internal_window.size().unwrap();
                        //let real_size = get_real_window_size(window.internal_window.hwnd);
                        if let Some(capture) = window.window_capture.as_mut() {
                            capture.mesh = WindowCapture::gen_mesh(sk, 0.0, 0.0, (size.x as f32 - 17.0) / size.x as f32, size.y as f32 / (size.y as f32 + 10.0)).unwrap();
                            capture.model.set_mesh(sk, 0, &capture.mesh);
                        }
                    }
                }
                true => {
                    if let Some(window) = self.windows.get_mut(&changed_id) {

                        let real_rect = get_real_window_rect(window.internal_window.hwnd);
                        let size = UVec2::from([(real_rect.right - real_rect.left) as u32, (real_rect.bottom - real_rect.top) as u32]);
                        let pos = IVec2::from([real_rect.left, real_rect.top]);
                        //let size = window.internal_window.size().unwrap();
                        //let pos = window.internal_window.pos();
                        let monitor_pos = self.fake_monitor.pos;
                        let monitor_size = self.fake_monitor.size;
                        let crop = calculate_crop_values( monitor_pos.x, monitor_pos.y, monitor_size.x, monitor_size.y, pos.x, pos.y, size.x, size.y);
                        if let Some(capture) = window.window_capture.as_mut() {
                            capture.mesh = WindowCapture::gen_mesh(sk, crop.0, crop.1, crop.2, crop.3).unwrap();
                            capture.model.set_mesh(sk, 0, &capture.mesh);
                        }
                    }
                }
            }
        }
    }
    fn try_grab_window(&mut self, sk: &StereoKitDraw, keyboard_mouse: &mut KeyboardMouseState) {
        let mut win_bring_top = None;
        for (id, window) in &self.windows {
            let mouse_ray = self.v_mouse.gen_ray(sk, Vec3::new(0.0, 0.0, 0.0), &window.grab_bar_matrix());
            if mouse_ray.model_intersect(&window.grab_bar.model, Cull::None).is_some() {
                if keyboard_mouse.get_input(Key::MouseLeft).active {
                    let offset = Vec3::from(window.pose.position) - self.v_mouse.pos;
                    self.grabbed_window.replace((*id, offset));
                    win_bring_top.replace(*id);
                    break;
                }
            }
        }
        if let Some(win_bring_top) = win_bring_top {
            self.bring_to_top(win_bring_top);
        }
    }
    fn resize_or_capture_check(&mut self, sk: &StereoKitDraw, keyboard_mouse: &mut KeyboardMouseState, internal_mouse: &mut IMouse, focus_changed: &mut Option<isize>) {
        if let Some((id, resize_type, offset)) = self.resize_window.take() {
            let mut change_aspect_ratio = None;
            if let Some(window) = self.windows.get(&id) {
                if keyboard_mouse.get_input(Key::MouseLeft).active {
                    let mut temp_offset = Vec3::from(window.pose.position);
                    if let Some(window_capture) = window.window_capture.as_ref() {
                        temp_offset.y -= (window_capture.get_mesh().get_bounds(sk).dimensions.x / 2.0);
                        let new_offset = temp_offset - self.v_mouse.pos;
                        match resize_type {
                            ResizeType::Vertical => {}
                            ResizeType::Horizontal => {
                                // let mut offset = cart_2_cyl(offset);
                                // offset.x = 2.0;
                                // let mut new_offset = cart_2_cyl(new_offset);
                                // new_offset.x = 2.0;
                                // let dist = new_offset.z - offset.z;
                                // if let Some(aspect_ratio) = window.internal_window.aspect_ratio() {
                                //     change_aspect_ratio.replace(aspect_ratio + ( dist * 0.01 ));
                                // }
                            }
                            ResizeType::MixedLeft => {}
                            ResizeType::MixedRight => {}
                        }
                    }
                    self.resize_window.replace((id, resize_type, offset));
                }
            }
            if let Some(aspect_ratio) = change_aspect_ratio {
                self.windows.get_mut(&id).unwrap().internal_window.set_aspect_ratio(aspect_ratio);
            }
        } else {
            self.v_mouse.set_cursor_type(CursorType::Point);
            let mut data_to_change = None;
            let mut resize = None;
            for (id, window) in &self.windows {
                let ray = self.v_mouse.gen_ray(sk, self.center, &Mat4::from(window.matrix().unwrap()));
                if let Some(window_capture) = window.window_capture.as_ref() {
                    let intersect = ray.model_intersect(window_capture.get_model(self.is_focused(*id), &self.capture_desktop), Cull::None);
                    if let Some(ray) = intersect {
                        let bounds = window_capture.get_mesh().get_bounds(sk);
                        let mut pos = Vec3::from(ray.pos);
                        pos.x += bounds.dimensions.x / 2.0;
                        pos.y += bounds.dimensions.y / 2.0;
                        pos.x /= bounds.dimensions.x;
                        pos.y /= bounds.dimensions.y;
                        pos.y = 1.0 - pos.y;

                        let aspect_ratio = match window.internal_window.aspect_ratio() {
                            None => continue,
                            Some(aspect_ratio) => aspect_ratio,
                        };

                        let mut temp_offset = Vec3::from(window.pose.position);
                        temp_offset.y -= (bounds.dimensions.y / 2.0);
                        let offset = temp_offset - self.v_mouse.pos;

                        let mouse_left = keyboard_mouse.get_input(Key::MouseLeft).active && keyboard_mouse.get_input(Key::MouseLeft).just_changed;

                        let mouse_right = keyboard_mouse.get_input(Key::MouseRight).active && keyboard_mouse.get_input(Key::MouseRight).just_changed;

                        let amount = 0.03;
                        let amount_2 = 1.0 - amount;
                        if pos.x <= amount && pos.y >= amount_2 {
                            self.v_mouse.set_cursor_type(CursorType::Resize(ResizeType::MixedLeft));
                            if keyboard_mouse.get_input(Key::MouseLeft).active && keyboard_mouse.get_input(Key::MouseLeft).just_changed {
                                //resize.replace((*id))
                                //self.resize_window.replace((*id, ResizeType::MixedLeft, offset));
                            }
                            break;
                        }
                        if pos.x <= amount && pos.y <= amount {
                            self.v_mouse.set_cursor_type(CursorType::Resize(ResizeType::MixedLeft));
                            if keyboard_mouse.get_input(Key::MouseLeft).active {
                                //self.resize_window.replace((*id, ResizeType::MixedLeft, offset));
                            }
                            break;
                        }
                        if pos.x >= amount_2 && pos.y >= amount {
                            self.v_mouse.set_cursor_type(CursorType::Resize(ResizeType::MixedRight));
                            if keyboard_mouse.get_input(Key::MouseLeft).active {
                                //self.resize_window.replace((*id, ResizeType::MixedRight, offset));
                            }
                            break;
                        }
                        if pos.x >= amount_2 && pos.y <= amount {
                            self.v_mouse.set_cursor_type(CursorType::Resize(ResizeType::MixedRight));
                            if keyboard_mouse.get_input(Key::MouseLeft).active {
                                //self.resize_window.replace((*id, ResizeType::MixedRight, offset));
                            }
                            break;
                        }

                        if pos.x <= amount {
                            self.v_mouse.set_cursor_type(CursorType::Resize(ResizeType::Horizontal));
                            if mouse_left {
                                resize.replace((*id, window.internal_window.aspect_ratio().unwrap() + 0.1));
                                //self.resize_window.replace((*id, ResizeType::Horizontal, offset));
                            }
                            if mouse_right {
                                resize.replace((*id, window.internal_window.aspect_ratio().unwrap() - 0.1));
                            }
                            break;
                        }
                        if pos.x >= amount_2 {
                            self.v_mouse.set_cursor_type(CursorType::Resize(ResizeType::Horizontal));
                            if mouse_left {
                                resize.replace((*id, window.internal_window.aspect_ratio().unwrap() + 0.1));
                                //self.resize_window.replace((*id, ResizeType::Horizontal, offset));
                            }
                            if mouse_right {
                                resize.replace((*id, window.internal_window.aspect_ratio().unwrap() - 0.1));
                            }
                            break;
                        }
                        if pos.y <= amount {
                            self.v_mouse.set_cursor_type(CursorType::Resize(ResizeType::Vertical));
                            if mouse_left {
                                resize.replace((*id, window.internal_window.aspect_ratio().unwrap() - 0.1));
                                //self.resize_window.replace((*id, ResizeType::Vertical, offset));
                            }
                            if mouse_right {
                                resize.replace((*id, window.internal_window.aspect_ratio().unwrap() + 0.1));
                            }
                            break;
                        }
                        if pos.y >= amount_2 {
                            self.v_mouse.set_cursor_type(CursorType::Resize(ResizeType::Vertical));
                            if mouse_left {
                                resize.replace((*id, window.internal_window.aspect_ratio().unwrap() - 0.1));
                                //self.resize_window.replace((*id, ResizeType::Vertical, offset));
                            }
                            if mouse_right {
                                resize.replace((*id, window.internal_window.aspect_ratio().unwrap() + 0.1));
                            }
                            break;
                        }

                        if let Some(size) = window.internal_window.size() {
                            pos.x *= size.x as f32;
                            pos.y *= size.y as f32;
                            let mut pos = IVec2::from([pos.x as i32, pos.y as i32]);
                            data_to_change.replace((pos, *id));
                            break;
                        }

                    }
                }
            }
            if let Some((id, aspect_ratio)) = resize {
                self.bring_to_top(id);
                self.windows.get_mut(&id).unwrap().internal_window.set_aspect_ratio(aspect_ratio);
            }
            if let Some((mut pos, id)) = data_to_change {
                focus_changed.replace(id);
                self.bring_to_top(id);
                for (_, win) in &mut self.windows {
                    win.internal_window.move_to_inactive();
                }
                let window = self.windows.get_mut(&id).unwrap();
                window.internal_window.move_to_active();
                self.captured_window.replace(id);
                pos.x += window.internal_window.pos().x;
                pos.y += window.internal_window.pos().y;
                internal_mouse.set_pos(pos);
                internal_mouse.lock_cursor = false;
            }
        }
    }
}
fn calculate_crop_values(
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: u32,
    monitor_height: u32,
    window_x: i32,
    window_y: i32,
    window_width: u32,
    window_height: u32,
) -> (f32, f32, f32, f32) {
    let monitor_x = monitor_x as f32;
    let monitor_y = monitor_y as f32;
    let monitor_width = monitor_width as f32;
    let monitor_height = monitor_height as f32;
    let window_x = window_x as f32;
    let window_y = window_y as f32;
    let window_width = window_width as f32;
    let window_height = window_height as f32;

    //println!("monitor_x: {monitor_x}, monitor_y: {monitor_y}, monitor_width: {monitor_width}, monitor_height: {monitor_height}");
    //println!("window_x: {window_x}, window_y: {window_y}, window_width: {window_width}, window_height: {window_height}");

    //(0.5025, 0.017, 0.9880, 0.4895)
    let mut crop_left = (window_x - monitor_x) / monitor_width;
    let mut crop_right = (monitor_x + monitor_width - window_x - window_width) / monitor_width;
    let mut crop_bottom = (window_y - monitor_y) / monitor_height;
    let mut crop_top = (monitor_y + monitor_height - window_y - window_height) / monitor_height;

    crop_bottom = 1.0 - crop_bottom;
    crop_top = 1.0 - crop_top;

    let border = 0.0001;
    crop_left += border;
    crop_right += border;
    crop_bottom -= border;
    crop_top -= border;
    //crop_left += 0.0022;
    //crop_right += 0.0068;
    //crop_bottom -= 0.004;
    //crop_top += 0.005;

    (crop_left, crop_right, crop_bottom, crop_top)
    //(0.0, 0.0, 1.0, 1.0)
}