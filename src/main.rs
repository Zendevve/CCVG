use eframe::{egui, NativeOptions};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;
use sysinfo::System;
use directories::UserDirs;
use walkdir::WalkDir;
use semver::Version;

const APP_TITLE: &str = "CapCut Version Guard";
const GITHUB_URL: &str = "https://github.com/Start9-Studios/CapCut-Version-Guard";

// --- Theme Constants ---
const COLOR_BG: egui::Color32 = egui::Color32::from_rgb(17, 24, 39);      // Gray 900
const COLOR_SURFACE: egui::Color32 = egui::Color32::from_rgb(31, 41, 55); // Gray 800
const COLOR_ACCENT: egui::Color32 = egui::Color32::from_rgb(59, 130, 246); // Blue 500
const COLOR_ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(37, 99, 235); // Blue 600
const COLOR_TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(243, 244, 246); // Gray 100
const COLOR_TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(156, 163, 175); // Gray 400
const COLOR_SUCCESS: egui::Color32 = egui::Color32::from_rgb(16, 185, 129); // Emerald 500
const COLOR_WARNING: egui::Color32 = egui::Color32::from_rgb(245, 158, 11); // Amber 500
const COLOR_ERROR: egui::Color32 = egui::Color32::from_rgb(239, 68, 68);    // Red 500

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([500.0, 500.0]) // Slightly taller for padding
            .with_resizable(false)
            .with_icon(eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()), // Could add real icon later
        ..Default::default()
    };
    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(CapCutGuardApp::new(cc)))),
    )
}

struct CapCutGuardApp {
    status_text: String,
    status_detail: String,
    status_color: egui::Color32,
    icon: String,
    is_scanning: bool,
    is_fixing: bool,
    capcut_running: bool,
    scan_requested: bool,
    fix_requested: bool,

    // Thread communication
    tx: std::sync::mpsc::Sender<WorkerMessage>,
    rx: std::sync::mpsc::Receiver<WorkerMessage>,
}

enum WorkerMessage {
    ScanResult(bool), // true if running
    FixStatus(String),
    FixComplete(Result<(), String>),
}

impl CapCutGuardApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_visuals(&cc.egui_ctx);
        configure_fonts(&cc.egui_ctx);

        let (tx, rx) = std::sync::mpsc::channel();
        let app = Self {
            status_text: "Initializing".to_owned(),
            status_detail: "Preparing environment...".to_owned(),
            status_color: COLOR_TEXT_SECONDARY,
            icon: "🛡️".to_owned(),
            is_scanning: false,
            is_fixing: false,
            capcut_running: false,
            scan_requested: true,
            fix_requested: false,
            tx,
            rx,
        };
        app
    }

    fn check_worker(&mut self) {
        if self.scan_requested {
            self.scan_requested = false;
            self.is_scanning = true;
            self.status_text = "Scanning".to_owned();
            self.status_detail = "Checking system processes...".to_owned();
            self.status_color = COLOR_TEXT_PRIMARY;
            self.icon = "🔍".to_owned();

            let tx = self.tx.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(600));
                let mut sys = System::new_all();
                sys.refresh_all();
                let running = sys.processes_by_name("CapCut").next().is_some() || sys.processes_by_name("CapCut.exe").next().is_some();
                let _ = tx.send(WorkerMessage::ScanResult(running));
            });
        }

        if self.fix_requested {
            self.fix_requested = false;
            self.is_fixing = true;
            self.status_text = "Securing CapCut".to_owned();
            self.status_detail = "Starting protection sequence...".to_owned();
            self.icon = "⚙️".to_owned();
            self.status_color = COLOR_ACCENT;

            let tx = self.tx.clone();
            thread::spawn(move || {
                match perform_fix(&tx) {
                    Ok(_) => { let _ = tx.send(WorkerMessage::FixComplete(Ok(()))); }
                    Err(e) => { let _ = tx.send(WorkerMessage::FixComplete(Err(e))); }
                }
            });
        }

        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                WorkerMessage::ScanResult(running) => {
                    self.is_scanning = false;
                    self.capcut_running = running;
                    if running {
                        self.status_text = "Action Required".to_owned();
                        self.status_detail = "CapCut is currently running. Please close it first.".to_owned();
                        self.status_color = COLOR_WARNING;
                        self.icon = "⚠️".to_owned();
                    } else {
                        self.status_text = "Ready to Secure".to_owned();
                        self.status_detail = "System is ready for version locking.".to_owned();
                        self.status_color = COLOR_SUCCESS;
                        self.icon = "🔓".to_owned();
                    }
                }
                WorkerMessage::FixStatus(msg) => {
                    self.status_detail = msg;
                }
                WorkerMessage::FixComplete(res) => {
                    self.is_fixing = false;
                    match res {
                        Ok(_) => {
                            self.status_text = "Protected".to_owned();
                            self.status_detail = "CapCut version is locked and guarded.".to_owned();
                            self.status_color = COLOR_SUCCESS;
                            self.icon = "🛡️".to_owned();
                        }
                        Err(e) => {
                            self.status_text = "Protection Failed".to_owned();
                            self.status_detail = e;
                            self.status_color = COLOR_ERROR;
                            self.icon = "❌".to_owned();
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for CapCutGuardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_worker();

        egui::CentralPanel::default().show(ctx, |ui| {
            // -- App Header --
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("CapCut Version Guard").size(28.0).strong().color(COLOR_TEXT_PRIMARY));
                ui.add_space(5.0);
                ui.label(egui::RichText::new("Automated Version Locking Tool").size(14.0).color(COLOR_TEXT_SECONDARY));
            });
            ui.add_space(30.0);

            // -- Card Area --
            let card_rect = ui.available_rect_before_wrap();
            let card_width = 400.0;
            let card_height = 200.0;


            egui::Frame::none()
                .fill(COLOR_SURFACE)
                .rounding(12.0)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(20)))
                .inner_margin(20.0)
                .outer_margin(egui::Margin::symmetric((ui.available_width() - card_width).max(0.0) / 2.0, 0.0))
                .show(ui, |ui| {
                    ui.set_min_size(egui::vec2(card_width, card_height));
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        // Icon
                        ui.label(egui::RichText::new(&self.icon).size(60.0));
                        ui.add_space(15.0);

                        // Main Status
                        ui.label(egui::RichText::new(&self.status_text).size(22.0).strong().color(self.status_color));
                        ui.add_space(8.0);

                        // Detail
                        ui.label(egui::RichText::new(&self.status_detail).size(15.0).color(COLOR_TEXT_SECONDARY));

                        // Spinner if working
                        if self.is_scanning || self.is_fixing {
                            ui.add_space(15.0);
                            ui.add(egui::Spinner::new().size(24.0).color(COLOR_ACCENT));
                        }
                    });
                });

            // -- Action Area --
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                if !self.is_scanning && !self.is_fixing {
                    let btn_size = egui::vec2(220.0, 50.0);

                    if self.capcut_running {
                         let btn = egui::Button::new(egui::RichText::new("Close CapCut & Retry").size(16.0).strong())
                            .fill(COLOR_WARNING)
                            .min_size(btn_size)
                            .rounding(8.0);

                         if ui.add(btn).clicked() {
                            self.scan_requested = true;
                         }
                    } else {
                        // Main Action
                        let (text, color) = if self.status_text == "Protected" {
                            ("Re-Scan Environment", COLOR_SURFACE)
                        } else {
                            ("Secure CapCut Now", COLOR_ACCENT)
                        };

                        // If it's pure Success, allow re-scan but emphasize success
                        let btn = egui::Button::new(egui::RichText::new(text).size(16.0).strong())
                            .fill(color)
                            .min_size(btn_size)
                            .rounding(8.0);

                        if ui.add(btn).clicked() {
                            if self.status_text == "Protected" {
                                self.scan_requested = true;
                            } else {
                                self.fix_requested = true;
                            }
                        }
                    }
                } else {
                    // Placeholder space to prevent jump
                    ui.add_space(50.0);
                }
            });

            // -- Footer --
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add_space(15.0);
                let link = ui.add(egui::Hyperlink::from_label_and_url(
                    egui::RichText::new("Star on GitHub").size(13.0).color(COLOR_TEXT_SECONDARY).underline(),
                    GITHUB_URL
                ));
            });
        });

        if self.is_scanning || self.is_fixing {
            ctx.request_repaint();
        }
    }
}

// --- Helpers ---

fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_rounding = egui::Rounding::same(0.0);
    visuals.panel_fill = COLOR_BG;

    visuals.widgets.noninteractive.bg_fill = COLOR_SURFACE;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, COLOR_TEXT_SECONDARY);

    visuals.widgets.inactive.bg_fill = COLOR_SURFACE;
    visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, COLOR_TEXT_PRIMARY);

    visuals.widgets.hovered.bg_fill = COLOR_ACCENT_HOVER;
    visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

    visuals.widgets.active.bg_fill = COLOR_ACCENT;
    visuals.widgets.active.rounding = egui::Rounding::same(8.0);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

    ctx.set_visuals(visuals);
}

fn configure_fonts(ctx: &egui::Context) {
    // Simple font config to ensure clean sizing
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (egui::TextStyle::Heading, egui::FontId::new(28.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Body, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Button, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Small, egui::FontId::new(12.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Monospace, egui::FontId::new(14.0, egui::FontFamily::Monospace)),
    ].into();
    ctx.set_style(style);
}

// --- Logic (Identical to previous, kept for completeness) ---

fn perform_fix(tx: &std::sync::mpsc::Sender<WorkerMessage>) -> Result<(), String> {
    let mut sys = System::new_all();
    sys.refresh_all();
    if sys.processes_by_name("CapCut").next().is_some() || sys.processes_by_name("CapCut.exe").next().is_some() {
        return Err("CapCut is running. Please close it.".to_string());
    }

    let local_app_data = std::env::var("LOCALAPPDATA").map_err(|_| "LOCALAPPDATA not found")?;
    let capcut_path = PathBuf::from(&local_app_data).join("CapCut");
    let apps_path = capcut_path.join("Apps");

    if !apps_path.exists() {
        return Err(format!("Apps folder not found at {:?}", apps_path));
    }

    let _ = tx.send(WorkerMessage::FixStatus("Cleaning old versions...".to_string()));
    clean_versions(&apps_path)?;

    let _ = tx.send(WorkerMessage::FixStatus("Locking configuration...".to_string()));
    lock_configuration(&apps_path)?;

    let _ = tx.send(WorkerMessage::FixStatus("Creating protection files...".to_string()));
    create_dummy_files(&capcut_path, &apps_path)?;

    Ok(())
}

fn clean_versions(apps_path: &Path) -> Result<(), String> {

    // Scan logic...
    // (Simplification of logic for this snippet, assumes reuse of logic from before)
    // To ensure this file runs standalone, I will include the full clean_versions logic here.

    // Re-read logic from previous file
    let mut dirs: Vec<PathBuf> = fs::read_dir(apps_path)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();

    dirs.sort_by(|a, b| {
         let na = a.file_name().unwrap_or_default().to_string_lossy();
         let nb = b.file_name().unwrap_or_default().to_string_lossy();
         human_sort::compare(&na, &nb)
    });

    if dirs.len() > 1 {
        let victim = dirs.last().unwrap();
        unset_readonly_recursive(victim)?;
        fs::remove_dir_all(victim).map_err(|e| format!("Failed to delete {:?}: {}", victim, e))?;
    }
    Ok(())
}

fn lock_configuration(apps_path: &Path) -> Result<(), String> {
    let config_path = apps_path.join("configure.ini");
    let content = if config_path.exists() {
        fs::read_to_string(&config_path).unwrap_or_default()
    } else {
        String::new()
    };
    let mut new_lines: Vec<String> = Vec::new();
    let mut found = false;
    for line in content.lines() {
        if line.trim().starts_with("last_version") {
            new_lines.push("last_version=1.0.0.0".to_string());
            found = true;
        } else {
            new_lines.push(line.to_string());
        }
    }
    if !found { new_lines.push("last_version=1.0.0.0".to_string()); }
    fs::write(config_path, new_lines.join("\n")).map_err(|e| e.to_string())?;
    Ok(())
}

fn create_dummy_files(capcut_path: &Path, apps_path: &Path) -> Result<(), String> {
    let pinfo = apps_path.join("ProductInfo.xml");
    create_readonly(&pinfo)?;
    let download_dir = capcut_path.join("User Data").join("Download");
    fs::create_dir_all(&download_dir).map_err(|e| e.to_string())?;
    let update_exe = download_dir.join("update.exe");
    create_readonly(&update_exe)?;
    Ok(())
}

fn create_readonly(path: &Path) -> Result<(), String> {
    if path.exists() {
        unset_readonly_recursive(path).ok();
        if path.is_dir() { fs::remove_dir_all(path).map_err(|e| e.to_string())?; }
        else { fs::remove_file(path).map_err(|e| e.to_string())?; }
    }
    fs::write(path, "").map_err(|e| e.to_string())?;
    Command::new("attrib").arg("+r").arg(path).output().map_err(|e| e.to_string())?;
    Ok(())
}

fn unset_readonly_recursive(path: &Path) -> Result<(), String> {
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        let mut perms = fs::metadata(p).map_err(|e| e.to_string())?.permissions();
        if perms.readonly() {
            perms.set_readonly(false);
            fs::set_permissions(p, perms).ok();
        }
    }
    Ok(())
}
