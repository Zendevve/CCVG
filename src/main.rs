use eframe::{egui, NativeOptions};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;
use sysinfo::System;
use walkdir::WalkDir;

const APP_TITLE: &str = "CapCut Version Guard";
const GITHUB_URL: &str = "https://github.com/Zendevve/capcut-version-guard";

// --- Premium Theme Colors (Cisco/Enterprise-Inspired) ---
const COLOR_BG_DEEP: egui::Color32 = egui::Color32::from_rgb(10, 12, 18);       // Near-black
const COLOR_BG_CARD: egui::Color32 = egui::Color32::from_rgb(22, 27, 38);       // Dark card
const COLOR_BG_CARD_HIGHLIGHT: egui::Color32 = egui::Color32::from_rgb(30, 38, 55); // Elevated card
const COLOR_BORDER_SUBTLE: egui::Color32 = egui::Color32::from_rgb(40, 45, 55);
const COLOR_BORDER_GLOW: egui::Color32 = egui::Color32::from_rgb(0, 100, 140);

const COLOR_ACCENT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(0, 164, 239);   // Cisco Blue
const COLOR_ACCENT_GRADIENT_END: egui::Color32 = egui::Color32::from_rgb(80, 200, 255);
const COLOR_SUCCESS: egui::Color32 = egui::Color32::from_rgb(40, 200, 140);        // Teal Green
const COLOR_WARNING: egui::Color32 = egui::Color32::from_rgb(255, 171, 0);         // Amber
const COLOR_ERROR: egui::Color32 = egui::Color32::from_rgb(255, 82, 82);           // Red

const COLOR_TEXT_BRIGHT: egui::Color32 = egui::Color32::from_rgb(248, 250, 252);
const COLOR_TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(148, 163, 184);
const COLOR_TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(100, 116, 139);

// --- Step Definitions ---
#[derive(Clone, PartialEq, Debug)]
enum ProgressStep {
    Idle,
    Scanning,
    CleaningVersions,
    LockingConfig,
    CreatingBlockers,
    Complete,
    Failed(String),
}

impl ProgressStep {
    fn label(&self) -> &str {
        match self {
            ProgressStep::Idle => "Idle",
            ProgressStep::Scanning => "Scanning System",
            ProgressStep::CleaningVersions => "Cleaning Old Versions",
            ProgressStep::LockingConfig => "Locking Configuration",
            ProgressStep::CreatingBlockers => "Creating Update Blockers",
            ProgressStep::Complete => "Protection Complete",
            ProgressStep::Failed(_) => "Protection Failed",
        }
    }

    fn index(&self) -> usize {
        match self {
            ProgressStep::Idle => 0,
            ProgressStep::Scanning => 1,
            ProgressStep::CleaningVersions => 2,
            ProgressStep::LockingConfig => 3,
            ProgressStep::CreatingBlockers => 4,
            ProgressStep::Complete => 5,
            ProgressStep::Failed(_) => 5,
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([560.0, 620.0])
            .with_resizable(false)
            .with_decorations(true),
        ..Default::default()
    };
    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(CapCutGuardApp::new(cc)))),
    )
}

struct CapCutGuardApp {
    current_step: ProgressStep,
    capcut_running: bool,
    action_log: Vec<String>,

    scan_requested: bool,
    fix_requested: bool,

    tx: std::sync::mpsc::Sender<WorkerMessage>,
    rx: std::sync::mpsc::Receiver<WorkerMessage>,
}

enum WorkerMessage {
    StepUpdate(ProgressStep),
    LogMessage(String),
    ScanResult(bool),
    FixComplete(Result<(), String>),
}

impl CapCutGuardApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_visuals(&cc.egui_ctx);
        configure_fonts(&cc.egui_ctx);

        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            current_step: ProgressStep::Idle,
            capcut_running: false,
            action_log: vec!["Application initialized.".to_string()],
            scan_requested: true, // Auto-scan on start
            fix_requested: false,
            tx,
            rx,
        }
    }

    fn is_working(&self) -> bool {
        !matches!(self.current_step, ProgressStep::Idle | ProgressStep::Complete | ProgressStep::Failed(_))
    }

    fn process_messages(&mut self) {
        // Start scan
        if self.scan_requested {
            self.scan_requested = false;
            self.current_step = ProgressStep::Scanning;
            self.action_log.push("Scanning for CapCut processes...".to_string());

            let tx = self.tx.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(800));
                let mut sys = System::new_all();
                sys.refresh_all();
                let running = sys.processes_by_name("CapCut").next().is_some()
                    || sys.processes_by_name("CapCut.exe").next().is_some();
                let _ = tx.send(WorkerMessage::ScanResult(running));
            });
        }

        // Start fix
        if self.fix_requested {
            self.fix_requested = false;
            self.current_step = ProgressStep::Scanning;
            self.action_log.clear();
            self.action_log.push("Starting protection sequence...".to_string());

            let tx = self.tx.clone();
            thread::spawn(move || {
                run_fix_sequence(&tx);
            });
        }

        // Process incoming messages
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                WorkerMessage::StepUpdate(step) => {
                    self.current_step = step;
                }
                WorkerMessage::LogMessage(log) => {
                    self.action_log.push(log);
                }
                WorkerMessage::ScanResult(running) => {
                    self.capcut_running = running;
                    if running {
                        self.current_step = ProgressStep::Failed("CapCut is running. Please close it.".to_string());
                        self.action_log.push("⚠ CapCut detected running.".to_string());
                    } else {
                        self.current_step = ProgressStep::Idle;
                        self.action_log.push("✓ No running CapCut instance found.".to_string());
                    }
                }
                WorkerMessage::FixComplete(res) => {
                    match res {
                        Ok(_) => {
                            self.current_step = ProgressStep::Complete;
                            self.action_log.push("✓ Protection sequence completed successfully.".to_string());
                        }
                        Err(e) => {
                            self.current_step = ProgressStep::Failed(e.clone());
                            self.action_log.push(format!("✗ Error: {}", e));
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for CapCutGuardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_messages();

        egui::CentralPanel::default().show(ctx, |ui| {
            // Wrap everything in a scroll area for responsiveness
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                let available_width = ui.available_width();
                let card_margin = 20.0_f32.min(available_width * 0.05);

                ui.add_space(20.0);

                // --- Header ---
                ui.vertical_centered(|ui| {
                    // Draw shield icon as painted shape (no Unicode)
                    let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::hover());
                    draw_shield_icon(ui.painter(), icon_rect.center(), 18.0, COLOR_ACCENT_PRIMARY);

                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("CapCut Version Guard").size(24.0).strong().color(COLOR_TEXT_BRIGHT));
                    ui.label(egui::RichText::new("Enterprise Edition").size(11.0).color(COLOR_TEXT_DIM));
                });
                ui.add_space(20.0);

                // --- Progress Steps Card ---
                egui::Frame::none()
                    .fill(COLOR_BG_CARD)
                    .rounding(12.0)
                    .stroke(egui::Stroke::new(1.0, COLOR_BORDER_SUBTLE))
                    .inner_margin(egui::Margin::symmetric(16.0, 14.0))
                    .outer_margin(egui::Margin::symmetric(card_margin, 0.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        ui.label(egui::RichText::new("Protection Status").size(12.0).strong().color(COLOR_TEXT_MUTED));
                        ui.add_space(12.0);

                        let steps = [
                            ("1", "System Scan", ProgressStep::Scanning),
                            ("2", "Version Cleanup", ProgressStep::CleaningVersions),
                            ("3", "Config Lock", ProgressStep::LockingConfig),
                            ("4", "Update Blocker", ProgressStep::CreatingBlockers),
                        ];

                        let current_idx = self.current_step.index();
                        let is_complete = matches!(self.current_step, ProgressStep::Complete);
                        let is_failed = matches!(self.current_step, ProgressStep::Failed(_));

                        for (i, (num, label, step)) in steps.iter().enumerate() {
                            let step_idx = step.index();
                            let is_active = current_idx == step_idx;
                            let is_done = current_idx > step_idx || is_complete;

                            ui.horizontal(|ui| {
                                let circle_color = if is_done {
                                    COLOR_SUCCESS
                                } else if is_active {
                                    COLOR_ACCENT_PRIMARY
                                } else {
                                    COLOR_BG_CARD_HIGHLIGHT
                                };

                                let circle_text_color = if is_done || is_active {
                                    COLOR_TEXT_BRIGHT
                                } else {
                                    COLOR_TEXT_DIM
                                };

                                let (rect, _) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
                                ui.painter().circle_filled(rect.center(), 12.0, circle_color);

                                if is_done {
                                    // Draw checkmark as lines (no Unicode)
                                    draw_checkmark(ui.painter(), rect.center(), 6.0, circle_text_color);
                                } else {
                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        *num,
                                        egui::FontId::new(11.0, egui::FontFamily::Proportional),
                                        circle_text_color,
                                    );
                                }

                                ui.add_space(10.0);

                                let label_color = if is_done || is_active {
                                    COLOR_TEXT_BRIGHT
                                } else {
                                    COLOR_TEXT_DIM
                                };
                                ui.label(egui::RichText::new(*label).size(13.0).color(label_color));

                                if is_active && !is_failed {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add(egui::Spinner::new().size(14.0).color(COLOR_ACCENT_PRIMARY));
                                    });
                                }
                            });

                            if i < steps.len() - 1 {
                                ui.horizontal(|ui| {
                                    ui.add_space(11.0);
                                    let line_color = if current_idx > step_idx || is_complete {
                                        COLOR_SUCCESS
                                    } else {
                                        COLOR_BG_CARD_HIGHLIGHT
                                    };
                                    let (rect, _) = ui.allocate_exact_size(egui::vec2(2.0, 12.0), egui::Sense::hover());
                                    ui.painter().rect_filled(rect, 1.0, line_color);
                                });
                            }
                        }

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(10.0);

                        let (status_text, status_color, show_shield) = match &self.current_step {
                            ProgressStep::Complete => ("System Protected", COLOR_SUCCESS, true),
                            ProgressStep::Failed(e) => (e.as_str(), COLOR_ERROR, false),
                            ProgressStep::Idle => ("Ready", COLOR_TEXT_MUTED, false),
                            _ => (self.current_step.label(), COLOR_ACCENT_PRIMARY, false),
                        };

                        ui.horizontal(|ui| {
                            // Draw status icon as shape
                            let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                            if show_shield {
                                draw_shield_icon(ui.painter(), icon_rect.center(), 8.0, status_color);
                            } else {
                                ui.painter().circle_filled(icon_rect.center(), 5.0, status_color);
                            }
                            ui.add_space(6.0);
                            ui.label(egui::RichText::new(status_text).size(14.0).strong().color(status_color));
                        });
                    });

                ui.add_space(16.0);

                // --- Activity Log Card ---
                egui::Frame::none()
                    .fill(COLOR_BG_CARD)
                    .rounding(12.0)
                    .stroke(egui::Stroke::new(1.0, COLOR_BORDER_SUBTLE))
                    .inner_margin(egui::Margin::symmetric(16.0, 12.0))
                    .outer_margin(egui::Margin::symmetric(card_margin, 0.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        ui.label(egui::RichText::new("Activity Log").size(12.0).strong().color(COLOR_TEXT_MUTED));
                        ui.add_space(8.0);

                        let log_height = 60.0_f32.min(ui.available_height() * 0.2).max(40.0);
                        egui::ScrollArea::vertical().max_height(log_height).show(ui, |ui| {
                            for log in &self.action_log {
                                let color = if log.starts_with("✓") {
                                    COLOR_SUCCESS
                                } else if log.starts_with("✗") || log.starts_with("⚠") {
                                    COLOR_WARNING
                                } else {
                                    COLOR_TEXT_DIM
                                };
                                ui.label(egui::RichText::new(log).size(11.0).color(color));
                            }
                        });
                    });

                ui.add_space(20.0);

                // --- Action Button ---
                ui.vertical_centered(|ui| {
                    if !self.is_working() {
                        let (btn_text, btn_color) = match &self.current_step {
                            ProgressStep::Complete => ("Re-Scan System", COLOR_BG_CARD_HIGHLIGHT),
                            ProgressStep::Failed(_) => ("Retry Protection", COLOR_WARNING),
                            _ => ("Secure CapCut Now", COLOR_ACCENT_PRIMARY),
                        };

                        let btn_width = 180.0_f32.min(available_width - 40.0);
                        let btn = egui::Button::new(
                            egui::RichText::new(btn_text).size(14.0).strong().color(COLOR_TEXT_BRIGHT)
                        )
                            .fill(btn_color)
                            .min_size(egui::vec2(btn_width, 44.0))
                            .rounding(8.0);

                        if ui.add(btn).clicked() {
                            if matches!(self.current_step, ProgressStep::Complete) {
                                self.scan_requested = true;
                                self.current_step = ProgressStep::Idle;
                            } else {
                                self.fix_requested = true;
                            }
                        }
                    } else {
                        ui.add_space(44.0); // Reserve button space
                    }
                });

                ui.add_space(16.0);

                // --- Footer (always at bottom of scroll content) ---
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 80.0).max(0.0) / 2.0);
                        if ui.link(egui::RichText::new("GitHub").size(10.0).color(COLOR_TEXT_DIM)).clicked() {
                            let _ = open::that(GITHUB_URL);
                        }
                        ui.label(egui::RichText::new("•").size(10.0).color(COLOR_TEXT_DIM));
                        ui.label(egui::RichText::new("v1.0.0").size(10.0).color(COLOR_TEXT_DIM));
                    });
                });

                ui.add_space(12.0);
            });
        });

        if self.is_working() {
            ctx.request_repaint();
        }
    }
}
// --- Icon Drawing Helpers (no Unicode dependency) ---

fn draw_checkmark(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
    // Draw a simple checkmark as two lines forming a "V" shape
    let stroke = egui::Stroke::new(2.0, color);
    let left = egui::pos2(center.x - size * 0.6, center.y);
    let bottom = egui::pos2(center.x - size * 0.1, center.y + size * 0.5);
    let right = egui::pos2(center.x + size * 0.6, center.y - size * 0.4);

    painter.line_segment([left, bottom], stroke);
    painter.line_segment([bottom, right], stroke);
}

fn draw_shield_icon(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
    // Draw a simple shield shape as a filled polygon
    let points = vec![
        egui::pos2(center.x, center.y - size),           // Top center
        egui::pos2(center.x + size, center.y - size * 0.5), // Top right
        egui::pos2(center.x + size, center.y + size * 0.3), // Bottom right
        egui::pos2(center.x, center.y + size),           // Bottom point
        egui::pos2(center.x - size, center.y + size * 0.3), // Bottom left
        egui::pos2(center.x - size, center.y - size * 0.5), // Top left
    ];

    let shape = egui::Shape::convex_polygon(points, color, egui::Stroke::NONE);
    painter.add(shape);
}

// --- Fix Sequence (with step updates) ---
fn run_fix_sequence(tx: &std::sync::mpsc::Sender<WorkerMessage>) {
    // Step 1: Scan
    let _ = tx.send(WorkerMessage::StepUpdate(ProgressStep::Scanning));
    let _ = tx.send(WorkerMessage::LogMessage("Checking for running CapCut processes...".to_string()));
    thread::sleep(Duration::from_millis(600));

    let mut sys = System::new_all();
    sys.refresh_all();
    if sys.processes_by_name("CapCut").next().is_some() || sys.processes_by_name("CapCut.exe").next().is_some() {
        let _ = tx.send(WorkerMessage::FixComplete(Err("CapCut is running. Please close it first.".to_string())));
        return;
    }
    let _ = tx.send(WorkerMessage::LogMessage("✓ No running instances detected.".to_string()));

    // Get paths
    let local_app_data = match std::env::var("LOCALAPPDATA") {
        Ok(p) => p,
        Err(_) => {
            let _ = tx.send(WorkerMessage::FixComplete(Err("LOCALAPPDATA not found".to_string())));
            return;
        }
    };
    let capcut_path = PathBuf::from(&local_app_data).join("CapCut");
    let apps_path = capcut_path.join("Apps");

    if !apps_path.exists() {
        let _ = tx.send(WorkerMessage::FixComplete(Err(format!("Apps folder not found at {:?}", apps_path))));
        return;
    }

    // Step 2: Clean versions
    let _ = tx.send(WorkerMessage::StepUpdate(ProgressStep::CleaningVersions));
    let _ = tx.send(WorkerMessage::LogMessage("Analyzing installed versions...".to_string()));
    thread::sleep(Duration::from_millis(500));

    if let Err(e) = clean_versions(&apps_path) {
        let _ = tx.send(WorkerMessage::FixComplete(Err(e)));
        return;
    }
    let _ = tx.send(WorkerMessage::LogMessage("✓ Version cleanup complete.".to_string()));

    // Step 3: Lock config
    let _ = tx.send(WorkerMessage::StepUpdate(ProgressStep::LockingConfig));
    let _ = tx.send(WorkerMessage::LogMessage("Modifying configuration file...".to_string()));
    thread::sleep(Duration::from_millis(400));

    if let Err(e) = lock_configuration(&apps_path) {
        let _ = tx.send(WorkerMessage::FixComplete(Err(e)));
        return;
    }
    let _ = tx.send(WorkerMessage::LogMessage("✓ Configuration locked.".to_string()));

    // Step 4: Create blockers
    let _ = tx.send(WorkerMessage::StepUpdate(ProgressStep::CreatingBlockers));
    let _ = tx.send(WorkerMessage::LogMessage("Creating update blocker files...".to_string()));
    thread::sleep(Duration::from_millis(400));

    if let Err(e) = create_dummy_files(&capcut_path, &apps_path) {
        let _ = tx.send(WorkerMessage::FixComplete(Err(e)));
        return;
    }
    let _ = tx.send(WorkerMessage::LogMessage("✓ Update blockers in place.".to_string()));

    // Done
    let _ = tx.send(WorkerMessage::FixComplete(Ok(())));
}

// --- Helper Functions ---
fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = COLOR_BG_DEEP;
    visuals.window_rounding = egui::Rounding::ZERO;

    visuals.widgets.noninteractive.bg_fill = COLOR_BG_CARD;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, COLOR_TEXT_MUTED);

    visuals.widgets.inactive.bg_fill = COLOR_BG_CARD_HIGHLIGHT;
    visuals.widgets.inactive.rounding = egui::Rounding::same(10.0);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, COLOR_TEXT_BRIGHT);

    visuals.widgets.hovered.bg_fill = COLOR_ACCENT_PRIMARY;
    visuals.widgets.hovered.rounding = egui::Rounding::same(10.0);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

    visuals.widgets.active.bg_fill = COLOR_ACCENT_GRADIENT_END;
    visuals.widgets.active.rounding = egui::Rounding::same(10.0);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

    ctx.set_visuals(visuals);
}

fn configure_fonts(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (egui::TextStyle::Heading, egui::FontId::new(26.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Button, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Small, egui::FontId::new(11.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Monospace, egui::FontId::new(12.0, egui::FontFamily::Monospace)),
    ].into();
    style.spacing.button_padding = egui::vec2(16.0, 8.0);
    ctx.set_style(style);
}

// --- Core Logic (unchanged) ---
fn clean_versions(apps_path: &Path) -> Result<(), String> {
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
