use std::path::PathBuf;
use egui::{ScrollArea, Ui};

use crate::file_monitor::PdfEvent;

pub struct FileMonitorApp {
    folder_path: String,
    files: Vec<PathBuf>,
    url: String,
    pub email_notify: bool,
    pub show_log: bool,
    log_messages: Vec<LogEntry>,
    file_statuses: std::collections::HashMap<PathBuf, PdfEvent>,
}

use egui::RichText;

pub struct LogEntry {
    message: String,
    level: LogLevel,
}

pub enum LogLevel {
    Info,
    Warning,
    Error,
}

impl LogEntry {
    pub fn new(message: String, level: LogLevel) -> Self {
        Self { message, level }
    }

    pub fn rich_text(&self) -> RichText {
        let color = match self.level {
            LogLevel::Info => egui::Color32::GREEN,
            LogLevel::Warning => egui::Color32::YELLOW,
            LogLevel::Error => egui::Color32::RED,
        };
        RichText::new(&self.message).color(color)
    }
}

impl FileMonitorApp {
    pub fn new(folder_path: String) -> Self {
        let files = Self::get_files(&folder_path);
        Self {
            folder_path: folder_path.clone(),
            files,
            url: folder_path,
            email_notify: true,
            show_log: false,
            log_messages: Vec::new(),
            file_statuses: std::collections::HashMap::new(),
        }
    }

    fn get_files(folder_path: &str) -> Vec<PathBuf> {
        std::fs::read_dir(folder_path)
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.path())
                    .filter(|path| path.is_file())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn update(&mut self) {
        self.files = Self::get_files(&self.folder_path);
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.set_min_width(400.0);
        ui.set_max_width(400.0);
    
        ui.vertical_centered(|ui| {
            ui.heading("File Monitor");
        });
    
        ui.add_space(10.0);
    
        ui.horizontal(|ui| {
            ui.label("Folder path:");
            ui.add(egui::TextEdit::singleline(&mut self.url).desired_width(280.0));
            if ui.button("Search").clicked() {
                self.folder_path = self.url.clone();
                self.update();
            }
        });
    
        ui.add_space(10.0);
    
        ui.group(|ui| {
            ui.set_min_width(380.0);
            ui.label("Folder contents:");
            ScrollArea::vertical()
                .id_source("folder_contents_scroll")
                .max_height(150.0)
                .show(ui, |ui| {
                    for file in &self.files {
                        if let Some(file_name) = file.file_name() {
                            let text = file_name.to_string_lossy();
                            let color = match self.file_statuses.get(file) {
                                Some(PdfEvent::Created(_)) => egui::Color32::GREEN,
                                Some(PdfEvent::Modified(_)) => egui::Color32::YELLOW,
                                Some(PdfEvent::Deleted(_)) => egui::Color32::RED,
                                None => ui.style().visuals.text_color(),
                            };
                            ui.colored_label(color, text);
                        }
                    }
                });
        });
    
        ui.add_space(10.0);
    
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.email_notify, "Email Notify");
            ui.checkbox(&mut self.show_log, "Show Log");
        });
    
        ui.add_space(10.0);
    
        if self.show_log {
            ui.group(|ui| {
                ui.set_min_width(380.0);
                ui.label("Log:");
                ScrollArea::vertical()
                    .id_source("log_scroll")
                    .max_height(100.0)
                    .show(ui, |ui| {
                        for log_entry in &self.log_messages {
                            ui.label(log_entry.rich_text());
                        }
                    });
            });
        }
    }

    pub fn add_log_message(&mut self, message: String, level: LogLevel) {
        self.log_messages.push(LogEntry::new(message, level));
    }

    pub fn update_file_status(&mut self, event: &PdfEvent) {
        match event {
            PdfEvent::Created(path) | PdfEvent::Modified(path) => {
                self.file_statuses.insert(path.clone(), event.clone());
            }
            PdfEvent::Deleted(path) => {
                self.file_statuses.remove(path);
            }
        }
        self.update();
    }
}