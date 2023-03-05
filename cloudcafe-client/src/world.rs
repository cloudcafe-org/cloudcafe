use glam::{Mat4, Quat, Vec3};
use stereokit::color_named::WHITE;
use stereokit::lifecycle::{StereoKitContext, StereoKitDraw};
use stereokit::model::Model;
use stereokit::render::RenderLayer;
use stereokit::shader::Shader;
use stereokit::texture::Texture;

pub struct WorldModel {
    world_model: Model,
    world_matrix: Mat4,
}
impl WorldModel {
    pub fn new(sk: &impl StereoKitContext) -> Self {
        let sky_texture= Texture::from_cubemap_equirectangular(sk, "./assets/table_mountain_2_puresky_4k.hdr", false, 0).unwrap();
        sk.set_skylight(&sky_texture.1);
        sk.set_skytex(&sky_texture.0);
        let world_model = Model::from_file(sk, "./assets/room.glb", Some(&Shader::default(sk))).unwrap();
        let world_matrix = Mat4::from_scale_rotation_translation(Vec3::new(0.35, 0.35, 0.35), Quat::IDENTITY, Vec3::new(0.0, 0.0, 0.0));
        Self {
            world_model,
            world_matrix,
        }
    }
    pub fn draw(&mut self, sk: &StereoKitDraw) {
        self.world_model.draw(sk, self.world_matrix.into(), WHITE, RenderLayer::Layer1);
    }
}