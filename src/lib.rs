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
    let persistence_path = data_dir.join("ui_prefs.ron");
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
        persistence_path: Some(persistence_path),
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

#[cfg(target_os = "android")]
mod ime_inset {
    use std::sync::atomic::{AtomicU32, Ordering};
    static BOTTOM_PX: AtomicU32 = AtomicU32::new(0);
    pub fn set_bottom_px(px: u32) { BOTTOM_PX.store(px, Ordering::Relaxed); }
    pub fn points(ctx: &eframe::egui::Context) -> f32 {
        let px = BOTTOM_PX.load(Ordering::Relaxed) as f32;
        if px <= 0.0 { 0.0 } else { px / ctx.pixels_per_point().max(1.0) }
    }
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_org_ujournal_MainActivity_nativeImeInset(
        _env: *mut std::ffi::c_void,
        _class: *mut std::ffi::c_void,
        bottom_px: i32,
    ) {
        set_bottom_px(bottom_px.max(0) as u32);
    }
}
#[cfg(target_os = "android")]
pub(crate) fn ime_inset_points(ctx: &eframe::egui::Context) -> f32 { ime_inset::points(ctx) }
#[cfg(not(target_os = "android"))]
pub(crate) fn ime_inset_points(_ctx: &eframe::egui::Context) -> f32 { 0.0 }
