//! egui presentation layer: journal list, section tabs and page editor.

use eframe::egui;

use crate::journal::{
    AnnotationColor, Bullet, BulletMarker, Journal, JournalName, JournalStorage, SectionKind,
};

const CAVEAT: &str = "caveat";
const SCRIPT_TEXT_STYLE: &str = "Script";
const SCRIPT_SIZE: f32 = 16.0;
const SCRIPT_HEADING_SIZE: f32 = 24.0;
const MARKER_SIZE: f32 = 16.0;

const ERROR_INK: egui::Color32 = egui::Color32::from_rgb(0x8A, 0x3B, 0x36);
const SUCCESS_INK: egui::Color32 = egui::Color32::from_rgb(0x3E, 0x5C, 0x46);
const PENDING_INK: egui::Color32 = egui::Color32::from_rgb(0x8A, 0x6D, 0x2F);
const DESATURATED_BLUE: egui::Color32 = egui::Color32::from_rgb(0x6E, 0x86, 0xA8);
const EMERALD_GREEN: egui::Color32 = egui::Color32::from_rgb(0x3E, 0x8A, 0x63);
const RUST_RED: egui::Color32 = egui::Color32::from_rgb(0xA6, 0x58, 0x4E);

fn annotation_color(color: AnnotationColor, theme: Theme) -> egui::Color32 {
    let ink = theme.palette().ink;
    match color {
        AnnotationColor::Black => ink,
        AnnotationColor::DesaturatedBlue => DESATURATED_BLUE,
        AnnotationColor::EmeraldGreen => EMERALD_GREEN,
        AnnotationColor::RustRed => RUST_RED,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Theme {
    #[default]
    PapelAmarillento,
    AzulNordico,
    BosqueAntiguo,
    SuenoVioleta,
}

impl Theme {
    const ALL: [Self; 4] = [
        Self::PapelAmarillento,
        Self::AzulNordico,
        Self::BosqueAntiguo,
        Self::SuenoVioleta,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::PapelAmarillento => "Papel amarillento",
            Self::AzulNordico => "Azul nórdico",
            Self::BosqueAntiguo => "Bosque antiguo",
            Self::SuenoVioleta => "Sueño violeta",
        }
    }

    fn palette(self) -> ThemePalette {
        match self {
            Self::PapelAmarillento => ThemePalette {
                bg: egui::Color32::from_rgb(0xEA, 0xE6, 0xDF),
                bg_alt: egui::Color32::from_rgb(0xD5, 0xD6, 0xD2),
                accent_soft: egui::Color32::from_rgb(0xE2, 0xD9, 0xC5),
                accent: egui::Color32::from_rgb(0xCF, 0xC2, 0xB4),
                ink: egui::Color32::from_rgb(0x3B, 0x35, 0x2F),
                dark_mode: false,
            },
            Self::AzulNordico => ThemePalette {
                bg: egui::Color32::from_rgb(0x07, 0x11, 0x1F),
                bg_alt: egui::Color32::from_rgb(0x0B, 0x3D, 0x6E),
                accent_soft: egui::Color32::from_rgb(0x1D, 0x8A, 0xCB),
                accent: egui::Color32::from_rgb(0x60, 0xD6, 0xFF),
                ink: egui::Color32::from_rgb(0xB8, 0xF3, 0xFF),
                dark_mode: true,
            },
            Self::BosqueAntiguo => ThemePalette {
                bg: egui::Color32::from_rgb(0x08, 0x14, 0x0F),
                bg_alt: egui::Color32::from_rgb(0x1E, 0x4D, 0x35),
                accent_soft: egui::Color32::from_rgb(0x3F, 0xA8, 0x6D),
                accent: egui::Color32::from_rgb(0x9A, 0xD9, 0x8B),
                ink: egui::Color32::from_rgb(0xE1, 0xF0, 0xB5),
                dark_mode: true,
            },
            Self::SuenoVioleta => ThemePalette {
                bg: egui::Color32::from_rgb(0x0E, 0x07, 0x19),
                bg_alt: egui::Color32::from_rgb(0x5A, 0x2C, 0xFF),
                accent_soft: egui::Color32::from_rgb(0x9B, 0x5C, 0xFF),
                accent: egui::Color32::from_rgb(0xF2, 0x5C, 0xFF),
                ink: egui::Color32::from_rgb(0x58, 0xF2, 0xD7),
                dark_mode: true,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ThemePalette {
    bg: egui::Color32,
    bg_alt: egui::Color32,
    accent_soft: egui::Color32,
    accent: egui::Color32,
    ink: egui::Color32,
    dark_mode: bool,
}

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

fn configure_style(ctx: &egui::Context, theme: Theme) {
    let p = theme.palette();
    let mut style = (*ctx.style()).clone();
    style
        .text_styles
        .insert(script_text_style(), script_font(SCRIPT_SIZE));
    let visuals = &mut style.visuals;

    visuals.dark_mode = p.dark_mode;
    visuals.override_text_color = Some(p.ink);
    visuals.panel_fill = p.bg;
    visuals.window_fill = p.bg;
    visuals.faint_bg_color = p.bg_alt;
    visuals.extreme_bg_color = p.bg_alt;
    visuals.text_edit_bg_color = Some(p.bg);
    visuals.code_bg_color = p.bg_alt;
    visuals.selection.bg_fill = p.accent_soft;
    visuals.selection.stroke = egui::Stroke::new(1.0, p.ink);
    visuals.hyperlink_color = p.accent;
    visuals.window_stroke = egui::Stroke::new(1.0, p.accent);
    visuals.window_corner_radius = egui::CornerRadius::same(6);
    visuals.menu_corner_radius = egui::CornerRadius::same(6);
    visuals.text_cursor.stroke = egui::Stroke::new(1.5, p.ink);

    style_widget(&mut visuals.widgets.noninteractive, p.bg, p.accent, p.ink);
    style_widget(&mut visuals.widgets.inactive, p.bg_alt, p.accent, p.ink);
    style_widget(&mut visuals.widgets.hovered, p.accent_soft, p.accent, p.ink);
    style_widget(&mut visuals.widgets.active, p.accent, p.bg, p.bg);
    style_widget(&mut visuals.widgets.open, p.accent_soft, p.accent, p.ink);

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

fn hint(text: &str, theme: Theme) -> egui::RichText {
    egui::RichText::new(text).color(theme.palette().accent)
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

fn collapse_arrow(
    ui: &mut egui::Ui,
    expanded: &mut bool,
    open_symbol: &str,
    closed_symbol: &str,
    color: egui::Color32,
) {
    let symbol = if *expanded {
        open_symbol
    } else {
        closed_symbol
    };
    if ui
        .add(egui::Button::new(egui::RichText::new(symbol).color(color)).frame(false))
        .clicked()
    {
        *expanded = !*expanded;
    }
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
    show_journal_list: bool,
    show_create_bar: bool,
    show_sections_pages: bool,
    show_annotations: bool,
    editing_bullet: Option<(usize, String)>,
    theme: Theme,
}

impl UrustJournalApp {
    /// Builds the application and loads the journal list.
    pub fn new(cc: &eframe::CreationContext<'_>, storage: JournalStorage) -> Self {
        let theme = Theme::default();
        install_fonts(&cc.egui_ctx);
        configure_style(&cc.egui_ctx, theme);

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
            show_journal_list: true,
            show_create_bar: true,
            show_sections_pages: true,
            show_annotations: true,
            editing_bullet: None,
            theme,
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

    fn top_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let arrow_color = self.theme.palette().accent;
        ui.horizontal(|ui| {
            collapse_arrow(ui, &mut self.show_create_bar, "▲", "▼", arrow_color);
            ui.heading("UrustJournal");
            if self.show_create_bar {
                ui.separator();
                ui.label("Nuevo journal:");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.new_journal_name)
                        .font(script_font(SCRIPT_SIZE))
                        .hint_text(hint("mi_journal", self.theme))
                        .desired_width(180.0),
                );
                let submitted =
                    response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
                if ui.button("Crear").clicked() || submitted {
                    self.create_journal();
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Acerca de").clicked() {
                    self.show_about = true;
                }
                ui.separator();
                ui.label("Tema:");
                let mut changed = false;
                egui::ComboBox::from_id_salt("theme_select")
                    .selected_text(self.theme.label())
                    .show_ui(ui, |ui| {
                        for theme in Theme::ALL {
                            if ui
                                .selectable_value(&mut self.theme, theme, theme.label())
                                .changed()
                            {
                                changed = true;
                            }
                        }
                    });
                if changed {
                    configure_style(ctx, self.theme);
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
                    .hint_text(hint("nuevo_nombre", self.theme)),
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
                    .hint_text(hint("2026-11-08", self.theme))
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

    fn page_editor(&mut self, ui: &mut egui::Ui, show_annotations: bool) {
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
        let mut color_change = None;
        let mut text_change = None;
        let mut edit_start = None;

        if show_annotations {
            let bullets_height = (ui.available_height() - 60.0).max(80.0);

            let p = self.theme.palette();
            egui::Frame::new()
                .fill(p.bg)
                .stroke(egui::Stroke::new(1.5, p.accent))
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
                            let editing_index = self.editing_bullet.as_ref().map(|(i, _)| *i);
                            for (index, bullet) in page.bullets().iter().enumerate() {
                                let is_editing = editing_index == Some(index);
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
                                                            .color(self.theme.palette().ink),
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

                                            if is_editing {
                                                if let Some((_, ref mut edit_text)) =
                                                    self.editing_bullet
                                                {
                                                    let response = ui.add(
                                                        egui::TextEdit::singleline(edit_text)
                                                            .font(script_font(SCRIPT_SIZE))
                                                            .desired_width(280.0),
                                                    );
                                                    let submitted = response.lost_focus()
                                                        && ui.input(|i| {
                                                            i.key_pressed(egui::Key::Enter)
                                                        });
                                                    if submitted
                                                        || ui.input(|i| {
                                                            i.key_pressed(egui::Key::Escape)
                                                        })
                                                    {
                                                        if submitted {
                                                            text_change =
                                                                Some((index, edit_text.clone()));
                                                        }
                                                        self.editing_bullet = None;
                                                    }
                                                }
                                            } else {
                                                let mut rich = egui::RichText::new(bullet.text())
                                                    .text_style(script_text_style())
                                                    .color(annotation_color(
                                                        bullet.color(),
                                                        self.theme,
                                                    ));
                                                if bullet.marker() == Some(BulletMarker::Completed)
                                                {
                                                    rich = rich.strikethrough().weak();
                                                }
                                                let response = ui
                                                    .add(
                                                        egui::Label::new(rich)
                                                            .sense(egui::Sense::click()),
                                                    )
                                                    .on_hover_text("Clic para opciones");
                                                egui::Popup::menu(&response).show(|ui| {
                                                    if ui.button("Editar texto").clicked() {
                                                        edit_start =
                                                            Some((index, bullet.text().to_owned()));
                                                        ui.close();
                                                    }
                                                    ui.separator();
                                                    ui.label("Color del texto:");
                                                    for color in AnnotationColor::ALL {
                                                        let selected = color == bullet.color();
                                                        let label =
                                                            egui::RichText::new(color.label())
                                                                .color(annotation_color(
                                                                    color, self.theme,
                                                                ));
                                                        if ui
                                                            .selectable_label(selected, label)
                                                            .clicked()
                                                        {
                                                            color_change = Some((index, color));
                                                            ui.close();
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                    }
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if is_editing {
                                                if ui.small_button("✓").clicked() {
                                                    if let Some((_, ref t)) = self.editing_bullet {
                                                        text_change = Some((index, t.clone()));
                                                    }
                                                    self.editing_bullet = None;
                                                }
                                            } else if ui.small_button("x").clicked() {
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
                                    egui::Stroke::new(1.0, self.theme.palette().accent),
                                );
                                ui.add_space(3.0);
                            }
                        });
                });
        }

        if let Some((index, text)) = edit_start {
            self.editing_bullet = Some((index, text));
        }

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
        if let Some((index, color)) = color_change {
            if let Some(page) = self
                .current
                .as_mut()
                .and_then(|journal| journal.page_mut(kind, page_index))
                && let Some(bullet) = page.bullet_mut(index)
            {
                bullet.set_color(color);
            }
            self.dirty = true;
        }
        if let Some((index, text)) = text_change {
            if let Some(page) = self
                .current
                .as_mut()
                .and_then(|journal| journal.page_mut(kind, page_index))
                && let Some(bullet) = page.bullet_mut(index)
            {
                bullet.set_text(text);
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
            if self
                .editing_bullet
                .as_ref()
                .is_some_and(|(i, _)| *i == index)
            {
                self.editing_bullet = None;
            }
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
                    .hint_text(hint("Escribe aquí…", self.theme))
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
        ui.horizontal(|ui| {
            collapse_arrow(
                ui,
                &mut self.show_sections_pages,
                "▲",
                "▼",
                self.theme.palette().accent,
            );
            ui.label("Secciones y páginas");
        });
        if self.show_sections_pages {
            self.section_tabs(ui);
            self.page_bar(ui);
        }

        ui.separator();
        ui.horizontal(|ui| {
            collapse_arrow(
                ui,
                &mut self.show_annotations,
                "▲",
                "▼",
                self.theme.palette().accent,
            );
            ui.label("Anotaciones");
        });
        self.page_editor(ui, self.show_annotations);
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
                ui.label("© Ulises Lafuente Ramos - 2026");
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

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| self.top_bar(ui, ctx));

        if self.show_journal_list {
            egui::SidePanel::left("journal_list")
                .resizable(true)
                .default_width(220.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Journals");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            collapse_arrow(
                                ui,
                                &mut self.show_journal_list,
                                "◀",
                                "▶",
                                self.theme.palette().accent,
                            );
                        });
                    });
                    ui.separator();
                    self.journal_list(ui);
                });
        } else {
            egui::SidePanel::left("journal_list_collapsed")
                .resizable(false)
                .exact_width(28.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(6.0);
                        collapse_arrow(
                            ui,
                            &mut self.show_journal_list,
                            "◀",
                            "▶",
                            self.theme.palette().accent,
                        );
                    });
                });
        }

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
        configure_style(&ctx, Theme::default());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("Tarea de prueba: ¿ñáé?").text_style(script_text_style()),
                );
            });
        });
    }
}
