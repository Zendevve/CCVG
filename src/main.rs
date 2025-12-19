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

// --- Theme Colors (60-30-10 Rule) ---
// 60% - Background/Neutral
const COLOR_BG: egui::Color32 = egui::Color32::from_rgb(15, 17, 23);           // Deep dark
const COLOR_BG_CARD: egui::Color32 = egui::Color32::from_rgb(24, 28, 38);      // Card surface
// 30% - Secondary
const COLOR_SECONDARY: egui::Color32 = egui::Color32::from_rgb(35, 42, 55);    // Borders, inactive
// 10% - Accent
const COLOR_ACCENT: egui::Color32 = egui::Color32::from_rgb(56, 189, 248);     // Sky blue
const COLOR_SUCCESS: egui::Color32 = egui::Color32::from_rgb(52, 211, 153);    // Emerald
const COLOR_WARNING: egui::Color32 = egui::Color32::from_rgb(251, 191, 36);    // Amber
const COLOR_ERROR: egui::Color32 = egui::Color32::from_rgb(248, 113, 113);     // Red

// Text colors
const COLOR_TEXT: egui::Color32 = egui::Color32::from_rgb(248, 250, 252);
const COLOR_TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(148, 163, 184);
const COLOR_TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(100, 116, 139);

// --- Wizard Screens ---
#[derive(Clone, PartialEq, Debug)]
enum WizardScreen {
    Welcome,
    PreCheck,
    VersionSelect,
    Running,
    Complete,
    Error(String),
}

// --- Version Info ---
#[derive(Clone, Debug)]
struct VersionInfo {
    name: String,
    path: PathBuf,
    size_mb: f64,
}

// --- Progress Steps (for Running screen) ---
#[derive(Clone, PartialEq, Debug)]
enum ProgressStep {
    Scanning,
    CleaningVersions,
    LockingConfig,
    CreatingBlockers,
    Done,
}

impl ProgressStep {
    fn index(&self) -> usize {
        match self {
            ProgressStep::Scanning => 0,
            ProgressStep::CleaningVersions => 1,
            ProgressStep::LockingConfig => 2,
            ProgressStep::CreatingBlockers => 3,
            ProgressStep::Done => 4,
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([520.0, 580.0])
            .with_resizable(true)
            .with_min_inner_size([400.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(CapCutGuardApp::new(cc)))),
    )
}

struct CapCutGuardApp {
    screen: WizardScreen,
    current_step: ProgressStep,
    action_log: Vec<String>,

    // Pre-check results
    capcut_found: bool,
    capcut_running: bool,
    capcut_path: Option<PathBuf>,

    // Version selection
    available_versions: Vec<VersionInfo>,
    selected_version_idx: Option<usize>,

    // Async
    check_requested: bool,
    fix_requested: bool,
    tx: std::sync::mpsc::Sender<WorkerMessage>,
    rx: std::sync::mpsc::Receiver<WorkerMessage>,
}

enum WorkerMessage {
    PreCheckResult { found: bool, running: bool, path: Option<PathBuf>, versions: Vec<VersionInfo> },
    StepUpdate(ProgressStep),
    LogMessage(String),
    FixComplete(Result<(), String>),
}

impl CapCutGuardApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_visuals(&cc.egui_ctx);
        configure_fonts(&cc.egui_ctx);

        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            screen: WizardScreen::Welcome,
            current_step: ProgressStep::Scanning,
            action_log: Vec::new(),
            capcut_found: false,
            capcut_running: false,
            capcut_path: None,
            available_versions: Vec::new(),
            selected_version_idx: None,
            check_requested: false,
            fix_requested: false,
            tx,
            rx,
        }
    }

    fn process_messages(&mut self) {
        // Start pre-check
        if self.check_requested {
            self.check_requested = false;
            self.screen = WizardScreen::PreCheck;

            let tx = self.tx.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(500));

                let mut sys = System::new_all();
                sys.refresh_all();
                let running = sys.processes_by_name("CapCut").next().is_some()
                    || sys.processes_by_name("CapCut.exe").next().is_some();

                let local_app_data = std::env::var("LOCALAPPDATA").ok();
                let path = local_app_data.map(|p| PathBuf::from(p).join("CapCut").join("Apps"));
                let found = path.as_ref().map(|p| p.exists()).unwrap_or(false);

                // Scan for versions
                let versions = if let Some(ref apps_path) = path {
                    scan_versions(apps_path)
                } else {
                    Vec::new()
                };

                let _ = tx.send(WorkerMessage::PreCheckResult { found, running, path, versions });
            });
        }

        // Start fix
        if self.fix_requested {
            self.fix_requested = false;
            self.screen = WizardScreen::Running;
            self.current_step = ProgressStep::Scanning;
            self.action_log.clear();

            let tx = self.tx.clone();
            let capcut_path = self.capcut_path.clone();
            let versions_to_delete: Vec<PathBuf> = self.available_versions
                .iter()
                .enumerate()
                .filter(|(idx, _)| Some(*idx) != self.selected_version_idx)
                .map(|(_, v)| v.path.clone())
                .collect();
            let selected_version = self.selected_version_idx
                .and_then(|idx| self.available_versions.get(idx).cloned());

            thread::spawn(move || {
                run_fix_sequence(&tx, capcut_path, versions_to_delete, selected_version);
            });
        }

        // Process messages
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                WorkerMessage::PreCheckResult { found, running, path, versions } => {
                    self.capcut_found = found;
                    self.capcut_running = running;
                    self.capcut_path = path;
                    self.available_versions = versions;
                    // Auto-select the oldest version (index 0) by default
                    if !self.available_versions.is_empty() {
                        self.selected_version_idx = Some(0);
                    }
                }
                WorkerMessage::StepUpdate(step) => {
                    self.current_step = step;
                }
                WorkerMessage::LogMessage(log) => {
                    self.action_log.push(log);
                }
                WorkerMessage::FixComplete(res) => {
                    match res {
                        Ok(_) => {
                            self.current_step = ProgressStep::Done;
                            self.screen = WizardScreen::Complete;
                        }
                        Err(e) => {
                            self.screen = WizardScreen::Error(e);
                        }
                    }
                }
            }
        }
    }

    fn is_working(&self) -> bool {
        matches!(self.screen, WizardScreen::Running) ||
        (matches!(self.screen, WizardScreen::PreCheck) && !self.capcut_found && !self.capcut_running)
    }
}

impl eframe::App for CapCutGuardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_messages();

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                match &self.screen.clone() {
                    WizardScreen::Welcome => self.render_welcome(ui),
                    WizardScreen::PreCheck => self.render_precheck(ui),
                    WizardScreen::VersionSelect => self.render_version_select(ui),
                    WizardScreen::Running => self.render_running(ui),
                    WizardScreen::Complete => self.render_complete(ui),
                    WizardScreen::Error(e) => self.render_error(ui, e),
                }
            });
        });

        if self.is_working() {
            ctx.request_repaint();
        }
    }
}

// --- Screen Renderers ---
impl CapCutGuardApp {
    fn render_welcome(&mut self, ui: &mut egui::Ui) {
        ui.add_space(60.0);

        ui.vertical_centered(|ui| {
            // Shield icon
            ui.label(egui::RichText::new(egui_phosphor::fill::SHIELD_CHECK).size(72.0).color(COLOR_ACCENT));
            ui.add_space(16.0);

            ui.label(egui::RichText::new("CapCut Version Guard").size(28.0).strong().color(COLOR_TEXT));
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Lock your CapCut version and prevent auto-updates").size(14.0).color(COLOR_TEXT_MUTED));

            ui.add_space(40.0);

            // Feature list
            egui::Frame::none()
                .fill(COLOR_BG_CARD)
                .rounding(12.0)
                .inner_margin(20.0)
                .outer_margin(egui::Margin::symmetric(40.0, 0.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    let features = [
                        (egui_phosphor::regular::MAGNIFYING_GLASS, "Detects installed CapCut versions"),
                        (egui_phosphor::regular::TRASH, "Removes unwanted version updates"),
                        (egui_phosphor::regular::LOCK_KEY, "Locks configuration to prevent changes"),
                        (egui_phosphor::regular::SHIELD_SLASH, "Blocks the auto-updater permanently"),
                    ];

                    for (icon, text) in features {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(icon).size(18.0).color(COLOR_ACCENT));
                            ui.add_space(12.0);
                            ui.label(egui::RichText::new(text).size(13.0).color(COLOR_TEXT_MUTED));
                        });
                        ui.add_space(8.0);
                    }
                });

            ui.add_space(40.0);

            // Start button
            let btn = egui::Button::new(
                egui::RichText::new(format!("{}  Start Protection", egui_phosphor::regular::ARROW_RIGHT))
                    .size(16.0).strong().color(COLOR_TEXT)
            )
                .fill(COLOR_ACCENT)
                .min_size(egui::vec2(200.0, 48.0))
                .rounding(10.0);

            if ui.add(btn).clicked() {
                self.check_requested = true;
            }
        });

        self.render_footer(ui);
    }

    fn render_precheck(&mut self, ui: &mut egui::Ui) {
        ui.add_space(50.0);

        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(egui_phosphor::fill::FOLDER_NOTCH_OPEN).size(56.0).color(COLOR_ACCENT));
            ui.add_space(12.0);
            ui.label(egui::RichText::new("System Check").size(24.0).strong().color(COLOR_TEXT));
            ui.label(egui::RichText::new("Verifying your system before proceeding").size(13.0).color(COLOR_TEXT_MUTED));
        });

        ui.add_space(30.0);

        // Check results
        egui::Frame::none()
            .fill(COLOR_BG_CARD)
            .rounding(12.0)
            .inner_margin(20.0)
            .outer_margin(egui::Margin::symmetric(40.0, 0.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // CapCut Installation
                ui.horizontal(|ui| {
                    let (icon, color, text) = if self.capcut_found {
                        (egui_phosphor::fill::CHECK_CIRCLE, COLOR_SUCCESS, "CapCut installation found")
                    } else if !self.capcut_running && !self.capcut_found {
                        (egui_phosphor::fill::CIRCLE_NOTCH, COLOR_ACCENT, "Checking installation...")
                    } else {
                        (egui_phosphor::fill::X_CIRCLE, COLOR_ERROR, "CapCut installation not found")
                    };
                    ui.label(egui::RichText::new(icon).size(20.0).color(color));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(text).size(14.0).color(COLOR_TEXT));
                });

                ui.add_space(12.0);

                // CapCut Running
                ui.horizontal(|ui| {
                    let (icon, color, text) = if self.capcut_running {
                        (egui_phosphor::fill::WARNING, COLOR_WARNING, "CapCut is running - please close it")
                    } else if self.capcut_found {
                        (egui_phosphor::fill::CHECK_CIRCLE, COLOR_SUCCESS, "CapCut is not running")
                    } else {
                        (egui_phosphor::fill::CIRCLE_NOTCH, COLOR_ACCENT, "Checking processes...")
                    };
                    ui.label(egui::RichText::new(icon).size(20.0).color(color));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(text).size(14.0).color(COLOR_TEXT));
                });
            });

        ui.add_space(30.0);

        ui.vertical_centered(|ui| {
            if self.capcut_found && !self.capcut_running {
                // Can proceed to version selection
                let btn = egui::Button::new(
                    egui::RichText::new(format!("{}  Select Version", egui_phosphor::regular::LIST_BULLETS))
                        .size(15.0).strong().color(COLOR_TEXT)
                )
                    .fill(COLOR_ACCENT)
                    .min_size(egui::vec2(180.0, 44.0))
                    .rounding(8.0);

                if ui.add(btn).clicked() {
                    self.screen = WizardScreen::VersionSelect;
                }
            } else if self.capcut_running {
                // CapCut running - retry
                let btn = egui::Button::new(
                    egui::RichText::new(format!("{}  Re-Check", egui_phosphor::regular::ARROW_CLOCKWISE))
                        .size(15.0).strong().color(COLOR_TEXT)
                )
                    .fill(COLOR_WARNING)
                    .min_size(egui::vec2(160.0, 44.0))
                    .rounding(8.0);

                if ui.add(btn).clicked() {
                    self.check_requested = true;
                }
            } else {
                // Still checking
                ui.add(egui::Spinner::new().size(24.0).color(COLOR_ACCENT));
            }

            ui.add_space(12.0);

            // Back button
            if ui.link(egui::RichText::new(format!("{} Back", egui_phosphor::regular::ARROW_LEFT)).size(13.0).color(COLOR_TEXT_DIM)).clicked() {
                self.screen = WizardScreen::Welcome;
            }
        });

        self.render_footer(ui);
    }

    fn render_running(&mut self, ui: &mut egui::Ui) {
        ui.add_space(40.0);

        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(egui_phosphor::fill::GEAR).size(48.0).color(COLOR_ACCENT));
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Applying Protection").size(22.0).strong().color(COLOR_TEXT));
            ui.label(egui::RichText::new("Please wait while we secure your CapCut installation").size(13.0).color(COLOR_TEXT_MUTED));
        });

        ui.add_space(24.0);

        // Progress steps
        egui::Frame::none()
            .fill(COLOR_BG_CARD)
            .rounding(12.0)
            .inner_margin(20.0)
            .outer_margin(egui::Margin::symmetric(40.0, 0.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                let steps = [
                    ("System Scan", ProgressStep::Scanning),
                    ("Version Cleanup", ProgressStep::CleaningVersions),
                    ("Config Lock", ProgressStep::LockingConfig),
                    ("Update Blocker", ProgressStep::CreatingBlockers),
                ];

                let current_idx = self.current_step.index();

                for (i, (label, step)) in steps.iter().enumerate() {
                    let step_idx = step.index();
                    let is_active = current_idx == step_idx;
                    let is_done = current_idx > step_idx;

                    ui.horizontal(|ui| {
                        let (icon, color) = if is_done {
                            (egui_phosphor::fill::CHECK_CIRCLE, COLOR_SUCCESS)
                        } else if is_active {
                            (egui_phosphor::fill::CIRCLE_NOTCH, COLOR_ACCENT)
                        } else {
                            (egui_phosphor::regular::CIRCLE, COLOR_SECONDARY)
                        };

                        ui.label(egui::RichText::new(icon).size(20.0).color(color));
                        ui.add_space(10.0);

                        let text_color = if is_done || is_active { COLOR_TEXT } else { COLOR_TEXT_DIM };
                        ui.label(egui::RichText::new(*label).size(14.0).color(text_color));

                        if is_active {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.add(egui::Spinner::new().size(14.0).color(COLOR_ACCENT));
                            });
                        }
                    });

                    if i < steps.len() - 1 {
                        ui.horizontal(|ui| {
                            ui.add_space(9.0);
                            let line_color = if current_idx > step_idx { COLOR_SUCCESS } else { COLOR_SECONDARY };
                            let (rect, _) = ui.allocate_exact_size(egui::vec2(2.0, 10.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 1.0, line_color);
                        });
                    }
                }
            });

        ui.add_space(20.0);

        // Activity log
        egui::Frame::none()
            .fill(COLOR_BG_CARD)
            .rounding(12.0)
            .inner_margin(16.0)
            .outer_margin(egui::Margin::symmetric(40.0, 0.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("Activity Log").size(11.0).strong().color(COLOR_TEXT_DIM));
                ui.add_space(6.0);

                egui::ScrollArea::vertical().max_height(60.0).show(ui, |ui| {
                    for log in &self.action_log {
                        let (prefix, color) = if log.starts_with("[OK]") {
                            (egui_phosphor::regular::CHECK, COLOR_SUCCESS)
                        } else if log.starts_with("[!]") {
                            (egui_phosphor::regular::WARNING, COLOR_WARNING)
                        } else {
                            (egui_phosphor::regular::DOT, COLOR_TEXT_DIM)
                        };

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(prefix).size(10.0).color(color));
                            ui.label(egui::RichText::new(log.trim_start_matches("[OK] ").trim_start_matches("[!] ").trim_start_matches(">> ")).size(11.0).color(COLOR_TEXT_DIM));
                        });
                    }
                });
            });

        self.render_footer(ui);
    }

    fn render_complete(&mut self, ui: &mut egui::Ui) {
        ui.add_space(60.0);

        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(egui_phosphor::fill::SHIELD_CHECK).size(80.0).color(COLOR_SUCCESS));
            ui.add_space(16.0);
            ui.label(egui::RichText::new("Protection Complete").size(26.0).strong().color(COLOR_TEXT));
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Your CapCut installation is now locked and protected").size(14.0).color(COLOR_TEXT_MUTED));
        });

        ui.add_space(30.0);

        // Summary
        egui::Frame::none()
            .fill(COLOR_BG_CARD)
            .rounding(12.0)
            .inner_margin(20.0)
            .outer_margin(egui::Margin::symmetric(50.0, 0.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("What was done:").size(12.0).strong().color(COLOR_TEXT_MUTED));
                ui.add_space(10.0);

                let items = [
                    (egui_phosphor::fill::CHECK, "Cleaned up extra version folders"),
                    (egui_phosphor::fill::CHECK, "Locked configuration file"),
                    (egui_phosphor::fill::CHECK, "Created update blocker files"),
                ];

                for (icon, text) in items {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(icon).size(14.0).color(COLOR_SUCCESS));
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(text).size(13.0).color(COLOR_TEXT));
                    });
                    ui.add_space(4.0);
                }
            });

        ui.add_space(30.0);

        ui.vertical_centered(|ui| {
            let btn = egui::Button::new(
                egui::RichText::new(format!("{}  Close", egui_phosphor::regular::X))
                    .size(15.0).color(COLOR_TEXT)
            )
                .fill(COLOR_SECONDARY)
                .min_size(egui::vec2(140.0, 42.0))
                .rounding(8.0);

            if ui.add(btn).clicked() {
                std::process::exit(0);
            }
        });

        self.render_footer(ui);
    }

    fn render_error(&mut self, ui: &mut egui::Ui, error: &str) {
        ui.add_space(60.0);

        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(egui_phosphor::fill::WARNING_OCTAGON).size(64.0).color(COLOR_ERROR));
            ui.add_space(16.0);
            ui.label(egui::RichText::new("Protection Failed").size(24.0).strong().color(COLOR_TEXT));
            ui.add_space(8.0);
            ui.label(egui::RichText::new(error).size(13.0).color(COLOR_TEXT_MUTED));
        });

        ui.add_space(30.0);

        ui.vertical_centered(|ui| {
            let btn = egui::Button::new(
                egui::RichText::new(format!("{}  Retry", egui_phosphor::regular::ARROW_CLOCKWISE))
                    .size(15.0).color(COLOR_TEXT)
            )
                .fill(COLOR_WARNING)
                .min_size(egui::vec2(140.0, 42.0))
                .rounding(8.0);

            if ui.add(btn).clicked() {
                self.screen = WizardScreen::Welcome;
            }
        });

        self.render_footer(ui);
    }

    fn render_version_select(&mut self, ui: &mut egui::Ui) {
        ui.add_space(40.0);

        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(egui_phosphor::fill::FOLDERS).size(48.0).color(COLOR_ACCENT));
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Select Version").size(22.0).strong().color(COLOR_TEXT));
            ui.label(egui::RichText::new("Choose the version you want to KEEP").size(13.0).color(COLOR_TEXT_MUTED));
        });

        ui.add_space(24.0);

        // Version list
        egui::Frame::none()
            .fill(COLOR_BG_CARD)
            .rounding(12.0)
            .inner_margin(16.0)
            .outer_margin(egui::Margin::symmetric(40.0, 0.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                if self.available_versions.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(egui_phosphor::regular::WARNING).size(16.0).color(COLOR_WARNING));
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("No versions found").size(14.0).color(COLOR_TEXT_MUTED));
                    });
                } else {
                    ui.label(egui::RichText::new(format!("{} versions detected", self.available_versions.len())).size(11.0).color(COLOR_TEXT_DIM));
                    ui.add_space(12.0);

                    let mut new_selection = self.selected_version_idx;

                    for (idx, version) in self.available_versions.iter().enumerate() {
                        let is_selected = self.selected_version_idx == Some(idx);
                        let is_oldest = idx == 0;
                        let is_newest = idx == self.available_versions.len() - 1;

                        let bg_color = if is_selected { COLOR_ACCENT } else { COLOR_SECONDARY };
                        let text_color = if is_selected { COLOR_TEXT } else { COLOR_TEXT_MUTED };

                        egui::Frame::none()
                            .fill(bg_color)
                            .rounding(8.0)
                            .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());

                                ui.horizontal(|ui| {
                                    // Radio indicator
                                    let radio_icon = if is_selected {
                                        egui_phosphor::fill::RADIO_BUTTON
                                    } else {
                                        egui_phosphor::regular::CIRCLE
                                    };
                                    ui.label(egui::RichText::new(radio_icon).size(16.0).color(text_color));
                                    ui.add_space(10.0);

                                    // Version name
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(&version.name).size(14.0).strong().color(text_color));
                                            if is_oldest {
                                                ui.label(egui::RichText::new(" (oldest)").size(11.0).color(COLOR_SUCCESS));
                                            }
                                            if is_newest {
                                                ui.label(egui::RichText::new(" (newest)").size(11.0).color(COLOR_WARNING));
                                            }
                                        });
                                        ui.label(egui::RichText::new(format!("{:.1} MB", version.size_mb)).size(11.0).color(text_color));
                                    });
                                });
                            });

                        // Make the whole frame clickable
                        let response = ui.interact(
                            ui.min_rect(),
                            ui.make_persistent_id(format!("version_{}", idx)),
                            egui::Sense::click()
                        );
                        if response.clicked() {
                            new_selection = Some(idx);
                        }

                        ui.add_space(6.0);
                    }

                    self.selected_version_idx = new_selection;
                }
            });

        ui.add_space(20.0);

        // Info box
        if self.available_versions.len() > 1 {
            egui::Frame::none()
                .fill(COLOR_BG_CARD)
                .rounding(8.0)
                .inner_margin(12.0)
                .outer_margin(egui::Margin::symmetric(40.0, 0.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(egui_phosphor::regular::INFO).size(14.0).color(COLOR_ACCENT));
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Other versions will be deleted. Selected version will be locked.").size(11.0).color(COLOR_TEXT_MUTED));
                    });
                });
        }

        ui.add_space(20.0);

        ui.vertical_centered(|ui| {
            let can_proceed = self.selected_version_idx.is_some();

            let btn = egui::Button::new(
                egui::RichText::new(format!("{}  Apply Protection", egui_phosphor::regular::SHIELD_CHECK))
                    .size(15.0).strong().color(COLOR_TEXT)
            )
                .fill(if can_proceed { COLOR_SUCCESS } else { COLOR_SECONDARY })
                .min_size(egui::vec2(180.0, 44.0))
                .rounding(8.0);

            if ui.add_enabled(can_proceed, btn).clicked() {
                self.fix_requested = true;
            }

            ui.add_space(12.0);

            if ui.link(egui::RichText::new(format!("{} Back", egui_phosphor::regular::ARROW_LEFT)).size(13.0).color(COLOR_TEXT_DIM)).clicked() {
                self.screen = WizardScreen::PreCheck;
            }
        });

        self.render_footer(ui);
    }

    fn render_footer(&self, ui: &mut egui::Ui) {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 80.0).max(0.0) / 2.0);
                if ui.link(egui::RichText::new("GitHub").size(10.0).color(COLOR_TEXT_DIM)).clicked() {
                    let _ = open::that(GITHUB_URL);
                }
                ui.label(egui::RichText::new("  v1.0.0").size(10.0).color(COLOR_TEXT_DIM));
            });
        });
        ui.add_space(12.0);
    }
}

// --- Fix Sequence ---
fn run_fix_sequence(
    tx: &std::sync::mpsc::Sender<WorkerMessage>,
    capcut_path: Option<PathBuf>,
    versions_to_delete: Vec<PathBuf>,
    selected_version: Option<VersionInfo>,
) {
    let _ = tx.send(WorkerMessage::StepUpdate(ProgressStep::Scanning));
    let _ = tx.send(WorkerMessage::LogMessage(">> Checking system state...".to_string()));
    thread::sleep(Duration::from_millis(500));

    let mut sys = System::new_all();
    sys.refresh_all();
    if sys.processes_by_name("CapCut").next().is_some() || sys.processes_by_name("CapCut.exe").next().is_some() {
        let _ = tx.send(WorkerMessage::FixComplete(Err("CapCut is still running. Please close it.".to_string())));
        return;
    }
    let _ = tx.send(WorkerMessage::LogMessage("[OK] No running instances".to_string()));

    let apps_path = match capcut_path {
        Some(p) => p,
        None => {
            let _ = tx.send(WorkerMessage::FixComplete(Err("CapCut path not found".to_string())));
            return;
        }
    };

    let capcut_root = apps_path.parent().unwrap_or(&apps_path).to_path_buf();

    // Step 2: Delete unselected versions
    let _ = tx.send(WorkerMessage::StepUpdate(ProgressStep::CleaningVersions));
    if let Some(ref ver) = selected_version {
        let _ = tx.send(WorkerMessage::LogMessage(format!(">> Keeping version: {}", ver.name)));
    }
    thread::sleep(Duration::from_millis(300));

    for path in &versions_to_delete {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let _ = tx.send(WorkerMessage::LogMessage(format!(">> Deleting: {}", name)));

        if let Err(e) = unset_readonly_recursive(path) {
            let _ = tx.send(WorkerMessage::LogMessage(format!("[!] Warning: {}", e)));
        }
        if let Err(e) = fs::remove_dir_all(path) {
            let _ = tx.send(WorkerMessage::FixComplete(Err(format!("Failed to delete {}: {}", name, e))));
            return;
        }
    }

    if versions_to_delete.is_empty() {
        let _ = tx.send(WorkerMessage::LogMessage("[OK] No versions to delete".to_string()));
    } else {
        let _ = tx.send(WorkerMessage::LogMessage(format!("[OK] Deleted {} version(s)", versions_to_delete.len())));
    }

    // Step 3: Lock config
    let _ = tx.send(WorkerMessage::StepUpdate(ProgressStep::LockingConfig));
    let _ = tx.send(WorkerMessage::LogMessage(">> Modifying config...".to_string()));
    thread::sleep(Duration::from_millis(300));

    if let Err(e) = lock_configuration(&apps_path) {
        let _ = tx.send(WorkerMessage::FixComplete(Err(e)));
        return;
    }
    let _ = tx.send(WorkerMessage::LogMessage("[OK] Configuration locked".to_string()));

    // Step 4: Create blockers
    let _ = tx.send(WorkerMessage::StepUpdate(ProgressStep::CreatingBlockers));
    let _ = tx.send(WorkerMessage::LogMessage(">> Creating blockers...".to_string()));
    thread::sleep(Duration::from_millis(300));

    if let Err(e) = create_dummy_files(&capcut_root, &apps_path) {
        let _ = tx.send(WorkerMessage::FixComplete(Err(e)));
        return;
    }
    let _ = tx.send(WorkerMessage::LogMessage("[OK] Update blockers created".to_string()));

    let _ = tx.send(WorkerMessage::FixComplete(Ok(())));
}

// --- Visual Config ---
fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = COLOR_BG;
    visuals.window_rounding = egui::Rounding::ZERO;

    visuals.widgets.noninteractive.bg_fill = COLOR_BG_CARD;
    visuals.widgets.inactive.bg_fill = COLOR_SECONDARY;
    visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
    visuals.widgets.hovered.bg_fill = COLOR_ACCENT;
    visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
    visuals.widgets.active.bg_fill = COLOR_ACCENT;
    visuals.widgets.active.rounding = egui::Rounding::same(8.0);

    ctx.set_visuals(visuals);
}

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Fill);
    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (egui::TextStyle::Heading, egui::FontId::new(24.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Button, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Small, egui::FontId::new(11.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Monospace, egui::FontId::new(12.0, egui::FontFamily::Monospace)),
    ].into();
    ctx.set_style(style);
}

// --- Version Scanning ---
fn scan_versions(apps_path: &Path) -> Vec<VersionInfo> {
    if !apps_path.exists() {
        return Vec::new();
    }

    let mut versions: Vec<VersionInfo> = fs::read_dir(apps_path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .map(|p| {
            let name = p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let size_mb = calculate_dir_size(&p) as f64 / (1024.0 * 1024.0);
            VersionInfo { name, path: p, size_mb }
        })
        .collect();

    // Sort by version name (oldest first)
    versions.sort_by(|a, b| human_sort::compare(&a.name, &b.name));
    versions
}

fn calculate_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

// --- Core Logic ---
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
        if let Ok(meta) = fs::metadata(p) {
            let mut perms = meta.permissions();
            if perms.readonly() {
                perms.set_readonly(false);
                fs::set_permissions(p, perms).ok();
            }
        }
    }
    Ok(())
}
