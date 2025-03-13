use egui::{NumExt as _, Vec2b};

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct AsteroidApp {
    pause: bool,
    time: f64,
}

impl AsteroidApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for AsteroidApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            self.pause = !self.pause;
        }

        egui::TopBottomPanel::top("Top panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Main panel");
            let plot = egui_plot::Plot::new("sin")
                .legend(egui_plot::Legend::default())
                .show_axes(true)
                .show_grid(true)
                .auto_bounds(Vec2b::new(false, false));

            let time = self.time;

            if !self.pause {
                ui.ctx().request_repaint();
                self.time += ui.input(|i| i.unstable_dt).at_most(1.0 / 30.0) as f64;
            }

            plot.show(ui, |plot_ui| {
                plot_ui.line(
                    egui_plot::Line::new(egui_plot::PlotPoints::from_explicit_callback(
                        move |x| (2.0 * x + time).sin(),
                        -2.0..2.0,
                        512,
                    ))
                    .color(egui::Color32::from_rgb(200, 100, 100))
                    .style(egui_plot::LineStyle::dotted_dense()),
                );
            })
            .response
        });
    }
}
