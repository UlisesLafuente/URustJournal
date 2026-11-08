use urust_journal::app::UrustJournalApp;
use urust_journal::journal::JournalStorage;

fn main() -> eframe::Result<()> {
    let storage = match JournalStorage::new_default() {
        Ok(storage) => storage,
        Err(error) => {
            eprintln!("No se pudo inicializar el almacenamiento: {error}");
            std::process::exit(1);
        }
    };

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_min_inner_size([640.0, 400.0])
            .with_title("UrustJournal"),
        ..Default::default()
    };

    eframe::run_native(
        "UrustJournal",
        options,
        Box::new(move |_cc| Ok(Box::new(UrustJournalApp::new(storage)))),
    )
}
