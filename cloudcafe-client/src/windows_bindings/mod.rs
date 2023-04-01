use std::ffi::{c_void, OsStr, OsString};
use std::mem::size_of;
use std::os::windows::prelude::OsStringExt;
use std::string::FromUtf16Error;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, POINT, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Graphics::Gdi::{GetDC, HDC};
use windows::Win32::System::Console::GetConsoleWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetClassNameW, GetCursorPos, GetWindow, GetWindowRect, GW_CHILD, GW_HWNDNEXT, IsWindow, IsWindowVisible, MoveWindow, SetCursorPos};
use crate::values::UVec2;

pub type Hwnd = HWND;
pub type Rect = RECT;
pub type Point = POINT;

pub fn get_window_rect(hwnd: Hwnd) -> Rect {
    println!("enter: get_window_rect");
    let mut rect = Rect::default();
    unsafe {
        GetWindowRect(hwnd, &mut rect);
    }
    println!("exit: get_window_rect");
    rect
}
pub fn get_real_window_rect(hwnd: Hwnd) -> Rect {
    println!("enter: get_real_window_rect");
    let mut rect = Rect::default();
    let mut frame = Rect::default();
    unsafe {
        GetWindowRect(hwnd, &mut rect);
        DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, &mut frame as *mut _ as *mut c_void, size_of::<RECT>() as u32).unwrap();
    }
    // let mut border = RECT {
    //     left: frame.left - rect.left,
    //     top: frame.top - rect.top,
    //     right: rect.right - frame.right,
    //     bottom: rect.bottom - frame.bottom,
    // };
    //
    // rect.left -= border.left;
    // rect.top -= border.top;
    // rect.right += border.left + border.right;
    // rect.bottom += border.top + border.bottom;

    frame.right -= 21;
    frame.top -= 10;
    println!("exit: get_real_window_rect");
    frame
}
pub fn get_real_window_size(hwnd: Hwnd) -> (u32, u32) {
    println!("enter: get_real_window_size");
    let rect = get_real_window_rect(hwnd);
    let ret = ((rect.right - rect.left) as u32, (rect.bottom - rect.top) as u32);
    println!("exit: get_real_window_size");
    ret
}
pub fn move_window(hwnd: Hwnd, x: i32, y: i32, width: i32, height: i32, repaint: bool) {
    println!("enter: move_window");
    unsafe {
        MoveWindow(hwnd, x, y, width, height, repaint);
    }
    println!("exit: move_window");
}
pub fn get_cursor_pos() -> Point {
    println!("enter: get_cursor_pos");
    let mut point = Point::default();
    unsafe {
        GetCursorPos(&mut point);
    }
    println!("exit: get_cursor_pos");
    point
}
pub fn set_cursor_pos(x: i32, y: i32) {
    println!("enter: get_cursor_pos");
    unsafe {
        SetCursorPos(x, y);
    }
    println!("exit: get_cursor_pos");
}
pub fn is_window(hwnd: Hwnd) -> bool {
    println!("enter: is_window");
    let ret = unsafe {
        IsWindow(hwnd)
    }.as_bool();
    println!("exit: is_window");
    ret
}
pub fn window_visible(hwnd: Hwnd) -> bool {
    println!("enter: window_visible");
    let ret = unsafe {
        IsWindowVisible(hwnd)
    }.as_bool();
    println!("exit: window_visible");
    ret
}
pub fn window_enabled(hwnd: Hwnd) -> bool {
    println!("enter: window_enabled");
    let ret = unsafe {
        IsWindowEnabled(hwnd)
    }.as_bool();
    println!("exit: window_enabled");
    ret
}
pub fn get_console_window() -> Option<Hwnd> {
    println!("enter: get_console_window");
    let hwnd = unsafe {
        GetConsoleWindow()
    };
    if hwnd.0 == 0 {
        println!("exit: get_console_window");
        return None;
    }
    println!("exit: get_console_window");
    return Some(hwnd)
}
pub fn class_name(hwnd: Hwnd) -> Option<String> {
    println!("enter: class_name");
    const MAX_CLASS_NAME_LENGTH: usize = 256;
    let mut class_name = [0u16; MAX_CLASS_NAME_LENGTH];
    let length = unsafe {
        GetClassNameW(hwnd, &mut class_name)
    };
    if length == 0 {
        return None;
    }
    let class_name = &class_name[..length as usize];
    let ret = match String::from_utf16(class_name) {
        Ok(string) => Some(string),
        Err(_) => None
    };
    println!("exit: class_name");
    ret
}
pub fn main_monitor_dimensions() -> UVec2 {
    println!("enter: main_monitor_dimensions");
    let main_monitor_width = unsafe { windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN) };
    let main_monitor_height = unsafe { windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN) };

    let ret = [main_monitor_width as u32, main_monitor_height as u32].into();
    println!("exit: main_monitor_dimensions");
    ret
}
pub fn get_dc(hwnd: Hwnd) -> HDC {
    println!("enter: get_dc");
    let ret = unsafe {
        GetDC(hwnd)
    };
    println!("exit: get_dc");
    ret
}
pub fn get_pixel(dc: HDC, x: i32, y: i32) -> COLORREF {
    println!("enter: get_pixel");
    let ret = unsafe {
        windows::Win32::Graphics::Gdi::GetPixel(dc, x, y)
    };
    println!("exit: get_pixel");
    ret
}