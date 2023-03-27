use dxcapture::{Capture, Device};
use crate::internal_os::FakeMonitor;
use crate::windows_bindings::{get_window_rect, Hwnd, move_window};
use color_eyre::{Report, Result};
use crate::values::{IVec2, UVec2};

pub struct IWindow {
    pub(crate) hwnd: Hwnd,
    fake_monitor: FakeMonitor,
    stored_size: UVec2,
    padding: i32
}

impl IWindow {
    pub fn new(hwnd: Hwnd, fake_monitor: FakeMonitor) -> Result<Self> {
        let mut this = Self {
            hwnd,
            fake_monitor,
            stored_size: UVec2::from([0, 0]),
            padding: 40,
        };
        match this.size() {
            None => {
                Err(Report::msg("size of window was invalid, less then 0 in width or height"))
            }
            Some(size) => {
                this.stored_size = size;
                Ok(this)
            }
        }
    }
    pub fn size_changed(&self) -> bool {
        if let Some(size) = self.size() {
            if size != self.stored_size {
                return true;
            }
            return false;
        }
        true
    }
    pub fn update_size(&mut self) -> Option<()> {
        self.stored_size = self.size()?;
        Some(())
    }
    pub fn size(&self) -> Option<UVec2> {
        let rect = get_window_rect(self.hwnd);
        let w = rect.right - rect.left;
        let h = rect.bottom - rect.top;
        if w < 0 || h < 0 {
            return None;
        }
        let pos = [w as u32, h as u32].into();
        Some(pos)
    }
    pub fn aspect_ratio(&self) -> Option<f32> {
        let size = self.size()?;
        Some(size.x as f32 / size.y as f32)
    }
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) -> Option<()> {
        let max_width = (self.fake_monitor.size.x - (self.padding as u32 * 2)) / 2;
        let max_height = self.fake_monitor.size.y - (self.padding as u32 * 2);
        let mut width = max_width;
        let mut height = max_height;
        if (width as f32 / aspect_ratio) <= max_height as f32 {
            height = (width as f32 / aspect_ratio) as u32;
        } else {
            // If the width is too large, we fit the height to the aspect ratio
            width = (height as f32 * aspect_ratio) as u32;
        }
        self.set_size([width, height].into());
        Some(())
    }
    pub fn pos(&self) -> IVec2 {
        let rect = get_window_rect(self.hwnd);
        [rect.left, rect.top].into()
    }
    pub fn set_size(&mut self, size: UVec2) {
        //self.stored_size = size;
        let pos = self.pos();
        move_window(self.hwnd, pos.x, pos.y, size.x as i32, size.y as i32, true);
    }
    pub fn set_pos(&mut self, pos: IVec2) -> Option<()> {
        let size = self.size()?;
        move_window(self.hwnd, pos.x, pos.y, size.x as i32, size.y as i32, true);
        Some(())
    }
    pub fn move_to_active(&mut self) -> Option<()> {
        self.set_pos([self.fake_monitor.pos.x - self.padding + self.fake_monitor.size.x as i32 - self.size()?.x as i32, self.fake_monitor.pos.y + self.padding].into())
    }
    pub fn move_to_inactive(&mut self) -> Option<()> {
        self.set_pos([self.fake_monitor.pos.x + self.padding, self.fake_monitor.pos.y + self.padding].into())
    }
}