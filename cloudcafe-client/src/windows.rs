// use std::ffi::{c_char, c_void, CStr};
// use std::{mem, ptr};
// use std::ptr::{null, null_mut};
// use glam::{Quat, Vec3};
// use mint::{ColumnMatrix4, RowMatrix4};
// use stereokit::lifecycle::StereoKitContext;
// use stereokit::material::{DEFAULT_ID_MATERIAL_UNLIT_CLIP, Material};
// use stereokit::pose::Pose;
// use stereokit::texture::{Texture, TextureAddress, TextureFormat, TextureType};
// use stereokit::values::{MMatrix, MQuat, MVec3};
// use stereokit_sys::{default_id_material_unlit, material_copy_id, material_t, matrix, mesh_t, pose_t, quat, render_get_device, tex_create, tex_t, vec3};
// use ustr::ustr;
// use widestring::U16CStr;
// use winapi::Interface;
// use winapi::shared::guiddef::{REFGUID, REFIID};
// use winapi::shared::minwindef::{BOOL, FALSE, LPARAM, TRUE};
// use winapi::shared::ntdef::ULONG;
// use winapi::um::winnt::{CHAR, HANDLE, HRESULT, LPWSTR, LUID_AND_ATTRIBUTES, PLUID, ULONGLONG, WCHAR};
// use winapi::um::winuser::{EnumChildWindows, EnumWindows, GA_ROOT, GetAncestor, GetShellWindow, GetWindowLongPtrW, GetWindowTextLengthW, GetWindowTextW, GWL_STYLE, IsWindowVisible, WS_DISABLED};
// use winapi::shared::windef::HWND;
// use winapi::um::d3d11::{ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D};
//
// type Error = u32;
// type FnMessageBox = extern "stdcall" fn(hWnd: *const c_void, lpText: *const u16, lpCaption: *const u16, uType: u32) -> i32;
// type DwmGetDxSharedSurface = extern "stdcall" fn(hHandle: HANDLE, phSurface: *mut HANDLE, pAdapterLuid: PLUID, pFmtWindow: *mut ULONG, pPresentFlags: *mut ULONG, pWin32kUpdateId: *mut ULONGLONG) -> HRESULT;
//
//
// #[link(name = "kernel32")]
// #[no_mangle]
// extern "stdcall" {
//     fn GetLastError() -> Error;
//     fn LoadLibraryExW(lpLibFileName: *const u16, hFile: *const c_void, dwFlags: u32) -> *const c_void;
//     fn FreeLibrary(hLibModule: *const c_void) -> i32;
//     fn GetProcAddress(hModule: *const c_void, lpProcName: *const u8) -> *const c_void;
// }
//
// pub fn main(sk: &impl StereoKitContext) -> Vec<WindowT> {
//     unsafe {
//         let h = LoadLibraryExW("user32.dll".to_nullterminated_u16().as_ptr(),
//                                ptr::null(),
//                                0x800).to_result().unwrap();
//
//         let p = GetProcAddress(h, "DwmGetDxSharedSurface\0".as_ptr()).to_result().unwrap();
//
//         let dwm_get_dx_shared_surface = std::mem::transmute::<_, DwmGetDxSharedSurface>(p);
//
//
//         let mut mirror_device: *mut c_void = null_mut();
//         let mut context: *mut c_void = null_mut();
//
//         render_get_device(&mut mirror_device, &mut context);
//         println!("mirror_device: {:?}", mirror_device);
//
//         let mirror_device: &ID3D11Device = &*(mirror_device as *mut ID3D11Device);
//         let mut windows = Vec::new();
//         enumerate_windows(|hwnd: HWND| {
//             if hwnd == GetShellWindow() {
//                 return true;
//             }
//             if IsWindowVisible(hwnd) == 0 {
//                 return true;
//             }
//             if GetAncestor(hwnd, GA_ROOT) != hwnd {
//                 return true;
//             }
//             let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
//             if (style as u32 & WS_DISABLED) != 0 {
//                 return true;
//             }
//             let length = 256;
//             let mut bytes: Vec<u16> = Vec::with_capacity((length + 1) as usize);
//
//             GetWindowTextW(hwnd, bytes.as_mut_ptr() as *mut WCHAR, length);
//             let str = U16CStr::from_ptr_str(bytes.as_ptr());
//             if str.to_string().unwrap().len() < 2 {
//                 return true;
//             }
//             println!("hwnd: {:?}, title: {}", hwnd, str.to_string().unwrap());
//             let mut surface: HANDLE  = null_mut();
//             let mut luid = winapi::shared::ntdef::LUID{
//                 LowPart: 0,
//                 HighPart: 0,
//             };
//             let mut format = 0;
//             let mut flags = 0;
//             let mut update_id = 0;
//             if dwm_get_dx_shared_surface(hwnd as *mut winapi::ctypes::c_void, &mut (surface), PLUID::from(&mut luid), &mut format, &mut flags, &mut update_id) == 0 {
//                 println!("failed to get surface");
//                 return true;
//             }
//             let mut shared_tex: *mut ID3D11Texture2D = null_mut();
//             let mut shared_tex: *mut winapi::ctypes::c_void = null_mut();
//             println!("{:?}", surface);
//             let ref_id = REFIID::from(&ID3D11Texture2D::uuidof());
//             let hresult = mirror_device.OpenSharedResource(surface, ref_id, &mut shared_tex);
//             //let mut shared_tex: &ID3D11Texture2D = &*(shared_tex as *mut ID3D11Texture2D);
//             if hresult < 0{
//                 println!("failed to open shared resource for surface: {:?}", str.to_string().unwrap());
//                 return true;
//             }
//             println!("hresult: {:?}", hresult);
//             println!("format: {format}");
//             println!("luid: {:?}, {:?}", luid.HighPart, luid.LowPart);
//             let material = Material::copy_from_id(sk, DEFAULT_ID_MATERIAL_UNLIT_CLIP).unwrap();
//             let texture = Texture::create(sk, TextureType::ImageNoMips, TextureFormat::None).unwrap();
//             texture.set_surface(shared_tex as *mut c_void, TextureType::ImageNoMips, format as i64, 0, 0, 1, true);
//             texture.set_address_mode(TextureAddress::Clamp);
//             material.set_texture(sk, "diffuse", &texture).unwrap();
//             let window = WindowT {
//                 window: hwnd,
//                 texture,
//                 material,
//                 pose: Pose::new(Vec3::new(0.0, 0.0, 0.0), Quat::IDENTITY),
//                 name: str.to_string().unwrap(),
//             };
//             windows.push(window);
//             enumerate_child_windows(hwnd, |hwnd| {
//                 if hwnd == GetShellWindow() {
//                     return true;
//                 }
//                 if IsWindowVisible(hwnd) == 0 {
//                     return true;
//                 }
//                 if GetAncestor(hwnd, GA_ROOT) != hwnd {
//                     return true;
//                 }
//                 let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
//                 if (style as u32 & WS_DISABLED) != 0 {
//                     return true;
//                 }
//                 let length = 256;
//                 let mut bytes: Vec<u16> = Vec::with_capacity((length + 1) as usize);
//
//                 GetWindowTextW(hwnd, bytes.as_mut_ptr() as *mut WCHAR, length);
//                 let str = U16CStr::from_ptr_str(bytes.as_ptr());
//                 if str.to_string().unwrap().len() < 2 {
//                     return true;
//                 }
//                 println!("hwnd: {:?}, title: {}", hwnd, str.to_string().unwrap());
//                 let mut surface: HANDLE  = null_mut();
//                 let mut luid = winapi::shared::ntdef::LUID{
//                     LowPart: 0,
//                     HighPart: 0,
//                 };
//                 let mut format = 0;
//                 let mut flags = 0;
//                 let mut update_id = 0;
//                 if dwm_get_dx_shared_surface(hwnd as *mut winapi::ctypes::c_void, &mut (surface), PLUID::from(&mut luid), &mut format, &mut flags, &mut update_id) == 0 {
//                     println!("failed to get surface");
//                     return true;
//                 }
//                 let mut shared_tex: *mut ID3D11Texture2D = null_mut();
//                 let mut shared_tex: *mut winapi::ctypes::c_void = null_mut();
//                 println!("{:?}", surface);
//                 let ref_id = REFIID::from(&ID3D11Texture2D::uuidof());
//                 let hresult = mirror_device.OpenSharedResource(surface, ref_id, &mut shared_tex);
//                 //let mut shared_tex: &ID3D11Texture2D = &*(shared_tex as *mut ID3D11Texture2D);
//                 if hresult < 0{
//                     println!("failed to open shared resource for surface: {:?}", str.to_string().unwrap());
//                     return true;
//                 }
//                 println!("hresult: {:?}", hresult);
//                 println!("format: {format}");
//                 println!("luid: {:?}, {:?}", luid.HighPart, luid.LowPart);
//                 let material = Material::copy_from_id(sk, DEFAULT_ID_MATERIAL_UNLIT_CLIP).unwrap();
//                 let texture = Texture::create(sk, TextureType::ImageNoMips, TextureFormat::None).unwrap();
//                 texture.set_surface(shared_tex as *mut c_void, TextureType::ImageNoMips, format as i64, 0, 0, 1, true);
//                 texture.set_address_mode(TextureAddress::Clamp);
//                 material.set_texture(sk, "diffuse", &texture).unwrap();
//                 let window = WindowT {
//                     window: hwnd,
//                     texture,
//                     material,
//                     pose: Pose::new(Vec3::new(0.0, 0.0, 0.0), Quat::IDENTITY),
//                     name: str.to_string().unwrap(),
//                 };
//                 windows.push(window);
//                 true
//             });
//             true
//         });
//
//         FreeLibrary(h);
//         windows
//     }
// }
//
//
// pub struct WindowT {
//     pub window: HWND,
//     pub texture: Texture,
//     pub material: Material,
//     pub pose: Pose,
//     pub name: String,
// }
//
// trait ToResult: Sized {
//     fn to_result(&self) -> Result<Self, Error>;
// }
//
// impl ToResult for *const c_void {
//     fn to_result(&self) -> Result<*const c_void, Error> {
//         if *self == ptr::null() {
//             unsafe {
//                 Err(GetLastError())
//             }
//         } else {
//             Ok(*self)
//         }
//     }
// }
//
// trait IntoNullTerminatedU16 {
//     fn to_nullterminated_u16(&self) -> Vec<u16>;
// }
//
// impl IntoNullTerminatedU16 for str {
//     fn to_nullterminated_u16(&self) -> Vec<u16> {
//         self.encode_utf16().chain(Some(0)).collect()
//     }
// }
//
// pub fn enumerate_windows<F>(mut callback: F)
//     where F: FnMut(HWND) -> bool
// {
//     let mut trait_obj: &mut dyn FnMut(HWND) -> bool = &mut callback;
//     let closure_pointer_pointer: *mut c_void = unsafe { mem::transmute(&mut trait_obj) };
//
//     let lparam = closure_pointer_pointer as LPARAM;
//     unsafe { EnumWindows(Some(enumerate_callback), lparam) };
// }
//
// pub fn enumerate_child_windows<F>(hwnd: HWND, mut callback: F)
//     where F: FnMut(HWND) -> bool
// {
//     let mut trait_obj: &mut dyn FnMut(HWND) -> bool = &mut callback;
//     let closure_pointer_pointer: *mut c_void = unsafe { mem::transmute(&mut trait_obj) };
//
//     let lparam = closure_pointer_pointer as LPARAM;
//     unsafe { EnumChildWindows(hwnd, Some(enumerate_callback), lparam) };
// }
//
// unsafe extern "system" fn enumerate_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
//     let closure: &mut &mut dyn FnMut(HWND) -> bool = mem::transmute(lparam as *mut c_void);
//     if closure(hwnd) { TRUE } else { FALSE }
// }

