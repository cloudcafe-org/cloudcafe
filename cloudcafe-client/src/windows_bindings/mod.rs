use std::ffi::{OsStr, OsString};
use std::os::windows::prelude::OsStringExt;
use std::string::FromUtf16Error;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::System::Console::GetConsoleWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
use windows::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetCursorPos, GetWindow, GetWindowRect, GW_CHILD, GW_HWNDNEXT, IsWindow, IsWindowVisible, MoveWindow, SetCursorPos};

pub type Hwnd = HWND;
pub type Rect = RECT;
pub type Point = POINT;

pub fn get_window_rect(hwnd: Hwnd) -> Rect {
    let mut rect = Rect::default();
    unsafe {
        GetWindowRect(hwnd, &mut rect);
    }
    rect
}
pub fn move_window(hwnd: Hwnd, x: i32, y: i32, width: i32, height: i32, repaint: bool) {
    unsafe {
        MoveWindow(hwnd, x, y, width, height, repaint);
    }
}
pub fn get_cursor_pos() -> Point {
    let mut point = Point::default();
    unsafe {
        GetCursorPos(&mut point);
    }
    point
}
pub fn set_cursor_pos(x: i32, y: i32) {
    unsafe {
        SetCursorPos(x, y);
    }
}
pub fn is_window(hwnd: Hwnd) -> bool {
    unsafe {
        IsWindow(hwnd)
    }.as_bool()
}
pub fn window_visible(hwnd: Hwnd) -> bool {
    unsafe {
        IsWindowVisible(hwnd)
    }.as_bool()
}
pub fn window_enabled(hwnd: Hwnd) -> bool {
    unsafe {
        IsWindowEnabled(hwnd)
    }.as_bool()
}
pub fn get_console_window() -> Option<Hwnd> {
    let hwnd = unsafe {
        GetConsoleWindow()
    };
    if hwnd.0 == 0 {
        return None;
    }
    return Some(hwnd)
}
pub fn class_name(hwnd: Hwnd) -> Option<String> {
    const MAX_CLASS_NAME_LENGTH: usize = 256;
    let mut class_name = [0u16; MAX_CLASS_NAME_LENGTH];
    let length = unsafe {
        GetClassNameW(hwnd, &mut class_name)
    };
    if length == 0 {
        return None;
    }
    let class_name = &class_name[..length as usize];
    match String::from_utf16(class_name) {
        Ok(string) => Some(string),
        Err(_) => None
    }
}