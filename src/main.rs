mod config;
mod file_monitor;
mod email;
mod ui;

use std::sync::mpsc;
use std::thread;
use eframe::egui;
use file_monitor::{process_event, watch_folder, PdfEvent};
use log::{debug, error, info};
use ui::{FileMonitorApp, LogLevel};

use env_logger::Env;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    log::info!("Starting file monitor application");

    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let config_path = current_dir.join("config").join("config.toml");
    log::info!("Attempting to load config from: {:?}", config_path);

    let config = match config::load_config(config_path.to_str().unwrap()) {
        Ok(cfg) => {
            log::info!("Configuration loaded successfully");
            log::debug!("Folder path: {}", cfg.folder_path);
            cfg
        },
        Err(e) => {
            log::error!("Failed to load configuration: {:?}", e);
            return;
        }
    };

    let email_settings = config.email_settings.clone();
    let folder_path = config.folder_path.clone();

    let (tx, rx) = mpsc::channel();
    let tx_clone = tx.clone();

    thread::spawn(move || {
        match watch_folder(&config.folder_path, move |event| {
            if let Some((pdf_event, message)) = process_event(event) {
                info!("{}", message);
                tx_clone.send((pdf_event, message)).unwrap_or_else(|e| error!("Failed to send message: {:?}", e));
            } else {
                debug!("Non-PDF file event detected");
            }
        }) {
            Ok(_) => info!("Folder watcher started successfully"),
            Err(e) => error!("Failed to start folder watcher: {:?}", e),
        }
    });

    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(egui::vec2(410.0, 500.0));
    options.resizable = false;
    eframe::run_native(
            "File Monitor",
            options,
            Box::new(|_cc| Box::new(MyApp::new(folder_path, rx, email_settings))),
    ).unwrap();
}

struct MyApp {
    file_monitor: FileMonitorApp,
    rx: mpsc::Receiver<(PdfEvent, String)>,
    email_settings: email::EmailSettings,
}

impl MyApp {
    fn new(folder_path: String, rx: mpsc::Receiver<(PdfEvent, String)>, email_settings: email::EmailSettings) -> Self {
        Self {
            file_monitor: FileMonitorApp::new(folder_path),
            rx,
            email_settings,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok((pdf_event, message)) = self.rx.try_recv() {
            let (log_message, log_level) = match &pdf_event {
                PdfEvent::Created(_) => (format!("New PDF file created: {}", message), LogLevel::Info),
                PdfEvent::Modified(_) => (format!("PDF file modified: {}", message), LogLevel::Warning),
                PdfEvent::Deleted(_) => (format!("PDF file deleted: {}", message), LogLevel::Error),
            };
        
            self.file_monitor.add_log_message(log_message.clone(), log_level);
            self.file_monitor.update_file_status(&pdf_event);
        
            if self.file_monitor.email_notify {
                match email::send_email(&self.email_settings, log_message.clone()) {
                    Ok(_) => {
                        self.file_monitor.add_log_message(format!("Email sent: {}", log_message), LogLevel::Info);
                    },
                    Err(e) => {
                        self.file_monitor.add_log_message(format!("Failed to send email: {}", e), LogLevel::Error);
                    },
                }
            }
        }
    
        egui::CentralPanel::default().show(ctx, |ui| {
            self.file_monitor.ui(ui);
        });
    
        ctx.request_repaint();
    }
}