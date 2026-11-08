//! egui presentation layer: journal list, creation and editor.

use eframe::egui;

use crate::journal::{Bullet, Journal, JournalName, JournalStorage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NewBulletKind {
    Task,
    Note,
    Event,
}

impl NewBulletKind {
    fn label(self) -> &'static str {
        match self {
            Self::Task => "Tarea",
            Self::Note => "Nota",
            Self::Event => "Evento",
        }
    }
}

struct Status {
    message: String,
    is_error: bool,
}

/// Top-level application state.
pub struct UrustJournalApp {
    storage: JournalStorage,
    journals: Vec<JournalName>,
    selected: Option<JournalName>,
    current: Option<Journal>,
    new_journal_name: String,
    rename_name: String,
    new_bullet: String,
    new_bullet_kind: NewBulletKind,
    status: Option<Status>,
    dirty: bool,
}

impl UrustJournalApp {
    /// Builds the application and loads the journal list.
    pub fn new(storage: JournalStorage) -> Self {
        let mut app = Self {
            storage,
            journals: Vec::new(),
            selected: None,
            current: None,
            new_journal_name: String::new(),
            rename_name: String::new(),
            new_bullet: String::new(),
            new_bullet_kind: NewBulletKind::Task,
            status: None,
            dirty: false,
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        match self.storage.list() {
            Ok(names) => self.journals = names,
            Err(error) => self.show_error(error),
        }
    }

    fn show_info(&mut self, message: impl Into<String>) {
        self.status = Some(Status {
            message: message.into(),
            is_error: false,
        });
    }

    fn show_error(&mut self, error: impl std::fmt::Display) {
        self.status = Some(Status {
            message: error.to_string(),
            is_error: true,
        });
    }

    fn create_journal(&mut self) {
        let name = match JournalName::new(self.new_journal_name.trim()) {
            Ok(name) => name,
            Err(error) => return self.show_error(error),
        };
        match self.storage.create(name.clone()) {
            Ok(journal) => {
                self.new_journal_name.clear();
                self.rename_name = name.as_str().to_owned();
                self.selected = Some(name.clone());
                self.current = Some(journal);
                self.dirty = false;
                self.show_info(format!("Journal '{name}' creado."));
                self.refresh();
            }
            Err(error) => self.show_error(error),
        }
    }

    fn open_journal(&mut self, name: JournalName) {
        match self.storage.load(&name) {
            Ok(journal) => {
                self.rename_name = name.as_str().to_owned();
                self.selected = Some(name);
                self.current = Some(journal);
                self.dirty = false;
                self.status = None;
            }
            Err(error) => self.show_error(error),
        }
    }

    fn save_journal(&mut self) {
        let result = match self.current.as_ref() {
            Some(journal) => self.storage.save(journal),
            None => return,
        };
        match result {
            Ok(()) => {
                self.dirty = false;
                self.show_info("Journal guardado.");
            }
            Err(error) => self.show_error(error),
        }
    }

    fn delete_selected(&mut self) {
        let Some(name) = self.selected.clone() else {
            return;
        };
        match self.storage.delete(&name) {
            Ok(()) => {
                self.selected = None;
                self.current = None;
                self.rename_name.clear();
                self.dirty = false;
                self.show_info(format!("Journal '{name}' eliminado."));
                self.refresh();
            }
            Err(error) => self.show_error(error),
        }
    }

    fn rename_selected(&mut self) {
        let Some(old_name) = self.selected.clone() else {
            return;
        };
        let new_name = match JournalName::new(self.rename_name.trim()) {
            Ok(name) => name,
            Err(error) => return self.show_error(error),
        };
        if new_name == old_name {
            self.show_info("El nombre no ha cambiado.");
            return;
        }

        if self.dirty {
            let result = match self.current.as_ref() {
                Some(journal) => self.storage.save(journal),
                None => Ok(()),
            };
            if let Err(error) = result {
                return self.show_error(error);
            }
            self.dirty = false;
        }

        match self.storage.rename(&old_name, new_name.clone()) {
            Ok(()) => {
                if let Some(journal) = self.current.as_mut() {
                    journal.rename(new_name.clone());
                }
                self.selected = Some(new_name.clone());
                self.rename_name = new_name.as_str().to_owned();
                self.show_info(format!("Journal renombrado a '{new_name}'."));
                self.refresh();
            }
            Err(error) => self.show_error(error),
        }
    }

    fn add_bullet(&mut self) {
        let text = self.new_bullet.trim();
        if text.is_empty() {
            self.show_error("El texto del bullet no puede estar vacío.");
            return;
        }
        let bullet = match self.new_bullet_kind {
            NewBulletKind::Task => Bullet::task(text),
            NewBulletKind::Note => Bullet::note(text),
            NewBulletKind::Event => Bullet::event(text),
        };
        let Some(journal) = self.current.as_mut() else {
            self.show_error("No hay ningún journal abierto.");
            return;
        };
        journal.push(bullet);
        self.new_bullet.clear();
        self.dirty = true;
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("UrustJournal");
            ui.separator();
            ui.label("Nuevo journal:");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.new_journal_name)
                    .hint_text("mi_journal")
                    .desired_width(180.0),
            );
            let submitted =
                response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            if ui.button("Crear").clicked() || submitted {
                self.create_journal();
            }
        });

        if let Some(status) = &self.status {
            let color = if status.is_error {
                egui::Color32::LIGHT_RED
            } else {
                egui::Color32::LIGHT_GREEN
            };
            ui.colored_label(color, &status.message);
        }
    }

    fn journal_list(&mut self, ui: &mut egui::Ui) {
        ui.heading("Journals");
        ui.separator();

        if self.journals.is_empty() {
            ui.weak("Aún no hay journals.");
            return;
        }

        let mut to_open = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for name in &self.journals {
                let is_selected = self.selected.as_ref() == Some(name);
                if ui.selectable_label(is_selected, name.as_str()).clicked() {
                    to_open = Some(name.clone());
                }
            }
        });

        if let Some(name) = to_open {
            self.open_journal(name);
        }
    }

    fn editor(&mut self, ui: &mut egui::Ui) {
        let Some(name) = self.current.as_ref().map(|journal| journal.name().clone()) else {
            return;
        };

        ui.horizontal(|ui| {
            ui.heading(name.as_str());
            if self.dirty {
                ui.colored_label(egui::Color32::YELLOW, "• sin guardar");
            }
        });

        ui.horizontal(|ui| {
            ui.label("Renombrar:");
            ui.add(
                egui::TextEdit::singleline(&mut self.rename_name)
                    .desired_width(160.0)
                    .hint_text("nuevo_nombre"),
            );
            if ui.button("Renombrar").clicked() {
                self.rename_selected();
            }
            if ui
                .add_enabled(self.dirty, egui::Button::new("Guardar"))
                .clicked()
            {
                self.save_journal();
            }
            if ui.button("Eliminar").clicked() {
                self.delete_selected();
            }
        });

        ui.separator();

        let mut toggle_index = None;
        let mut remove_index = None;
        let bullets_height = (ui.available_height() - 48.0).max(80.0);

        egui::ScrollArea::vertical()
            .max_height(bullets_height)
            .show(ui, |ui| {
                let Some(journal) = self.current.as_ref() else {
                    return;
                };
                if journal.is_empty() {
                    ui.weak("Journal vacío. Añade una tarea, nota o evento.");
                }
                for (index, bullet) in journal.bullets().iter().enumerate() {
                    ui.horizontal(|ui| {
                        match bullet {
                            Bullet::Task { text, completed } => {
                                let mut done = *completed;
                                if ui.checkbox(&mut done, "").changed() {
                                    toggle_index = Some(index);
                                }
                                let mut rich = egui::RichText::new(text);
                                if *completed {
                                    rich = rich.strikethrough().weak();
                                }
                                ui.label(rich);
                            }
                            Bullet::Migrated(text) => {
                                ui.monospace(">");
                                ui.label(text);
                            }
                            Bullet::Scheduled(text) => {
                                ui.monospace("<");
                                ui.label(text);
                            }
                            Bullet::Event(text) => {
                                ui.monospace("o");
                                ui.label(text);
                            }
                            Bullet::Note(text) => {
                                ui.monospace("-");
                                ui.label(text);
                            }
                            Bullet::Raw(text) => {
                                ui.monospace(text);
                            }
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("x").clicked() {
                                remove_index = Some(index);
                            }
                        });
                    });
                }
            });

        if let Some(index) = toggle_index {
            if let Some(journal) = self.current.as_mut() {
                journal.toggle(index);
            }
            self.dirty = true;
        }
        if let Some(index) = remove_index {
            if let Some(journal) = self.current.as_mut() {
                journal.remove(index);
            }
            self.dirty = true;
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Añadir:");
            ui.selectable_value(
                &mut self.new_bullet_kind,
                NewBulletKind::Task,
                NewBulletKind::Task.label(),
            );
            ui.selectable_value(
                &mut self.new_bullet_kind,
                NewBulletKind::Note,
                NewBulletKind::Note.label(),
            );
            ui.selectable_value(
                &mut self.new_bullet_kind,
                NewBulletKind::Event,
                NewBulletKind::Event.label(),
            );
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.new_bullet)
                    .hint_text("Escribe aquí…")
                    .desired_width(300.0),
            );
            let submitted =
                response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            if ui.button("Añadir").clicked() || submitted {
                self.add_bullet();
            }
        });
    }
}

impl eframe::App for UrustJournalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.dirty
            && ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::S))
        {
            self.save_journal();
        }

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| self.top_bar(ui));

        egui::SidePanel::left("journal_list")
            .resizable(true)
            .default_width(220.0)
            .show(ctx, |ui| self.journal_list(ui));

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.current.is_some() {
                self.editor(ui);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.weak("Crea un journal o selecciona uno de la lista.");
                });
            }
        });
    }
}
