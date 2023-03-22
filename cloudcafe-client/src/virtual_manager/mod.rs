use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use dxcapture::{enumerate_displays, enumerate_windows};
use glam::Quat;
use stereokit::lifecycle::{StereoKitContext, StereoKitDraw};
use stereokit::pose::Pose;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::HMONITOR;
use crate::internal_os::FakeMonitor;
use crate::internal_os::internal_window::IWindow;
use crate::virtual_manager::virtual_window::{IsWindowValid, VWindow};
use crate::windows_bindings::{get_window_rect, Hwnd, is_window, Rect, window_enabled, window_visible};
use color_eyre::Result;

pub mod virtual_mouse;
pub mod virtual_window;

pub const INVALID_WINDOW_TITLES: [&'static str; 4] = ["Cloudcafe XR Desktop", "SteamVR", "OBS", "Mixed Reality Portal"];

pub struct VDesktop {
    windows: HashMap<isize, VWindow>,
    current_list_windows_thread: JoinHandle<()>,
    current_list_windows: Arc<Mutex<Vec<Hwnd>>>,
    fake_monitor: FakeMonitor,
    should_draw: bool,
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
    pub fn new(sk: &impl StereoKitContext) -> Result<Self> {
        let fake_monitor = {
            let displays = enumerate_displays();
            let monitor = displays.first().unwrap();
          FakeMonitor::new(HMONITOR(monitor.handle as isize),Rect {
              left: monitor.monitor_rect.left,
              top: monitor.monitor_rect.top,
              right: monitor.monitor_rect.right,
              bottom: monitor.monitor_rect.bottom,
          })?
        };
        let mut hwnds = Vec::new();
        for window_info in enumerate_windows() {
            if is_invalid_window(&window_info.title) {
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
        for i_window in i_windows {
            if let Ok(v_window) = VWindow::new(sk, i_window.hwnd, fake_monitor, Pose::new([0.0, 0.0, -2.0], Quat::IDENTITY)) {
                windows.insert(i_window.hwnd.0, v_window);
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
            windows,
            current_list_windows_thread,
            current_list_windows,
            fake_monitor,
            should_draw: false,
        })
    }
    fn get_current_list_of_windows(&self) -> Vec<HWND> {
        let mut windows = Vec::new();
        for window in self.current_list_windows.lock().unwrap().iter() {
            windows.push(*window);
        }
        windows
    }
    pub fn draw(&mut self, sk: &StereoKitDraw) {
        let mut invalid_windows = Vec::new();
        let windows = self.get_current_list_of_windows();
        for window in windows.iter() {
            if !self.windows.contains_key(&window.0) {
                match VWindow::new(sk, *window, self.fake_monitor, Pose::new([0.0, 0.0, -2.0], Quat::IDENTITY)) {
                    Ok(v_window) => {
                        self.windows.insert(window.0, v_window);
                    }
                    Err(err) => {
                        println!("new_window_err: {err}")
                    }
                }
            }
        }
        for (id, window) in &mut self.windows {
            if IsWindowValid::Invalid == window.draw(sk) {
                println!("window is invalid: {}", id);
                invalid_windows.push(*id);
            }
        }
        for invalid_window in invalid_windows {
            drop(self.windows.remove(&invalid_window));
        }
        for (id, window) in &mut self.windows {
            window.internal_window.move_to_inactive();
        }
    }
}