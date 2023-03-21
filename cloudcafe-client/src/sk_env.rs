use stereokit::lifecycle::{StereoKitContext, StereoKitDraw};
use stereokit::render::{RenderLayer, SphericalHarmonics};
use stereokit::texture::Texture;
use color_eyre::{Report, Result};
use glam::{Mat4, Quat, Vec3};
use stereokit::color_named::WHITE;
use stereokit::material::{DEFAULT_ID_MATERIAL_UNLIT, Material, Transparency};
use stereokit::model::Model;
use stereokit::shader::Shader;
use stereokit::values::Color128;

pub struct SkEnv {
    pub shader: Shader,
    pub skybox: Model,
    pub bridge_material: Material,
    pub bridge_lip: Model,
    pub bridge: Model,
}
impl SkEnv {
    pub fn new(sk: &impl StereoKitContext) -> Result<Self> {
        let shader = Shader::from_name(sk, "default/shader_unlit_clip")?;
        let skybox = Model::from_mem(sk, "skybox.glb", include_bytes!("..\\assets\\skybox.glb"), Some(&shader))?;
        let bridge_material: Material = Material::copy_from_id(sk, DEFAULT_ID_MATERIAL_UNLIT)?;
        let bridge_lip = Model::from_mem(sk, "top_base.glb", include_bytes!("..\\assets\\top_base.glb"), Some(&shader))?;
        let bridge = Model::from_mem(sk, "bottom_base.glb", include_bytes!("..\\assets\\bottom_base.glb"), Some(&shader))?;
        bridge_lip.set_material(sk, 0, &bridge_material);
        bridge.set_material(sk, 0, &bridge_material);
        bridge_material.set_transparency(sk, Transparency::Blend);
        bridge_material.set_parameter(sk, "color", &Color128::new(0.2, 0.2, 0.2, 1.0));
        Ok(Self {
            shader,
            skybox,
            bridge_material,
            bridge_lip,
            bridge,
        })
    }
    pub fn draw(&self, sk: &StereoKitDraw) {
        let scale = 0.15;
        self.skybox.draw(sk, Mat4::from_scale_rotation_translation(Vec3::new(scale, scale, scale), Quat::IDENTITY, Vec3::new(0.0, 0.0, 0.0)).into(),
                    WHITE, RenderLayer::Layer1);
        self.bridge_lip.draw(sk,
                   Mat4::from_scale_rotation_translation(Vec3::new(1.0, 1.0, 1.0), Quat::IDENTITY, Vec3::new(0.0, 0.0, 0.0)).into(),
                   Color128::new(0.2, 0.2, 0.2, 0.5), RenderLayer::Layer1);
        self.bridge.draw(sk,
                    Mat4::from_scale_rotation_translation(Vec3::new(1.0, 1.0, 1.0), Quat::IDENTITY, Vec3::new(0.0, 0.0, 0.0)).into(),
                    Color128::new(0.3, 0.3, 0.3, 0.8), RenderLayer::Layer1);
    }
}