use std::sync::Arc;

use eframe::egui;

use crate::{pmx::Pmx, types::PmxTextGroup};

pub fn start(model: Arc<Pmx>) {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "mmd_rs ui",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, model)))),
    )
    .unwrap();
}

struct App {
    model: Arc<Pmx>,
    show_local_text: bool,
}

impl App {
    /// Selects either the local or universal text based on the current state of `show_local_text`.
    fn select_text<'a>(&self, group: &'a PmxTextGroup) -> &'a str {
        if self.show_local_text {
            group.local().as_str()
        } else {
            group.universal().as_str()
        }
    }

    fn new(cc: &eframe::CreationContext<'_>, model: Arc<Pmx>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        // NOTE currently you need to download the font yourself and place it in this directory, it
        // can be found at: https://github.com/notofonts/noto-cjk/releases/download/Serif2.003/01_NotoSerifCJK.ttc.zip
        // it is not uploaded to git since its 162mb
        // plan to add a feature to download it automatically in the future + detect the system font if it is available
        fonts.font_data.insert(
            "noto_cjk".to_owned(),
            egui::FontData::from_static(include_bytes!("../../fonts/NotoSerifCJK.ttc")).into(),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "noto_cjk".to_owned());

        cc.egui_ctx.set_fonts(fonts);

        Self {
            model,
            show_local_text: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                let show_local_button_text = if self.show_local_text {
                    "Hide local text"
                } else {
                    "Show local text"
                };

                ui.button(show_local_button_text)
                    .clicked()
                    .then(|| self.show_local_text = !self.show_local_text);

                let model_name = self.select_text(self.model.header().name());

                ui.heading(format!("Viewing model: {model_name}",));

                ui.heading("Comment:");
                // show comment
                let model_comment = self.select_text(self.model.header().comment());
                ui.label(model_comment);

                ui.heading("Textures:");
                for (i, texture) in self.model.textures().iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("Texture {i}:"));

                        if ui.link(texture.path().as_str()).clicked() {
                            #[cfg(target_os = "windows")]
                            {
                                std::process::Command::new("explorer")
                                    .arg("/select,")
                                    .arg(dbg!(self.model.base_path().join(texture.path().as_str())))
                                    .spawn()
                                    .expect("Failed to open texture path");
                            }
                            #[cfg(target_os = "macos")]
                            {
                                std::process::Command::new("open")
                                    .arg("-R")
                                    .arg(dbg!(self.model.base_path().join(texture.path().as_str())))
                                    .spawn()
                                    .expect("Failed to open texture path");
                            }
                            #[cfg(target_os = "linux")]
                            {
                                std::process::Command::new("xdg-open")
                                    .arg(dbg!(
                                        self.model
                                            .base_path()
                                            .join(texture.path().as_str())
                                            .parent()
                                            .unwrap()
                                    ))
                                    .spawn()
                                    .expect("Failed to open texture path");
                            }
                        };
                    });
                }

                ui.heading("Materials:");
                for (i, material) in self.model.materials().iter().enumerate() {
                    let mat_name = self.select_text(material.name());

                    ui.label(format!("Material {i}: {mat_name}"));
                }
            });
        });
    }
}
