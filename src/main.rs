use eframe::egui;
use urust_journal::app::UrustJournalApp;
use urust_journal::journal::JournalStorage;

const APP_ID: &str = "org.urustjournal.UrustJournal";

fn main() -> eframe::Result<()> {
    let storage = match JournalStorage::new_default() {
        Ok(storage) => storage,
        Err(error) => {
            eprintln!("No se pudo inicializar el almacenamiento: {error}");
            std::process::exit(1);
        }
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_min_inner_size([640.0, 400.0])
            .with_title("UrustJournal")
            .with_app_id(APP_ID)
            .with_icon(generic_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "UrustJournal",
        options,
        Box::new(move |cc| Ok(Box::new(UrustJournalApp::new(cc, storage)))),
    )
}

fn generic_icon() -> egui::IconData {
    const SIZE: u32 = 256;
    const BONE: [u8; 4] = [0xEA, 0xE6, 0xDF, 0xFF];
    const TAUPE: [u8; 4] = [0xCF, 0xC2, 0xB4, 0xFF];

    let outer = (44.0, 28.0, 212.0, 228.0, 24.0);
    let inner = (52.0, 36.0, 204.0, 220.0, 18.0);

    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let color = if in_rounded_rect(px, py, inner) {
                Some(BONE)
            } else if in_rounded_rect(px, py, outer) {
                Some(TAUPE)
            } else {
                None
            };
            if let Some(color) = color {
                paint_pixel(&mut rgba, x, y, color);
            }
        }
    }

    for line_y in [92.0, 132.0, 172.0] {
        paint_rect(&mut rgba, 76.0, line_y, 180.0, line_y + 6.0, TAUPE);
    }
    paint_rect(&mut rgba, 76.0, 206.0, 148.0, 212.0, TAUPE);

    egui::IconData {
        rgba,
        width: SIZE,
        height: SIZE,
    }
}

fn in_rounded_rect(px: f32, py: f32, rect: (f32, f32, f32, f32, f32)) -> bool {
    let (min_x, min_y, max_x, max_y, radius) = rect;
    let closest_x = px.clamp(min_x + radius, max_x - radius);
    let closest_y = py.clamp(min_y + radius, max_y - radius);
    let dx = px - closest_x;
    let dy = py - closest_y;
    dx * dx + dy * dy <= radius * radius
}

fn paint_pixel(rgba: &mut [u8], x: u32, y: u32, color: [u8; 4]) {
    let index = ((y * 256 + x) * 4) as usize;
    rgba[index..index + 4].copy_from_slice(&color);
}

fn paint_rect(rgba: &mut [u8], min_x: f32, min_y: f32, max_x: f32, max_y: f32, color: [u8; 4]) {
    for y in min_y as u32..max_y as u32 {
        for x in min_x as u32..max_x as u32 {
            paint_pixel(rgba, x, y, color);
        }
    }
}
