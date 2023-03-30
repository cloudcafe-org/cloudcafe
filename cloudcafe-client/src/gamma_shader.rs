use stereokit::lifecycle::StereoKitContext;
use stereokit::shader::Shader;

#[allow(unused_variables)]
static mut GAMMA_SHADER: Option<Shader> = None;

#[allow(dead_code)]
pub fn gamma_shader(sk: &impl StereoKitContext) -> &'static Shader {
    unsafe {
        if GAMMA_SHADER.is_none() {
            GAMMA_SHADER.replace(Shader::from_mem(sk, include_bytes!("..\\assets\\desktop.hlsl.sks")).unwrap());
        }
        GAMMA_SHADER.as_ref().unwrap()
    }
}

static mut START_MENU_SHADER: Option<Shader> = None;

pub fn start_menu_shader(sk: &impl StereoKitContext) -> &'static Shader {
    unsafe {
        if START_MENU_SHADER.is_none() {
            START_MENU_SHADER.replace(Shader::from_mem(sk, include_bytes!("..\\assets\\start_menu.hlsl.sks")).unwrap());
        }
        START_MENU_SHADER.as_ref().unwrap()
    }
}