//! UJournal: a cross-platform bullet journal written in Rust.

pub mod app;
pub mod error;
pub mod i18n;
pub mod journal;
pub mod theme;

pub use error::{JournalError, Result};
pub use i18n::Lang;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use crate::app::UJournalApp;
    use crate::journal::JournalStorage;

    let Some(data_dir) = app.internal_data_path() else {
        eprintln!("No se pudo determinar el directorio de datos de Android");
        return;
    };
    let storage = match JournalStorage::with_dir(data_dir.join("journals")) {
        Ok(storage) => storage,
        Err(error) => {
            eprintln!("No se pudo inicializar el almacenamiento: {error}");
            return;
        }
    };

    let options = eframe::NativeOptions {
        android_app: Some(app),
        renderer: eframe::Renderer::Wgpu,
        viewport: eframe::egui::ViewportBuilder::default().with_title("UJournal"),
        ..Default::default()
    };

    if let Err(error) = eframe::run_native(
        "UJournal",
        options,
        Box::new(move |cc| Ok(Box::new(UJournalApp::new(cc, storage)))),
    ) {
        eprintln!("No se pudo iniciar UJournal: {error}");
    }
}
