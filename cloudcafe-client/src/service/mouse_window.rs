use eframe::egui;
use egui::{Color32, Visuals};

pub(crate) fn main() -> Result<(), eframe::Error> {

    let options = eframe::NativeOptions {
        maximized: true,
        decorated: false,
        ..Default::default()
    };
    eframe::run_native(
        "Cloudcafe XR Desktop",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

struct MyApp {
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(Visuals {
            dark_mode: false,
            override_text_color: None,
            widgets: Default::default(),
            selection: Default::default(),
            hyperlink_color: Default::default(),
            faint_bg_color: Default::default(),
            extreme_bg_color: Color32::from_rgb(255, 255, 255),
            code_bg_color: Default::default(),
            warn_fg_color: Default::default(),
            error_fg_color: Default::default(),
            window_rounding: Default::default(),
            window_shadow: Default::default(),
            window_fill: Default::default(),
            window_stroke: Default::default(),
            menu_rounding: Default::default(),
            panel_fill: Default::default(),
            popup_shadow: Default::default(),
            resize_corner_size: 0.0,
            text_cursor_width: 0.0,
            text_cursor_preview: false,
            clip_rect_margin: 0.0,
            button_frame: false,
            collapsing_header_frame: false,
            indent_has_left_vline: false,
            striped: false,
            slider_trailing_fill: false,
        })
        // egui::CentralPanel::default().show(ctx, |ui| {
        //     ui.heading("My egui Application");
        //     ui.horizontal(|ui| {
        //         let name_label = ui.label("Your name: ");
        //         ui.text_edit_singleline(&mut self.name)
        //             .labelled_by(name_label.id);
        //     });
        //     ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
        //     if ui.button("Click each year").clicked() {
        //         self.age += 1;
        //     }
        //     ui.label(format!("Hello '{}', age {}", self.name, self.age));
        // });
    }
}