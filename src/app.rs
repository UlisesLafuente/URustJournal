//! egui presentation layer: journal list, section tabs and page editor.

use eframe::egui;

use crate::journal::{Bullet, BulletMarker, Journal, JournalName, JournalStorage, SectionKind};

const CAVEAT: &str = "caveat";
const SCRIPT_TEXT_STYLE: &str = "Script";
const SCRIPT_SIZE: f32 = 16.0;
const SCRIPT_HEADING_SIZE: f32 = 24.0;
const MARKER_SIZE: f32 = 16.0;

const BONE: egui::Color32 = egui::Color32::from_rgb(0xEA, 0xE6, 0xDF);
const ASH: egui::Color32 = egui::Color32::from_rgb(0xD5, 0xD6, 0xD2);
const OAT: egui::Color32 = egui::Color32::from_rgb(0xE2, 0xD9, 0xC5);
const TAUPE: egui::Color32 = egui::Color32::from_rgb(0xCF, 0xC2, 0xB4);
const INK: egui::Color32 = egui::Color32::from_rgb(0x3B, 0x35, 0x2F);
const ERROR_INK: egui::Color32 = egui::Color32::from_rgb(0x8A, 0x3B, 0x36);
const SUCCESS_INK: egui::Color32 = egui::Color32::from_rgb(0x3E, 0x5C, 0x46);
const PENDING_INK: egui::Color32 = egui::Color32::from_rgb(0x8A, 0x6D, 0x2F);

fn script_family() -> egui::FontFamily {
    egui::FontFamily::Name("script".into())
}

fn script_text_style() -> egui::TextStyle {
    egui::TextStyle::Name(SCRIPT_TEXT_STYLE.into())
}

fn script_font(size: f32) -> egui::FontId {
    egui::FontId::new(size, script_family())
}

fn script_text(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text).text_style(script_text_style())
}

fn script_heading(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text).font(script_font(SCRIPT_HEADING_SIZE))
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        CAVEAT.to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/Caveat[wght].ttf"
        ))),
    );

    let mut script_fonts = vec![CAVEAT.to_owned()];
    if let Some(fallbacks) = fonts.families.get(&egui::FontFamily::Proportional) {
        script_fonts.extend(fallbacks.iter().cloned());
    }
    fonts.families.insert(script_family(), script_fonts);

    ctx.set_fonts(fonts);
}

fn configure_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style
        .text_styles
        .insert(script_text_style(), script_font(SCRIPT_SIZE));
    let visuals = &mut style.visuals;

    visuals.dark_mode = false;
    visuals.override_text_color = Some(INK);
    visuals.panel_fill = BONE;
    visuals.window_fill = BONE;
    visuals.faint_bg_color = ASH;
    visuals.extreme_bg_color = ASH;
    visuals.text_edit_bg_color = Some(BONE);
    visuals.code_bg_color = ASH;
    visuals.selection.bg_fill = OAT;
    visuals.selection.stroke = egui::Stroke::new(1.0, INK);
    visuals.hyperlink_color = INK;
    visuals.window_stroke = egui::Stroke::new(1.0, TAUPE);
    visuals.window_corner_radius = egui::CornerRadius::same(6);
    visuals.menu_corner_radius = egui::CornerRadius::same(6);
    visuals.text_cursor.stroke = egui::Stroke::new(1.5, INK);

    style_widget(&mut visuals.widgets.noninteractive, BONE, TAUPE, INK);
    style_widget(&mut visuals.widgets.inactive, ASH, TAUPE, INK);
    style_widget(&mut visuals.widgets.hovered, OAT, TAUPE, INK);
    style_widget(&mut visuals.widgets.active, TAUPE, INK, INK);
    style_widget(&mut visuals.widgets.open, OAT, TAUPE, INK);

    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);

    ctx.set_style(style);
}

fn style_widget(
    widget: &mut egui::style::WidgetVisuals,
    fill: egui::Color32,
    border: egui::Color32,
    text: egui::Color32,
) {
    widget.bg_fill = fill;
    widget.weak_bg_fill = fill;
    widget.bg_stroke = egui::Stroke::new(1.0, border);
    widget.fg_stroke = egui::Stroke::new(1.0, text);
}

fn hint(text: &str) -> egui::RichText {
    egui::RichText::new(text).color(TAUPE)
}

fn license_block(ui: &mut egui::Ui, text: &str, monospace: bool) {
    egui::ScrollArea::vertical()
        .max_height(220.0)
        .show(ui, |ui| {
            let mut rich = egui::RichText::new(text).size(11.0);
            if monospace {
                rich = rich.monospace();
            }
            ui.add(egui::Label::new(rich).selectable(true));
        });
}

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
    show_about: bool,
}

impl UrustJournalApp {
    /// Builds the application and loads the journal list.
    pub fn new(cc: &eframe::CreationContext<'_>, storage: JournalStorage) -> Self {
        install_fonts(&cc.egui_ctx);
        configure_style(&cc.egui_ctx);

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
            show_about: false,
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
                    .font(script_font(SCRIPT_SIZE))
                    .hint_text(hint("mi_journal"))
                    .desired_width(180.0),
            );
            let submitted =
                response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            if ui.button("Crear").clicked() || submitted {
                self.create_journal();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Acerca de").clicked() {
                    self.show_about = true;
                }
            });
        });

        if let Some(status) = &self.status {
            let color = if status.is_error {
                ERROR_INK
            } else {
                SUCCESS_INK
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
                if ui
                    .selectable_label(is_selected, script_text(name.as_str()))
                    .clicked()
                {
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
            ui.heading(script_heading(name.as_str()));
            if self.dirty {
                ui.colored_label(PENDING_INK, "• sin guardar");
            }
        });

        ui.horizontal(|ui| {
            ui.label("Renombrar:");
            ui.add(
                egui::TextEdit::singleline(&mut self.rename_name)
                    .font(script_font(SCRIPT_SIZE))
                    .desired_width(160.0)
                    .hint_text(hint("nuevo_nombre")),
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
                    .selectable_label(self.active_section == kind, script_text(kind.label()))
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
                    if ui
                        .selectable_label(is_selected, script_text(page.title()))
                        .clicked()
                    {
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
                    .font(script_font(SCRIPT_SIZE))
                    .hint_text(hint("2026-11-08"))
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

        ui.heading(script_heading(title));

        let mut marker_change = None;
        let mut remove_index = None;
        let bullets_height = (ui.available_height() - 60.0).max(80.0);

        egui::Frame::new()
            .fill(BONE)
            .stroke(egui::Stroke::new(1.5, TAUPE))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
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
                            let row = ui.horizontal(|ui| {
                                match bullet {
                                    Bullet::Raw(text) => {
                                        ui.monospace(text);
                                    }
                                    _ => {
                                        if let Some(current_marker) = bullet.marker() {
                                            let response = ui
                                                .add(
                                                    egui::Label::new(
                                                        egui::RichText::new(
                                                            current_marker.symbol(),
                                                        )
                                                        .size(MARKER_SIZE)
                                                        .color(INK),
                                                    )
                                                    .sense(egui::Sense::click()),
                                                )
                                                .on_hover_text("Cambiar marcador");

                                            egui::Popup::menu(&response).show(|ui| {
                                                for option in BulletMarker::ALL {
                                                    let selected = option == current_marker;
                                                    let label = format!(
                                                        "{}  {}",
                                                        option.symbol(),
                                                        option.label()
                                                    );
                                                    if ui
                                                        .selectable_label(selected, label)
                                                        .clicked()
                                                    {
                                                        marker_change = Some((index, option));
                                                        ui.close();
                                                    }
                                                }
                                            });
                                        }

                                        let mut rich = egui::RichText::new(bullet.text())
                                            .text_style(script_text_style());
                                        if bullet.marker() == Some(BulletMarker::Completed) {
                                            rich = rich.strikethrough().weak();
                                        }
                                        ui.label(rich);
                                    }
                                }
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.small_button("x").clicked() {
                                            remove_index = Some(index);
                                        }
                                    },
                                );
                            });

                            let line_y = row.response.rect.bottom() + 3.0;
                            let line_left = row.response.rect.left();
                            let line_right = ui.max_rect().right();
                            ui.painter().line_segment(
                                [
                                    egui::pos2(line_left, line_y),
                                    egui::pos2(line_right, line_y),
                                ],
                                egui::Stroke::new(1.0, TAUPE),
                            );
                            ui.add_space(3.0);
                        }
                    });
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
                    .font(script_font(SCRIPT_SIZE))
                    .hint_text(hint("Escribe aquí…"))
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

    fn about_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_about;
        egui::Window::new("Acerca de UrustJournal")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .show(ctx, |ui| {
                ui.heading(script_heading("UrustJournal"));
                ui.label(format!("Versión {}", env!("CARGO_PKG_VERSION")));
                ui.label(env!("CARGO_PKG_DESCRIPTION"));
                ui.label(format!("© Ulises Lafuente Ramos - 2026"));
                ui.label(format!("Licencia: {}", env!("CARGO_PKG_LICENSE")));

                let repository = env!("CARGO_PKG_REPOSITORY");
                if !repository.is_empty() {
                    ui.hyperlink_to("Repositorio", repository);
                }

                ui.separator();
                ui.collapsing("Licencia MIT", |ui| {
                    license_block(ui, include_str!("../LICENSE-MIT"), false);
                });
                ui.collapsing("Licencia Apache 2.0", |ui| {
                    license_block(ui, include_str!("../LICENSE-APACHE"), false);
                });
                ui.collapsing("Fuente Caveat (SIL OFL 1.1)", |ui| {
                    license_block(ui, include_str!("../assets/fonts/OFL-Caveat.txt"), false);
                });
                ui.collapsing("Dependencias de Rust", |ui| {
                    license_block(ui, include_str!("../assets/THIRD_PARTY_LICENSES.txt"), true);
                });
            });
        self.show_about = open;
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

        self.about_window(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_font_loads_and_lays_out_text() {
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        configure_style(&ctx);

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("Tarea de prueba: ¿ñáé?").text_style(script_text_style()),
                );
            });
        });
    }
}
