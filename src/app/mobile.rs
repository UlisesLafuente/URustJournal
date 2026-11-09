//! Touch-first, single-column interface for phones.
//!
//! The desktop UI lives in `src/app.rs`; both layouts share the same
//! [`super::UJournalApp`] state and actions. This module only decides how the
//! state is presented and which touch gestures trigger each action.

use eframe::egui;

use super::{
    AnnotationColor, Bullet, BulletMarker, JournalName, Lang, MARKER_SIZE, NewBulletKind,
    SectionKind, Theme, annotation_color, color_label, hint, marker_label, note_font, note_heading,
    note_text, script_text, section_label,
};
use crate::journal::Journal;

const TOUCH: f32 = 48.0;
pub(super) const PHONE_BREAKPOINT: f32 = 640.0;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum MobileScreen {
    #[default]
    Journals,
    Journal,
}

#[derive(Debug, Clone)]
pub(super) enum MobileOverlay {
    JournalMenu(JournalName),
    PageMenu(usize),
    BulletMenu(usize),
    MarkerPicker(usize),
    NewPage,
    Settings,
    About,
}

#[derive(Debug, Clone)]
pub(super) enum MobileConfirm {
    Journal(JournalName),
    Page(usize),
    Bullet(usize),
}

#[derive(Debug, Default)]
pub(super) struct MobileState {
    pub screen: MobileScreen,
    pub overlay: Option<MobileOverlay>,
    pub confirm: Option<MobileConfirm>,
    pub focus_pending: bool,
}

impl super::UJournalApp {
    pub(super) fn mobile_update(&mut self, ctx: &egui::Context) {
        self.mobile_back(ctx);

        if self.current.is_none() {
            self.mobile.screen = MobileScreen::Journals;
        }

        if self.mobile.screen == MobileScreen::Journals {
            self.mobile_journals_screen(ctx);
        } else {
            self.mobile_journal_screen(ctx);
        }

        self.mobile_overlays(ctx);
        self.mobile_snackbar(ctx);
    }

    fn mobile_back(&mut self, ctx: &egui::Context) {
        let back = ctx
            .input(|i| i.key_pressed(egui::Key::BrowserBack) || i.key_pressed(egui::Key::Escape));
        if !back {
            return;
        }

        if self.editing_bullet.is_some() {
            self.editing_bullet = None;
            return;
        }
        if self.mobile.confirm.is_some() {
            self.mobile.confirm = None;
            return;
        }
        if self.mobile.overlay.is_some() {
            self.mobile.overlay = None;
            return;
        }
        if self.creating_journal {
            self.creating_journal = false;
            return;
        }
        if self.renaming_journal.is_some() {
            self.renaming_journal = None;
            return;
        }
        if self.mobile.screen == MobileScreen::Journal {
            self.mobile.screen = MobileScreen::Journals;
            return;
        }
        if cfg!(target_os = "android") {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn mobile_title(&self) -> String {
        if self.mobile.screen == MobileScreen::Journal {
            self.current
                .as_ref()
                .map(|journal| journal.name().as_str().to_owned())
                .unwrap_or_else(|| "UJournal".to_owned())
        } else {
            self.lang.t("journals").to_owned()
        }
    }

    fn mobile_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("mobile_top")
            .exact_height(52.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if self.mobile.screen == MobileScreen::Journal {
                        if ui
                            .add_sized([TOUCH, TOUCH], egui::Button::new("←"))
                            .on_hover_text(self.lang.t("journals"))
                            .clicked()
                        {
                            self.mobile.screen = MobileScreen::Journals;
                        }
                    } else {
                        ui.add_space(8.0);
                    }

                    let title = self.mobile_title();
                    ui.add(egui::Label::new(note_heading(title, self.lang)).truncate());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_sized([TOUCH, TOUCH], egui::Button::new("⋯"))
                            .on_hover_text(self.lang.t("options"))
                            .clicked()
                        {
                            self.mobile.overlay = Some(MobileOverlay::Settings);
                        }
                    });
                });
            });

        if self.status.is_some() {
            egui::TopBottomPanel::top("mobile_status").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(status) = &self.status {
                        let color = if status.is_error {
                            super::ERROR_INK
                        } else {
                            super::SUCCESS_INK
                        };
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(&status.message).color(color).size(13.0),
                            )
                            .truncate(),
                        );
                    }
                    if ui
                        .add_sized([32.0, 28.0], egui::Button::new("×").frame(false))
                        .clicked()
                    {
                        self.status = None;
                    }
                });
            });
        }
    }

    fn mobile_journals_screen(&mut self, ctx: &egui::Context) {
        self.mobile_top_bar(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if self.journals.is_empty() {
                        ui.add_space(24.0);
                        ui.label(note_text(self.lang.t("no_journals"), self.lang));
                        ui.add_space(12.0);
                    }

                    let names = self.journals.clone();
                    for name in names {
                        let is_selected = self.selected.as_ref() == Some(&name);
                        ui.horizontal(|ui| {
                            let width = (ui.available_width() - TOUCH - 8.0).max(80.0);
                            let response = ui.add_sized(
                                [width, TOUCH],
                                egui::Button::new(note_text(name.as_str(), self.lang))
                                    .selected(is_selected),
                            );
                            if response.clicked() {
                                self.open_journal(name.clone());
                                self.mobile.screen = MobileScreen::Journal;
                            } else if response.long_touched() {
                                self.mobile.overlay =
                                    Some(MobileOverlay::JournalMenu(name.clone()));
                            }

                            let menu = ui.add_sized([TOUCH, TOUCH], egui::Button::new("⋯"));
                            menu.clone().on_hover_text(self.lang.t("options"));
                            if menu.clicked() {
                                self.mobile.overlay =
                                    Some(MobileOverlay::JournalMenu(name.clone()));
                            }
                        });
                        ui.add_space(4.0);
                    }

                    ui.add_space(12.0);
                    if ui
                        .add_sized(
                            [ui.available_width(), 56.0],
                            egui::Button::new(script_text(format!(
                                "＋ {}",
                                self.lang.t("new_journal")
                            ))),
                        )
                        .clicked()
                    {
                        self.new_journal_name.clear();
                        self.creating_journal = true;
                        self.mobile.focus_pending = true;
                    }
                });
        });
    }

    fn mobile_journal_screen(&mut self, ctx: &egui::Context) {
        self.mobile_top_bar(ctx);
        self.mobile_sections(ctx);
        self.mobile_pages(ctx);
        self.mobile_composer(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(page_index) = self.selected_page else {
                ui.add_space(24.0);
                ui.vertical_centered(|ui| {
                    ui.label(note_text(self.lang.t("select_page"), self.lang));
                    ui.add_space(12.0);
                    if ui
                        .add_sized(
                            [240.0, 52.0],
                            egui::Button::new(script_text(self.lang.t("create_page"))),
                        )
                        .clicked()
                    {
                        self.new_page_title.clear();
                        self.mobile.overlay = Some(MobileOverlay::NewPage);
                        self.mobile.focus_pending = true;
                    }
                });
                return;
            };

            let kind = self.active_section;
            let Some(page) = self
                .current
                .as_ref()
                .and_then(|journal| journal.page(kind, page_index))
            else {
                self.selected_page = None;
                return;
            };
            let title = page.title().to_owned();
            let bullets = page.bullets().to_vec();
            let mut text_change: Option<(usize, String)> = None;

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(6.0);
                    ui.add(
                        egui::Label::new(note_heading(title, self.lang))
                            .wrap()
                            .truncate(),
                    );
                    ui.add_space(8.0);

                    if bullets.is_empty() {
                        ui.label(note_text(self.lang.t("empty_page"), self.lang));
                    }

                    let editing_index = self.editing_bullet.as_ref().map(|(i, _)| *i);
                    for (index, bullet) in bullets.iter().enumerate() {
                        let is_editing = editing_index == Some(index);
                        let row = ui.horizontal(|ui| {
                            if let Bullet::Raw(raw) = bullet {
                                ui.add(
                                    egui::Label::new(egui::RichText::new(raw).monospace()).wrap(),
                                );
                                return;
                            }

                            if let Some(marker) = bullet.marker() {
                                let response = ui.add_sized(
                                    [TOUCH, TOUCH],
                                    egui::Button::new(
                                        egui::RichText::new(marker.symbol())
                                            .size(MARKER_SIZE + 6.0)
                                            .color(self.theme.palette().ink),
                                    )
                                    .frame(false),
                                );
                                if response.clicked() {
                                    self.mobile.overlay = Some(MobileOverlay::MarkerPicker(index));
                                }
                            }

                            let remaining = (ui.available_width() - TOUCH - 4.0).max(80.0);
                            if is_editing {
                                if let Some((_, ref mut edit_text)) = self.editing_bullet {
                                    let id = edit_bullet_id(kind, page_index, index);
                                    let response = ui.add_sized(
                                        [remaining, TOUCH],
                                        egui::TextEdit::singleline(edit_text)
                                            .id(id)
                                            .font(note_font(18.0, self.lang)),
                                    );
                                    let submitted = response.lost_focus()
                                        && ui.input(|i| i.key_pressed(egui::Key::Enter));
                                    if submitted {
                                        text_change = Some((index, edit_text.clone()));
                                        self.editing_bullet = None;
                                    }
                                }
                            } else {
                                let mut rich = egui::RichText::new(bullet.text())
                                    .font(note_font(18.0, self.lang))
                                    .color(annotation_color(bullet.color(), self.theme));
                                if bullet.marker() == Some(BulletMarker::Completed) {
                                    rich = rich.strikethrough().weak();
                                }
                                let response = ui.add_sized(
                                    [remaining, TOUCH],
                                    egui::Label::new(rich).wrap().sense(egui::Sense::click()),
                                );
                                if response.clicked() {
                                    self.editing_bullet = Some((index, bullet.text().to_owned()));
                                    let id = edit_bullet_id(kind, page_index, index);
                                    ui.ctx().memory_mut(|mem| mem.request_focus(id));
                                } else if response.long_touched() {
                                    self.mobile.overlay = Some(MobileOverlay::BulletMenu(index));
                                }
                            }

                            let menu = ui.add_sized([TOUCH, TOUCH], egui::Button::new("⋯"));
                            if menu.clicked() {
                                self.mobile.overlay = Some(MobileOverlay::BulletMenu(index));
                            }
                        });
                        let rect = row.response.rect;
                        let line_y = rect.bottom() + 2.0;
                        ui.painter().line_segment(
                            [
                                egui::pos2(rect.left(), line_y),
                                egui::pos2(ui.max_rect().right(), line_y),
                            ],
                            egui::Stroke::new(1.0, self.theme.palette().accent),
                        );
                        ui.add_space(2.0);
                    }
                    ui.add_space(24.0);
                });

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
        });
    }

    fn mobile_sections(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("mobile_sections")
            .exact_height(52.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let gap = 8.0;
                    let width = ((ui.available_width() - gap * 2.0) / 3.0).max(70.0);
                    for kind in SectionKind::ALL {
                        let response = ui.add_sized(
                            [width, 40.0],
                            egui::Button::new(note_text(section_label(kind, self.lang), self.lang))
                                .selected(self.active_section == kind),
                        );
                        if response.clicked() {
                            self.switch_section(kind);
                        }
                    }
                });
            });
    }

    fn mobile_pages(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("mobile_pages")
            .exact_height(56.0)
            .show(ctx, |ui| {
                egui::ScrollArea::horizontal()
                    .id_salt("mobile_page_strip")
                    .show(ui, |ui| {
                        ui.horizontal_centered(|ui| {
                            let pages: Vec<String> = self
                                .current
                                .as_ref()
                                .map(|journal| {
                                    journal
                                        .section(self.active_section)
                                        .pages()
                                        .iter()
                                        .map(|page| page.title().to_owned())
                                        .collect()
                                })
                                .unwrap_or_default();

                            for (index, title) in pages.iter().enumerate() {
                                let selected = self.selected_page == Some(index);
                                let response = ui.add_sized(
                                    [96.0, 40.0],
                                    egui::Button::new(note_text(title, self.lang))
                                        .selected(selected),
                                );
                                if response.clicked() {
                                    self.selected_page = Some(index);
                                } else if response.long_touched() {
                                    self.mobile.overlay = Some(MobileOverlay::PageMenu(index));
                                }
                            }

                            if ui
                                .add_sized([44.0, 40.0], egui::Button::new("＋"))
                                .on_hover_text(self.lang.t("new_page"))
                                .clicked()
                            {
                                self.new_page_title.clear();
                                self.mobile.overlay = Some(MobileOverlay::NewPage);
                                self.mobile.focus_pending = true;
                            }

                            if let Some(index) = self.selected_page {
                                let menu = ui.add_sized([44.0, 40.0], egui::Button::new("⋯"));
                                if menu.clicked() {
                                    self.mobile.overlay = Some(MobileOverlay::PageMenu(index));
                                }
                            }
                        });
                    });
            });
    }

    fn mobile_composer(&mut self, ctx: &egui::Context) {
        if self.selected_page.is_none() {
            return;
        }
        let ime = crate::ime_inset_points(ctx);
    egui::TopBottomPanel::bottom("mobile_composer").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for (kind, label) in [
                    (NewBulletKind::Task, self.lang.t("task")),
                    (NewBulletKind::Note, self.lang.t("note")),
                    (NewBulletKind::Event, self.lang.t("event")),
                ] {
                    let response = ui.add_sized(
                        [88.0, 36.0],
                        egui::Button::new(note_text(label, self.lang))
                            .selected(self.new_bullet_kind == kind),
                    );
                    if response.clicked() {
                        self.new_bullet_kind = kind;
                    }
                }
            });
            ui.horizontal(|ui| {
                let width = (ui.available_width() - TOUCH - 8.0).max(80.0);
                let response = ui.add_sized(
                    [width, TOUCH],
                    egui::TextEdit::singleline(&mut self.new_bullet)
                        .font(note_font(18.0, self.lang))
                        .hint_text(hint(self.lang.t("write_here"), self.theme)),
                );
                let submitted =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if ui
                    .add_sized([TOUCH, TOUCH], egui::Button::new("＋"))
                    .on_hover_text(self.lang.t("add_button"))
                    .clicked()
                    || submitted
                {
                    self.add_bullet();
                }
            });
            ui.add_space(4.0);
            if ime > 0.0 {
                ui.add_space(ime);
            }
        });
    }

    fn mobile_overlays(&mut self, ctx: &egui::Context) {
        if let Some(overlay) = self.mobile.overlay.clone() {
            let close = self.mobile_overlay_content(ctx, overlay);
            if close {
                self.mobile.overlay = None;
            }
        }

        if self.creating_journal {
            self.mobile_create_journal(ctx);
        }
        if self.renaming_journal.is_some() {
            self.mobile_rename_journal(ctx);
        }
        if let Some(confirm) = self.mobile.confirm.clone() {
            self.mobile_confirm(ctx, confirm);
        }
    }

    fn mobile_overlay_content(&mut self, ctx: &egui::Context, overlay: MobileOverlay) -> bool {
        match overlay {
            MobileOverlay::JournalMenu(name) => {
                let mut open = false;
                let mut rename = false;
                let mut delete = false;
                let mut create = false;
                let close = sheet(
                    ctx,
                    "mobile_journal_menu",
                    name.as_str(),
                    self.lang.t("cancel"),
                    self.theme,
                    |ui| {
                        if big_button(ui, self.lang.t("rename")) {
                            rename = true;
                            return true;
                        }
                        if big_button(ui, self.lang.t("delete")) {
                            delete = true;
                            return true;
                        }
                        if big_button(ui, self.lang.t("new_journal")) {
                            create = true;
                            return true;
                        }
                        if big_button(ui, self.lang.t("journals")) {
                            open = true;
                            return true;
                        }
                        false
                    },
                );

                if open {
                    self.open_journal(name.clone());
                    self.mobile.screen = MobileScreen::Journal;
                } else if rename {
                    self.rename_name = name.as_str().to_owned();
                    self.renaming_journal = Some(name.clone());
                    self.mobile.focus_pending = true;
                } else if delete {
                    self.mobile.confirm = Some(MobileConfirm::Journal(name));
                } else if create {
                    self.new_journal_name.clear();
                    self.creating_journal = true;
                    self.mobile.focus_pending = true;
                }
                close
            }
            MobileOverlay::PageMenu(index) => {
                let mut delete = false;
                let mut create = false;
                let title = self
                    .current
                    .as_ref()
                    .and_then(|journal| journal.page(self.active_section, index))
                    .map(|page| page.title().to_owned())
                    .unwrap_or_default();
                let close = sheet(
                    ctx,
                    "mobile_page_menu",
                    &title,
                    self.lang.t("cancel"),
                    self.theme,
                    |ui| {
                        if big_button(ui, self.lang.t("new_page")) {
                            create = true;
                            return true;
                        }
                        if big_button(ui, self.lang.t("delete_page")) {
                            delete = true;
                            return true;
                        }
                        false
                    },
                );
                if create {
                    self.new_page_title.clear();
                    self.mobile.overlay = Some(MobileOverlay::NewPage);
                    self.mobile.focus_pending = true;
                } else if delete {
                    self.mobile.confirm = Some(MobileConfirm::Page(index));
                }
                close
            }
            MobileOverlay::BulletMenu(index) => {
                let mut edit = false;
                let mut up = false;
                let mut down = false;
                let mut delete = false;
                let mut color: Option<AnnotationColor> = None;
                let title = self
                    .current
                    .as_ref()
                    .and_then(|journal| journal.page(self.active_section, self.selected_page?))
                    .and_then(|page| page.bullets().get(index))
                    .map(|bullet| bullet.text().to_owned())
                    .unwrap_or_default();
                let close = sheet(
                    ctx,
                    "mobile_bullet_menu",
                    &title,
                    self.lang.t("cancel"),
                    self.theme,
                    |ui| {
                        if big_button(ui, self.lang.t("edit_text")) {
                            edit = true;
                            return true;
                        }
                        if big_button(ui, self.lang.t("move")) {
                            ui.horizontal(|ui| {
                                if big_button(ui, "↑") {
                                    up = true;
                                }
                                if big_button(ui, "↓") {
                                    down = true;
                                }
                            });
                            return up || down;
                        }
                        ui.label(self.lang.t("text_color"));
                        for option in AnnotationColor::ALL {
                            let label = egui::RichText::new(color_label(option, self.lang))
                                .color(annotation_color(option, self.theme));
                            if ui
                                .add_sized([ui.available_width(), TOUCH], egui::Button::new(label))
                                .clicked()
                            {
                                color = Some(option);
                                return true;
                            }
                        }
                        if big_button(ui, self.lang.t("delete")) {
                            delete = true;
                            return true;
                        }
                        false
                    },
                );

                if edit {
                    if let Some(bullet) = self
                        .current
                        .as_ref()
                        .and_then(|journal| journal.page(self.active_section, self.selected_page?))
                        .and_then(|page| page.bullets().get(index))
                    {
                        self.editing_bullet = Some((index, bullet.text().to_owned()));
                        if let Some(page_index) = self.selected_page {
                            let id = edit_bullet_id(self.active_section, page_index, index);
                            ctx.memory_mut(|mem| mem.request_focus(id));
                        }
                    }
                } else if up {
                    self.mobile_move_bullet(index, -1);
                } else if down {
                    self.mobile_move_bullet(index, 1);
                } else if let Some(color) = color {
                    self.mobile_set_color(index, color);
                } else if delete {
                    self.mobile.confirm = Some(MobileConfirm::Bullet(index));
                }
                close
            }
            MobileOverlay::MarkerPicker(index) => {
                let mut selected = None;
                let close = sheet(
                    ctx,
                    "mobile_marker_menu",
                    self.lang.t("change_marker"),
                    self.lang.t("cancel"),
                    self.theme,
                    |ui| {
                        for marker in BulletMarker::ALL {
                            let label =
                                format!("{}  {}", marker.symbol(), marker_label(marker, self.lang));
                            if ui
                                .add_sized(
                                    [ui.available_width(), TOUCH],
                                    egui::Button::new(note_text(label, self.lang)),
                                )
                                .clicked()
                            {
                                selected = Some(marker);
                                return true;
                            }
                        }
                        false
                    },
                );
                if let Some(marker) = selected {
                    self.mobile_set_marker(index, marker);
                }
                close
            }
            MobileOverlay::NewPage => {
                let mut create = false;
                let close = sheet(
                    ctx,
                    "mobile_new_page",
                    self.lang.t("new_page"),
                    self.lang.t("cancel"),
                    self.theme,
                    |ui| {
                        let label = ui.label(self.lang.t("new_page"));
                        let response = ui.add_sized(
                            [ui.available_width(), TOUCH],
                            egui::TextEdit::singleline(&mut self.new_page_title)
                                .id(egui::Id::new("mobile_new_page_title"))
                                .font(note_font(18.0, self.lang))
                                .hint_text(hint(self.lang.t("hint_date"), self.theme)),
                        );
                        let response = response.labelled_by(label.id);
                        if self.mobile.focus_pending {
                            response.request_focus();
                            self.mobile.focus_pending = false;
                        }
                        let submitted =
                            response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if big_button(ui, self.lang.t("create_page")) || submitted {
                            create = true;
                            return true;
                        }
                        false
                    },
                );
                if create && !self.new_page_title.trim().is_empty() {
                    self.create_page();
                    if self.new_page_title.is_empty() {
                        self.mobile.overlay = None;
                    }
                    return false;
                }
                close
            }
            MobileOverlay::Settings => {
                let mut changed_theme = false;
                let close = sheet(
                    ctx,
                    "mobile_settings",
                    self.lang.t("options"),
                    self.lang.t("cancel"),
                    self.theme,
                    |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(ui.ctx().content_rect().height() * 0.55)
                            .show(ui, |ui| {
                                ui.heading(self.lang.t("theme_label"));
                                for theme in Theme::ALL {
                                    if ui
                                        .add_sized(
                                            [ui.available_width(), 40.0],
                                            egui::Button::new(theme.label())
                                                .selected(self.theme == theme),
                                        )
                                        .clicked()
                                    {
                                        self.theme = theme;
                                        changed_theme = true;
                                    }
                                }
                                ui.add_space(8.0);
                                ui.heading(self.lang.t("language_label"));
                                for lang in Lang::ALL {
                                    if ui
                                        .add_sized(
                                            [ui.available_width(), 40.0],
                                            egui::Button::new(lang.label())
                                                .selected(self.lang == lang),
                                        )
                                        .clicked()
                                    {
                                        self.lang = lang;
                                    }
                                }
                                ui.add_space(8.0);
                                if big_button(ui, self.lang.t("about")) {
                                    self.mobile.overlay = Some(MobileOverlay::About);
                                    return true;
                                }
                                false
                            })
                            .inner
                    },
                );
                if changed_theme {
                    super::configure_style(ctx, self.theme);
                }
                close
            }
            MobileOverlay::About => sheet(
                ctx,
                "mobile_about",
                self.lang.t("about_title"),
                self.lang.t("cancel"),
                self.theme,
                |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(ui.ctx().content_rect().height() * 0.55)
                        .show(ui, |ui| {
                            ui.heading(note_heading("UJournal", self.lang));
                            ui.label(format!(
                                "{} {}",
                                self.lang.t("version"),
                                env!("CARGO_PKG_VERSION")
                            ));
                            ui.label(env!("CARGO_PKG_DESCRIPTION"));
                            ui.label("© Ulises Lafuente Ramos - 2026");
                            ui.label(format!(
                                "{}: {}",
                                self.lang.t("license"),
                                env!("CARGO_PKG_LICENSE")
                            ));
                            let repository = env!("CARGO_PKG_REPOSITORY");
                            if !repository.is_empty() {
                                ui.hyperlink_to(self.lang.t("repository"), repository);
                            }
                            ui.separator();
                            ui.collapsing(self.lang.t("license_mit"), |ui| {
                                super::license_block(ui, include_str!("../../LICENSE-MIT"), false);
                            });
                            ui.collapsing(self.lang.t("license_apache"), |ui| {
                                super::license_block(
                                    ui,
                                    include_str!("../../LICENSE-APACHE"),
                                    false,
                                );
                            });
                            ui.collapsing(self.lang.t("font_license"), |ui| {
                                super::license_block(
                                    ui,
                                    include_str!("../../assets/fonts/OFL-Caveat.txt"),
                                    false,
                                );
                            });
                        });
                    false
                },
            ),
        }
    }

    fn mobile_create_journal(&mut self, ctx: &egui::Context) {
        let mut create = false;
        let close = sheet(
            ctx,
            "mobile_create_journal",
            self.lang.t("new_journal"),
            self.lang.t("cancel"),
            self.theme,
            |ui| {
                let label = ui.label(self.lang.t("new_journal"));
                let response = ui.add_sized(
                    [ui.available_width(), TOUCH],
                    egui::TextEdit::singleline(&mut self.new_journal_name)
                        .id(egui::Id::new("mobile_new_journal_name"))
                        .font(note_font(18.0, self.lang))
                        .hint_text(hint(self.lang.t("hint_journal"), self.theme)),
                );
                let response = response.labelled_by(label.id);
                if self.mobile.focus_pending {
                    response.request_focus();
                    self.mobile.focus_pending = false;
                }
                let submitted =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if big_button(ui, self.lang.t("create")) || submitted {
                    create = true;
                    return true;
                }
                false
            },
        );
        if create && self.create_journal() {
            self.mobile.screen = MobileScreen::Journal;
            self.mobile.overlay = None;
        }
        if close {
            self.creating_journal = false;
        }
    }

    fn mobile_rename_journal(&mut self, ctx: &egui::Context) {
        let Some(old_name) = self.renaming_journal.clone() else {
            return;
        };
        let mut rename = false;
        let close = sheet(
            ctx,
            "mobile_rename_journal",
            self.lang.t("rename"),
            self.lang.t("cancel"),
            self.theme,
            |ui| {
                ui.label(note_text(old_name.as_str(), self.lang));
                let label = ui.label(self.lang.t("rename_label"));
                let response = ui.add_sized(
                    [ui.available_width(), TOUCH],
                    egui::TextEdit::singleline(&mut self.rename_name)
                        .id(egui::Id::new("mobile_rename_journal_name"))
                        .font(note_font(18.0, self.lang)),
                );
                let response = response.labelled_by(label.id);
                if self.mobile.focus_pending {
                    response.request_focus();
                    self.mobile.focus_pending = false;
                }
                let submitted =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if big_button(ui, self.lang.t("rename")) || submitted {
                    rename = true;
                    return true;
                }
                false
            },
        );
        if rename && self.rename_selected() {
            self.renaming_journal = None;
        }
        if close {
            self.renaming_journal = None;
        }
    }

    fn mobile_confirm(&mut self, ctx: &egui::Context, confirm: MobileConfirm) {
        let mut confirmed = false;
        let (title, message) = match &confirm {
            MobileConfirm::Journal(name) => (
                self.lang.t("delete").to_owned(),
                format!("{}: {}", self.lang.t("delete"), name.as_str()),
            ),
            MobileConfirm::Page(index) => (
                self.lang.t("delete_page").to_owned(),
                self.current
                    .as_ref()
                    .and_then(|journal| journal.page(self.active_section, *index))
                    .map(|page| format!("{}: {}", self.lang.t("delete_page"), page.title()))
                    .unwrap_or_else(|| self.lang.t("delete_page").to_owned()),
            ),
            MobileConfirm::Bullet(index) => (
                self.lang.t("delete").to_owned(),
                self.current
                    .as_ref()
                    .and_then(|journal| journal.page(self.active_section, self.selected_page?))
                    .and_then(|page| page.bullets().get(*index))
                    .map(|bullet| bullet.text().to_owned())
                    .unwrap_or_default(),
            ),
        };
        let close = sheet(
            ctx,
            "mobile_confirm",
            &title,
            self.lang.t("cancel"),
            self.theme,
            |ui| {
                ui.label(note_text(message, self.lang));
                if big_button(ui, self.lang.t("delete")) {
                    confirmed = true;
                    return true;
                }
                false
            },
        );
        if confirmed {
            match confirm {
                MobileConfirm::Journal(name) => {
                    self.selected = Some(name);
                    self.delete_selected();
                    self.mobile.screen = MobileScreen::Journals;
                }
                MobileConfirm::Page(index) => self.delete_page(index),
                MobileConfirm::Bullet(index) => {
                    if let Some(page) = self.current.as_mut().and_then(|journal| {
                        journal.page_mut(self.active_section, self.selected_page?)
                    }) {
                        page.remove(index);
                    }
                    self.dirty = true;
                }
            }
            self.mobile.confirm = None;
        } else if close {
            self.mobile.confirm = None;
        }
    }

    fn mobile_set_marker(&mut self, index: usize, marker: BulletMarker) {
        if let Some(page) = self
            .current
            .as_mut()
            .and_then(|journal| journal.page_mut(self.active_section, self.selected_page?))
            && let Some(bullet) = page.bullet_mut(index)
        {
            bullet.set_marker(marker);
        }
        self.dirty = true;
    }

    fn mobile_set_color(&mut self, index: usize, color: AnnotationColor) {
        if let Some(page) = self
            .current
            .as_mut()
            .and_then(|journal| journal.page_mut(self.active_section, self.selected_page?))
            && let Some(bullet) = page.bullet_mut(index)
        {
            bullet.set_color(color);
        }
        self.dirty = true;
    }

    fn mobile_move_bullet(&mut self, index: usize, delta: isize) {
        let Some(page_index) = self.selected_page else {
            return;
        };
        let Some(page) = self
            .current
            .as_mut()
            .and_then(|journal| journal.page_mut(self.active_section, page_index))
        else {
            return;
        };
        let len = page.bullets().len();
        if len == 0 {
            return;
        }
        let target = if delta < 0 {
            index.saturating_sub(1)
        } else {
            (index + 1).min(len - 1)
        };
        if page.move_bullet(index, target) {
            self.dirty = true;
        }
    }

    fn mobile_snackbar(&mut self, ctx: &egui::Context) {
        let Some(status) = &self.status else {
            return;
        };
        let message = status.message.clone();
        let color = if status.is_error {
            super::ERROR_INK
        } else {
            super::SUCCESS_INK
        };
        let offset = if self.selected_page.is_some() {
            -116.0
        } else {
            -16.0
        };
        egui::Area::new(egui::Id::new("mobile_snackbar"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, offset))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(self.theme.palette().bg_alt)
                    .stroke(egui::Stroke::new(1.0, self.theme.palette().accent))
                    .corner_radius(egui::CornerRadius::same(10))
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&message).color(color).size(14.0),
                                )
                                .wrap(),
                            );
                            if ui
                                .add_sized([32.0, 32.0], egui::Button::new("×").frame(false))
                                .clicked()
                            {
                                self.status = None;
                            }
                        });
                    });
            });
    }
}

fn big_button(ui: &mut egui::Ui, label: &str) -> bool {
    ui.add_sized(
        [ui.available_width(), TOUCH],
        egui::Button::new(script_text(label)),
    )
    .clicked()
}

fn edit_bullet_id(kind: SectionKind, page_index: usize, index: usize) -> egui::Id {
    egui::Id::new(("edit_bullet", kind as u8, page_index, index))
}

fn sheet(
    ctx: &egui::Context,
    id: &'static str,
    title: &str,
    cancel_label: &str,
    theme: Theme,
    add: impl FnOnce(&mut egui::Ui) -> bool,
) -> bool {
    let p = theme.palette();
    let width = (ctx.content_rect().width() - 24.0).clamp(260.0, 520.0);
    let mut close = false;
    let ime = crate::ime_inset_points(ctx);
    let response = egui::Modal::new(egui::Id::new(id))
        .area(
            egui::Modal::default_area(egui::Id::new(id))
                .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -10.0 - ime)),
        )
        .frame(
            egui::Frame::new()
                .fill(p.bg)
                .stroke(egui::Stroke::new(1.0, p.accent))
                .corner_radius(egui::CornerRadius::same(14))
                .inner_margin(egui::Margin::same(14)),
        )
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.heading(title);
            ui.add_space(6.0);
            if add(ui) {
                close = true;
            }
            ui.add_space(6.0);
            if ui
                .add_sized(
                    [ui.available_width(), TOUCH],
                    egui::Button::new(cancel_label),
                )
                .clicked()
            {
                close = true;
            }
        });
    if response.backdrop_response.clicked() {
        close = true;
    }
    close
}

/// Kept for symmetry with the desktop layout, where the journal is passed
/// explicitly. Useful when a future mobile flow opens a journal by value.
#[allow(dead_code)]
fn _journal_name(journal: &Journal) -> &str {
    journal.name().as_str()
}
