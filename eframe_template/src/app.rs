use egui::NumExt as _;
use market::amount::Amount;
use simulation::configurations::MarketConfiguration;

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct AsteroidApp {
    pause: bool,
    time: f64,
    last_sim_step: f64,

    dt: f64,
    no_transactions: u64,

    #[serde(skip)]
    data: Vec<Amount>,

    #[serde(skip)]
    market_configuration: MarketConfiguration,
}

impl AsteroidApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self {
                dt: 0.1,
                pause: false,
                ..Default::default()
            }
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
            let max_y = self
                .data
                .iter()
                .map(|x| x.as_int)
                .reduce(i64::max)
                .unwrap_or(0) as f32;
            ui.heading("Main panel");
            let plot = egui_plot::Plot::new("sin")
                .legend(egui_plot::Legend::default())
                .include_x(self.data.len() as f32)
                .include_x(0)
                .include_y(max_y)
                .include_y(0)
                .show_axes(true)
                .show_grid(true);

            println!("{}, {}, {}", self.pause, self.no_transactions, self.time);

            if !self.pause {
                ui.ctx().request_repaint();
                let dt = ui.input(|i| i.unstable_dt).at_most(1.0 / 30.0) as f64;
                self.time += dt;
            }

            // if self.no_transactions > 10 {
            //     self.pause = true;
            // }

            if !self.pause {
                println!(
                    "history: {:?}",
                    self.market_configuration.history.transactions
                );
            }

            if self.last_sim_step + self.dt < self.time {
                println!("update");
                let price = self
                    .market_configuration
                    .step()
                    .or(self.data.last().cloned());
                self.data.push(price.unwrap_or_default());
                self.last_sim_step = self.time;

                if self.market_configuration.history.no_transactions() {
                    self.no_transactions += 1;
                }
            }

            plot.show(ui, |plot_ui| {
                plot_ui.line(
                    egui_plot::Line::new(egui_plot::PlotPoints::new(
                        self.data
                            .iter()
                            .enumerate()
                            .map(|(i, v)| [i as f64, v.as_int as f64])
                            .collect(),
                    ))
                    .color(egui::Color32::from_rgb(200, 100, 100))
                    .style(egui_plot::LineStyle::dotted_dense()),
                );
            })
            .response
        });
    }
}
