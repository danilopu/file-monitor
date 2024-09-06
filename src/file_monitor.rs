use log::error;
use notify::{Watcher, RecursiveMode, RecommendedWatcher, Config};
use notify::Event;
use std::sync::mpsc::channel;
use std::path::PathBuf;
use std::path::Path;

pub fn watch_folder<F>(path: &str, callback: F) -> Result<(), notify::Error>
where
    F: Fn(notify::Event) + Send + 'static,
{
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Config::default())?;
    
    watcher.watch(Path::new(path), RecursiveMode::Recursive)?;

    std::thread::spawn(move || {
        for event in rx {
            match event {
                Ok(event) => {
                    callback(event);
                },
                Err(e) => error!("Watch error: {:?}", e),
            }
        }
    });

    // Keep the watcher alive
    std::mem::forget(watcher);

    Ok(())
}

#[derive(Debug, Clone)]
pub enum PdfEvent {
    Modified(PathBuf),
    Created(PathBuf),
    Deleted(PathBuf),
}

pub fn process_event(event: Event) -> Option<(PdfEvent, String)> {
    if let Some(path) = event.paths.first() {
        if path.extension().map_or(false, |ext| ext == "pdf") {
            let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
            match event.kind {
                notify::EventKind::Create(_) => Some((PdfEvent::Created(path.to_owned()), format!("New PDF file created: {}", file_name))),
                notify::EventKind::Modify(_) => Some((PdfEvent::Modified(path.to_owned()), format!("PDF file modified: {}", file_name))),
                notify::EventKind::Remove(_) => Some((PdfEvent::Deleted(path.to_owned()), format!("PDF file deleted: {}", file_name))),
                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{CreateKind, EventKind};

    #[test]
    fn test_process_event() {
        let event = Event {
            kind: EventKind::Create(CreateKind::File),
            paths: vec![PathBuf::from("test_file.pdf")],
            attrs: notify::event::EventAttributes::new(),
        };

        if let Some((PdfEvent::Created(path), message)) = process_event(event) {
            assert_eq!(path, PathBuf::from("test_file.pdf"));
            assert_eq!(message, "New PDF file created: test_file.pdf");
        } else {
            panic!("Expected a Created event for PDF file");
        }
    }
}