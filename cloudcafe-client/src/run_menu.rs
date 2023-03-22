use std::collections::HashMap;
use std::f32::consts::PI;
use std::fs;
use std::fs::DirEntry;
use stereokit::lifecycle::{StereoKitContext, StereoKitDraw};
use color_eyre::Result;
use glam::{Mat4, Quat, Vec2, Vec3};
use glam::EulerRot::XYZ;
use runas::Command;
use stereokit::color_named;
use stereokit::color_named::BLANCHED_ALMOND;
use stereokit::font::Font;
use stereokit::material::{DEFAULT_ID_MATERIAL_UNLIT, Material, Transparency};
use stereokit::mesh::Mesh;
use stereokit::model::Model;
use stereokit::pose::Pose;
use stereokit::render::RenderLayer;
use stereokit::text::TextStyle;
use stereokit::ui::{MoveType, window, WindowType};
use stereokit::values::Color128;
use crate::input::{Key, KeyboardMouseState};
use crate::values::{quat_lookat, sphere_2_cart};

pub struct RunMenu {
    pose: Pose,
    input: Option<String>,
    entries: Vec<(String, DirEntry)>,
    search_textstyle: TextStyle,
    entry_textstyle: TextStyle,
    selected_option_mesh: Mesh,
    selected_option_material: Material,
    selected_option_model: Model,
    selected_option: Option<usize>,
}

pub const ALPHABET: [Key; 26] = [Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H, Key::I, Key::J, Key::K, Key::L, Key::M, Key::N, Key::O, Key::P, Key::Q, Key::R, Key::S, Key::T, Key::U, Key::V, Key::W, Key::X, Key::Y, Key::Z];

impl RunMenu {
    pub fn new(sk: &impl StereoKitContext) -> Result<Self> {
        let selected_option_mesh = Mesh::gen_plane(sk, [0.5, 0.5], Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0), 1)?;
        let selected_option_material = Material::copy_from_id(sk, DEFAULT_ID_MATERIAL_UNLIT)?;
        selected_option_material.set_transparency(sk, Transparency::Blend);
        let selected_option_model = Model::from_mesh(sk, &selected_option_mesh, &selected_option_material)?;

        let mut position = sphere_2_cart(Vec3::new(0.95, (PI / 2.0) + (PI / 16.0), -PI / 4.0));
        let mut entries = Vec::new();
        if let Ok(e) = fs::read_dir("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs") {
            for entry in e {
                if let Ok(entry) = entry {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    entries.push((name, entry));
                }
            }
        }
        entries.sort_by_key(|(k, _)| k.to_owned());
        Ok(Self {
            pose: Pose::new(position, quat_lookat(position, Vec3::new(0.0, 0.5, 0.0))),
            input: None,
            entries,
            search_textstyle: TextStyle::new(sk, Font::default(sk), 0.05, color_named::BURLY_WOOD),
            entry_textstyle: TextStyle::new(sk, Font::default(sk), 0.04, color_named::MOCCASIN),
            selected_option_mesh,
            selected_option_material,
            selected_option_model,
            selected_option: None,
        })
    }
    pub fn draw(&mut self, sk: &StereoKitDraw, keyboard_mouse: &mut KeyboardMouseState) {
        window(sk, "", &mut self.pose, Vec2::new(0.5, 0.5).into(), WindowType::WindowBody, MoveType::MoveNone, |ui| {
            if let Some(input) = self.input.as_ref() {
                if let Some(selected_entry) = self.selected_option.as_ref() {
                    self.selected_option_model.draw(sk,
                                                    Mat4::from_scale_rotation_translation(Vec3::new(1.0, 0.1, 1.0), Quat::from_euler(XYZ, 0.0, PI, 0.0), Vec3::new(0.0, -0.07 * (*selected_entry as f32) - 0.12, -0.001)).into(),
                                                    Color128::new(1.0, 0.9, 0.8, 0.6),
                                                    RenderLayer::Layer1);
                }
                ui.text_style(&self.search_textstyle, |ui| {
                    ui.label(input, false);
                });
                ui.text_style(&self.entry_textstyle, |ui| {
                    for (name, entry) in &self.entries {
                        if name.contains(input) {
                            ui.label(name, false);
                        }
                    }
                });
            }
        });
        if keyboard_mouse.get_input(Key::Windows).active {
            if keyboard_mouse.get_input(Key::O).active {
                self.input = Some(String::new());
                if self.entries.len() > 0 {
                    self.selected_option = Some(0);
                }
                return;
            }
        }

        let mut input_changed = false;
        if let Some(input) = self.input.as_mut() {
            if keyboard_mouse.get_input(Key::Backspace).just_changed && keyboard_mouse.get_input(Key::Backspace).active {
                input.pop();
                input_changed = true;
            }
            for key in ALPHABET {
                if keyboard_mouse.get_input(key).just_changed && keyboard_mouse.get_input(key).active {
                    input_changed = true;
                    input.push_str(key.as_str());
                }
            }
            let mut entries = Vec::new();
            for (name, dir) in &self.entries {
                if name.contains(input.as_str()) {
                    entries.push(dir);
                }
            }
            if input_changed {
                if let Some(selected) = self.selected_option.take() {
                    if selected >= entries.len() {
                        if entries.len() != 0 {
                            self.selected_option.replace(entries.len() - 1);
                        }
                    } else {
                        self.selected_option.replace(selected);
                    }
                }
            }
            if keyboard_mouse.get_input(Key::Enter).active {
                if let Some(selected) = self.selected_option.take() {
                    let path_to_run = entries.get(selected).unwrap().path();
                    println!("running: {:?}", path_to_run);
                    let _ = std::process::Command::new("cmd.exe").arg("/c").arg(path_to_run).spawn().unwrap();
                    self.input.take();
                }
            }
            if keyboard_mouse.get_input(Key::ArrowUp).just_changed && keyboard_mouse.get_input(Key::ArrowUp).active {
                if let Some(selected) = self.selected_option.take() {
                    if selected == 0 {
                        self.selected_option.replace(0);
                    } else {
                        self.selected_option.replace(selected - 1);
                    }
                }
            }
            if keyboard_mouse.get_input(Key::ArrowDown).just_changed && keyboard_mouse.get_input(Key::ArrowDown).active {
                if let Some(selected) = self.selected_option.take() {
                    if selected == entries.len() - 1 {
                        self.selected_option.replace(selected);
                    } else {
                        self.selected_option.replace(selected + 1);
                    }
                }
            }
        }
    }
}