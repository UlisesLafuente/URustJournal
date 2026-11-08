//! egui presentation layer: journal list, section tabs and page editor.

use eframe::egui;

use crate::journal::{Bullet, BulletMarker, Journal, JournalName, JournalStorage, SectionKind};

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
    active_section: SectionKind,
    selected_page: Option<usize>,
    new_journal_name: String,
    rename_name: String,
    new_page_title: String,
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
            active_section: SectionKind::Journal,
            selected_page: None,
            new_journal_name: String::new(),
            rename_name: String::new(),
            new_page_title: String::new(),
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

    fn first_page_index(&self, kind: SectionKind) -> Option<usize> {
        let section = self.current.as_ref()?.section(kind);
        if section.is_empty() { None } else { Some(0) }
    }

    fn switch_section(&mut self, kind: SectionKind) {
        if self.active_section == kind {
            return;
        }
        self.active_section = kind;
        self.selected_page = self.first_page_index(kind);
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
                self.active_section = SectionKind::Journal;
                self.selected_page = None;
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
                self.active_section = SectionKind::Journal;
                self.selected_page = self.first_page_index(SectionKind::Journal);
                self.new_page_title.clear();
                self.new_bullet.clear();
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
                self.selected_page = None;
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

    fn create_page(&mut self) {
        let title = self.new_page_title.clone();
        let kind = self.active_section;
        let result = match self.current.as_mut() {
            Some(journal) => journal.add_page(kind, &title),
            None => {
                self.show_error("No hay ningún journal abierto.");
                return;
            }
        };
        match result {
            Ok(index) => {
                self.new_page_title.clear();
                self.selected_page = Some(index);
                self.dirty = true;
                self.show_info(format!("Página '{}' creada.", title.trim()));
            }
            Err(error) => self.show_error(error),
        }
    }

    fn delete_page(&mut self, index: usize) {
        let kind = self.active_section;
        let removed = self
            .current
            .as_mut()
            .and_then(|journal| journal.remove_page(kind, index));
        if removed.is_none() {
            return;
        }
        self.selected_page = match self.selected_page {
            Some(current) if current == index => None,
            Some(current) if current > index => Some(current - 1),
            other => other,
        };
        self.dirty = true;
        self.show_info("Página eliminada.");
    }

    fn add_bullet(&mut self) {
        let Some(page_index) = self.selected_page else {
            self.show_error("Selecciona una página antes de añadir bullets.");
            return;
        };
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
        let kind = self.active_section;
        let Some(page) = self
            .current
            .as_mut()
            .and_then(|journal| journal.page_mut(kind, page_index))
        else {
            self.show_error("La página seleccionada ya no existe.");
            return;
        };
        page.push(bullet);
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

    fn journal_header(&mut self, ui: &mut egui::Ui, name: &JournalName) {
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
    }

    fn section_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for kind in SectionKind::ALL {
                if ui
                    .selectable_label(self.active_section == kind, kind.label())
                    .clicked()
                {
                    self.switch_section(kind);
                }
            }
        });
    }

    fn page_bar(&mut self, ui: &mut egui::Ui) {
        let kind = self.active_section;
        let mut select_page = None;
        let mut delete_page = None;

        ui.horizontal_wrapped(|ui| {
            if let Some(journal) = self.current.as_ref() {
                let section = journal.section(kind);
                if section.is_empty() {
                    ui.weak("Sin páginas en esta sección.");
                }
                for (index, page) in section.pages().iter().enumerate() {
                    let is_selected = self.selected_page == Some(index);
                    if ui.selectable_label(is_selected, page.title()).clicked() {
                        select_page = Some(index);
                    }
                    if ui
                        .small_button("x")
                        .on_hover_text("Eliminar página")
                        .clicked()
                    {
                        delete_page = Some(index);
                    }
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Nueva página:");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.new_page_title)
                    .hint_text("2026-11-08")
                    .desired_width(180.0),
            );
            let submitted =
                response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            if ui.button("Crear página").clicked() || submitted {
                self.create_page();
            }
        });

        if let Some(index) = select_page {
            self.selected_page = Some(index);
        }
        if let Some(index) = delete_page {
            self.delete_page(index);
        }
    }

    fn page_editor(&mut self, ui: &mut egui::Ui) {
        let kind = self.active_section;
        let Some(page_index) = self.selected_page else {
            ui.weak("Selecciona o crea una página en esta sección.");
            return;
        };

        let Some(title) = self
            .current
            .as_ref()
            .and_then(|journal| journal.page(kind, page_index))
            .map(|page| page.title().to_owned())
        else {
            self.selected_page = None;
            return;
        };

        ui.heading(title);

        let mut marker_change = None;
        let mut remove_index = None;
        let bullets_height = (ui.available_height() - 48.0).max(80.0);

        egui::ScrollArea::vertical()
            .max_height(bullets_height)
            .show(ui, |ui| {
                let Some(page) = self
                    .current
                    .as_ref()
                    .and_then(|journal| journal.page(kind, page_index))
                else {
                    return;
                };
                if page.is_empty() {
                    ui.weak("Página vacía. Añade una tarea, nota o evento.");
                }
                for (index, bullet) in page.bullets().iter().enumerate() {
                    ui.horizontal(|ui| {
                        match bullet {
                            Bullet::Raw(text) => {
                                ui.monospace(text);
                            }
                            _ => {
                                if let Some(current_marker) = bullet.marker() {
                                    ui.menu_button(current_marker.symbol(), |ui| {
                                        for option in BulletMarker::ALL {
                                            let selected = option == current_marker;
                                            let label =
                                                format!("{}  {}", option.symbol(), option.label());
                                            if ui.selectable_label(selected, label).clicked() {
                                                marker_change = Some((index, option));
                                                ui.close();
                                            }
                                        }
                                    })
                                    .response
                                    .on_hover_text("Cambiar marcador");
                                }

                                let mut rich = egui::RichText::new(bullet.text());
                                if bullet.marker() == Some(BulletMarker::Completed) {
                                    rich = rich.strikethrough().weak();
                                }
                                ui.label(rich);
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

        if let Some((index, marker)) = marker_change {
            if let Some(page) = self
                .current
                .as_mut()
                .and_then(|journal| journal.page_mut(kind, page_index))
                && let Some(bullet) = page.bullet_mut(index)
            {
                bullet.set_marker(marker);
            }
            self.dirty = true;
        }
        if let Some(index) = remove_index {
            if let Some(page) = self
                .current
                .as_mut()
                .and_then(|journal| journal.page_mut(kind, page_index))
            {
                page.remove(index);
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

    fn editor(&mut self, ui: &mut egui::Ui) {
        let Some(name) = self.current.as_ref().map(|journal| journal.name().clone()) else {
            return;
        };

        self.journal_header(ui, &name);
        ui.separator();
        self.section_tabs(ui);
        self.page_bar(ui);
        ui.separator();
        self.page_editor(ui);
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
