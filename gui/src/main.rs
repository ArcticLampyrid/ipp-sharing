#![windows_subsystem = "windows"]

use clap::Parser;
use eframe::egui::{self, vec2, Align, ViewportCommand};
use egui::{Align2, FontId, Id, PointerButton, Sense, UiBuilder};
use egui::{Button, RichText};
use ipp_sharing_core::config::read_config;
use ipp_sharing_core::ipp_sharing;
use log::{error, info};
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
struct Opts {
    #[arg(short, long)]
    config: Option<String>,
}

fn default_config_file_path() -> anyhow::Result<PathBuf> {
    let mut path = env::current_exe()?;
    path.pop();
    path.push("config.yaml");
    Ok(path)
}

async fn app_main() -> anyhow::Result<()> {
    let opts = match Opts::try_parse() {
        Ok(opts) => opts,
        Err(error) => {
            return Err(anyhow::anyhow!(
                "failed to parse command line arguments: {}",
                error
            ))
        }
    };
    let config_path = match opts.config {
        Some(path) => PathBuf::from_str(path.as_str())?,
        None => default_config_file_path()
            .map_err(|e| anyhow::anyhow!("failed to get default config file path: {}", e))?,
    };
    let config = read_config(config_path.as_path()).await.map_err(|e| {
        anyhow::anyhow!(
            "failed to read config file {}: {}",
            config_path.display(),
            e
        )
    })?;

    info!("Config File: {}", config_path.display());
    ipp_sharing(&config).await?;

    Ok(())
}

fn main() {
    egui_logger::builder()
        .init()
        .expect("Error initializing logger");

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.spawn(async {
        if let Err(e) = app_main().await {
            error!("Application error: {}", e);
        }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_maximized(true)
            .with_transparent(true)
            .with_active(true),

        ..Default::default()
    };
    eframe::run_native(
        "IPP Sharing",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(MyApp::default()))
        }),
    )
    .unwrap();

    runtime.shutdown_timeout(Duration::from_secs(1));
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "MiSans".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../fonts/MiSans-Regular.ttf"
        ))),
    );
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "MiSans".to_owned());
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("MiSans".to_owned());
    ctx.set_fonts(fonts);
}

#[derive(Default)]
struct MyApp {
    close_confirm_open: bool,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            title_bar_ui(ui, "IPP Sharing", &mut self.close_confirm_open);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("License: AGPLv3");
                ui.label(" | ");
                ui.hyperlink_to(
                    "Source Code",
                    "https://github.com/ArcticLampyrid/ipp-sharing",
                );
                ui.label(" | ");
                ui.hyperlink_to("Sponsor", "https://afdian.com/a/alampy");
            });
            egui::Window::new("Close Confirmation")
                .open(&mut self.close_confirm_open)
                .collapsible(false)
                .fixed_size(vec2(300.0, 100.0))
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label("Are you sure you want to close?");
                    ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                        if ui.button("Confirm").clicked() {
                            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                        }
                    });
                });
            egui_logger::LoggerUi::default().show(ui);
        });
    }
}

fn title_bar_ui(ui: &mut egui::Ui, title: &str, close_confirm_open: &mut bool) {
    let app_rect = ui.max_rect();
    let title_bar_height = 32.0;
    let title_bar_rect = {
        let mut rect = app_rect;
        rect.max.y = rect.min.y + title_bar_height;
        rect
    };

    ui.painter().text(
        title_bar_rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(20.0),
        ui.style().visuals.text_color(),
    );

    let title_bar_response = ui.interact(
        title_bar_rect,
        Id::new("title_bar"),
        Sense::click_and_drag(),
    );

    if title_bar_response.double_clicked() {
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx()
            .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
    }

    if title_bar_response.drag_started_by(PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }

    ui.scope_builder(
        UiBuilder::new()
            .max_rect(title_bar_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            let button_height = 20.0;
            let close_response = ui
                .add(Button::new(RichText::new("Close").size(button_height)))
                .on_hover_text("Close the window");
            if close_response.clicked() {
                *close_confirm_open = true;
            }
        },
    );

    ui.scope_builder(
        UiBuilder::new()
            .max_rect(title_bar_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            let button_height = 20.0;
            let minimized_response = ui
                .add(Button::new(RichText::new("Minimize").size(button_height)))
                .on_hover_text("Minimize the window");
            if minimized_response.clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
            }
        },
    );
}
