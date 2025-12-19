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

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([500.0, 400.0])
            .with_resizable(false),
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
        // Customize look
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(egui::Color32::from_rgb(220, 220, 220));
        cc.egui_ctx.set_visuals(visuals);

        let (tx, rx) = std::sync::mpsc::channel();
        let app = Self {
            status_text: "Initializing...".to_owned(),
            status_color: egui::Color32::GRAY,
            icon: "🛡️".to_owned(),
            is_scanning: false,
            is_fixing: false,
            capcut_running: false,
            scan_requested: true, // Auto-start scan
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
            self.status_text = "Scanning for CapCut processes...".to_owned();
            self.status_color = egui::Color32::WHITE;

            let tx = self.tx.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(500)); // UX delay
                let mut sys = System::new_all();
                sys.refresh_all();
                let running = sys.processes_by_name("CapCut").next().is_some() || sys.processes_by_name("CapCut.exe").next().is_some();
                let _ = tx.send(WorkerMessage::ScanResult(running));
            });
        }

        if self.fix_requested {
            self.fix_requested = false;
            self.is_fixing = true;
            self.status_text = "Applying fixes...".to_owned();

            let tx = self.tx.clone();
            thread::spawn(move || {
                let res = perform_fix(&tx);
                if let Err(e) = res {
                    let _ = tx.send(WorkerMessage::FixComplete(Err(e)));
                } else {
                    let _ = tx.send(WorkerMessage::FixComplete(Ok(())));
                }
            });
        }

        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                WorkerMessage::ScanResult(running) => {
                    self.is_scanning = false;
                    self.capcut_running = running;
                    if running {
                        self.status_text = "CapCut is currently running!".to_owned();
                        self.status_color = egui::Color32::from_rgb(255, 165, 0); // Orange
                        self.icon = "⚠️".to_owned();
                    } else {
                        self.status_text = "Ready to Secure CapCut".to_owned();
                        self.status_color = egui::Color32::GREEN;
                        self.icon = "🔓".to_owned();
                    }
                }
                WorkerMessage::FixStatus(msg) => {
                    self.status_text = msg;
                }
                WorkerMessage::FixComplete(res) => {
                    self.is_fixing = false;
                    match res {
                        Ok(_) => {
                            self.status_text = "Success! CapCut is guarded.".to_owned();
                            self.status_color = egui::Color32::GREEN;
                            self.icon = "✅".to_owned();
                        }
                        Err(e) => {
                            self.status_text = format!("Error: {}", e);
                            self.status_color = egui::Color32::RED;
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
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading(egui::RichText::new("CapCut Version Guard").size(24.0).strong());

                ui.add_space(20.0);
                ui.label(egui::RichText::new(&self.status_text).size(14.0).color(self.status_color));

                ui.add_space(20.0);
                ui.label(egui::RichText::new(&self.icon).size(64.0));

                ui.add_space(30.0);

                if self.is_scanning || self.is_fixing {
                    ui.spinner();
                    ui.label("Working...");
                } else {
                    if self.capcut_running {
                         if ui.add(egui::Button::new(egui::RichText::new("Please Close CapCut").size(16.0)).fill(egui::Color32::from_rgb(200, 100, 0))).clicked() {
                            self.scan_requested = true;
                         }
                    } else {
                        // Main Action
                        let btn_text = if self.icon == "✅" { "Fix Issues Again" } else { "Fix Issues & Lock Version" };
                        let btn_color = if self.icon == "✅" { egui::Color32::DARK_GREEN } else { egui::Color32::from_rgb(31, 106, 165) };

                        if ui.add(egui::Button::new(egui::RichText::new(btn_text).size(16.0)).min_size(egui::vec2(200.0, 40.0)).fill(btn_color)).clicked() {
                            self.fix_requested = true;
                        }
                    }
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    if ui.link("Star on GitHub").clicked() {
                        let _ = open::that(GITHUB_URL);
                    }
                });
            });
        });

        // Refresh frame for thread updates if working
        if self.is_scanning || self.is_fixing {
            ctx.request_repaint();
        }
    }
}

fn perform_fix(tx: &std::sync::mpsc::Sender<WorkerMessage>) -> Result<(), String> {
    // 1. Process Check (Double Check)
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

    // 2. Version Cleaning
    let _ = tx.send(WorkerMessage::FixStatus("Cleaning old versions...".to_string()));
    clean_versions(&apps_path)?;

    // 3. INI Lock
    let _ = tx.send(WorkerMessage::FixStatus("Locking configuration...".to_string()));
    lock_configuration(&apps_path)?;

    // 4. Dummy Files
    let _ = tx.send(WorkerMessage::FixStatus("Creating dummy files...".to_string()));
    create_dummy_files(&capcut_path, &apps_path)?;

    Ok(())
}

fn clean_versions(apps_path: &Path) -> Result<(), String> {
    let mut versions = Vec::new();

    for entry in fs::read_dir(apps_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Heuristic: starts with digit
                if name.chars().next().map_or(false, |c| c.is_digit(10)) {
                    // Try parse
                     if let Ok(v) = Version::parse(name) {
                        versions.push((v, path));
                     } else {
                        // Try lenient parse (fix x.x.x.x to x.x.x-x or just ignore)
                        // Packaging version is looser than SemVer.
                        // We'll just store name and path and sort by name string as fallback if strict semver fails
                        // Actually, CapCut assumes 1.2.3.4, semver needs 1.2.3
                        // Let's just string sort for safety or custom parse.
                        // Simpler: Keep all directories that look like versions.
                         versions.push((Version::new(0,0,0), path)); // Placeholder for sort
                     }
                }
            }
        }
    }

    // Proper sorting by folder name string is "Okay" roughly, but let's be better.
    // If strict semver fails, we'll assume string sort.
    // Re-read and sort by string name naturally
    let mut dirs: Vec<PathBuf> = fs::read_dir(apps_path)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();

    dirs.sort_by(|a, b| {
         let na = a.file_name().unwrap_or_default().to_string_lossy();
         let nb = b.file_name().unwrap_or_default().to_string_lossy();
         // Attempt naive version comparison
         human_sort::compare(&na, &nb)
    });

    // Logic: Keep 2nd newest or 1.5.0.
    // If > 1 version, delete the last one (newest).
    if dirs.len() > 1 {
        let victim = dirs.last().unwrap();
        // Check if it is valid to delete? Assume yes per requirements.
        // remove_dir_all sometimes fails with read-only
        // So we unset read-only recursively first
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

    let mut new_lines = Vec::new();
    let mut found = false;

    for line in content.lines() {
        if line.trim().starts_with("last_version") {
            new_lines.push("last_version=1.0.0.0".to_string());
            found = true;
        } else {
            new_lines.push(line.to_string());
        }
    }

    if !found {
        new_lines.push("last_version=1.0.0.0".to_string());
    }

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
        unset_readonly_recursive(path).ok(); // ignore error if not dir
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| e.to_string())?;
        } else {
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }

    fs::write(path, "").map_err(|e| e.to_string())?;

    // Set Read-Only
    Command::new("attrib").arg("+r").arg(path).output().map_err(|e| e.to_string())?;
    Ok(())
}

fn unset_readonly_recursive(path: &Path) -> Result<(), String> {
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        // best effort
        let mut perms = fs::metadata(p).map_err(|e| e.to_string())?.permissions();
        if perms.readonly() {
            perms.set_readonly(false);
            fs::set_permissions(p, perms).ok();
        }
    }
    Ok(())
}
