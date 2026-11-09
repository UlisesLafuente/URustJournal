use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    PapelAmarillento,
    AzulNordico,
    BosqueAntiguo,
    SuenoVioleta,
    VSCodeDarkPlus,
    Dracula,
    Nord,
    OneDarkPro,
    Monokai,
    SolarizedDark,
    SolarizedLight,
    GruvboxDark,
    CatppuccinMocha,
    GitHubLight,
    // cozy
    ChimeneaCrepitante,
    TardeDeLluvia,
    MantaDeLana,
    CafeConLeche,
    BibliotecaSilenciosa,
    NocheEstrellada,
    BrumaMatinal,
    HuertoDeOtono,
    CerezoEnFlor,
    SolDeInvierno,
    CabañaEnElBosque,
    PanRecienHecho,
    ArcillaYTierra,
    CosturaALaLumbre,
    MarEnCalma,
    Luciernagas,
    SoledadSerena,
    PrimerasHojas,
    MermeladaDeMora,
    AtardecerDorado,
}

impl Theme {
    pub const ALL: [Self; 34] = [
        Self::PapelAmarillento,
        Self::AzulNordico,
        Self::BosqueAntiguo,
        Self::SuenoVioleta,
        Self::VSCodeDarkPlus,
        Self::Dracula,
        Self::Nord,
        Self::OneDarkPro,
        Self::Monokai,
        Self::SolarizedDark,
        Self::SolarizedLight,
        Self::GruvboxDark,
        Self::CatppuccinMocha,
        Self::GitHubLight,
        Self::ChimeneaCrepitante,
        Self::TardeDeLluvia,
        Self::MantaDeLana,
        Self::CafeConLeche,
        Self::BibliotecaSilenciosa,
        Self::NocheEstrellada,
        Self::BrumaMatinal,
        Self::HuertoDeOtono,
        Self::CerezoEnFlor,
        Self::SolDeInvierno,
        Self::CabañaEnElBosque,
        Self::PanRecienHecho,
        Self::ArcillaYTierra,
        Self::CosturaALaLumbre,
        Self::MarEnCalma,
        Self::Luciernagas,
        Self::SoledadSerena,
        Self::PrimerasHojas,
        Self::MermeladaDeMora,
        Self::AtardecerDorado,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::PapelAmarillento => "Papel amarillento",
            Self::AzulNordico => "Azul nórdico",
            Self::BosqueAntiguo => "Bosque antiguo",
            Self::SuenoVioleta => "Sueño violeta",
            Self::VSCodeDarkPlus => "VS Code Dark+",
            Self::Dracula => "Dracula",
            Self::Nord => "Nord",
            Self::OneDarkPro => "One Dark Pro",
            Self::Monokai => "Monokai",
            Self::SolarizedDark => "Solarized Dark",
            Self::SolarizedLight => "Solarized Light",
            Self::GruvboxDark => "Gruvbox Dark",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::GitHubLight => "GitHub Light",
            Self::ChimeneaCrepitante => "Chimenea crepitante",
            Self::TardeDeLluvia => "Tarde de lluvia",
            Self::MantaDeLana => "Manta de lana",
            Self::CafeConLeche => "Café con leche",
            Self::BibliotecaSilenciosa => "Biblioteca silenciosa",
            Self::NocheEstrellada => "Noche estrellada",
            Self::BrumaMatinal => "Bruma matinal",
            Self::HuertoDeOtono => "Huerto de otoño",
            Self::CerezoEnFlor => "Cerezo en flor",
            Self::SolDeInvierno => "Sol de invierno",
            Self::CabañaEnElBosque => "Cabaña en el bosque",
            Self::PanRecienHecho => "Pan recién hecho",
            Self::ArcillaYTierra => "Arcilla y tierra",
            Self::CosturaALaLumbre => "Costura a la lumbre",
            Self::MarEnCalma => "Mar en calma",
            Self::Luciernagas => "Luciérnagas",
            Self::SoledadSerena => "Soledad serena",
            Self::PrimerasHojas => "Primeras hojas",
            Self::MermeladaDeMora => "Mermelada de mora",
            Self::AtardecerDorado => "Atardecer dorado",
        }
    }

    pub fn palette(self) -> ThemePalette {
        match self {
            Self::PapelAmarillento => {
                ThemePalette::new(0xEAE6DF, 0xD5D6D2, 0xE2D9C5, 0xCFC2B4, 0x3B352F, false)
            }
            Self::AzulNordico => {
                ThemePalette::new(0x07111F, 0x0B3D6E, 0x1D8ACB, 0x60D6FF, 0xB8F3FF, true)
            }
            Self::BosqueAntiguo => {
                ThemePalette::new(0x08140F, 0x1E4D35, 0x3FA86D, 0x9AD98B, 0xE1F0B5, true)
            }
            Self::SuenoVioleta => {
                ThemePalette::new(0x0E0719, 0x5A2CFF, 0x9B5CFF, 0xF25CFF, 0x58F2D7, true)
            }
            Self::VSCodeDarkPlus => {
                ThemePalette::new(0x1E1E1E, 0x252526, 0x264F78, 0x007ACC, 0xD4D4D4, true)
            }
            Self::Dracula => {
                ThemePalette::new(0x282A36, 0x44475A, 0x6272A4, 0xBD93F9, 0xF8F8F2, true)
            }
            Self::Nord => ThemePalette::new(0x2E3440, 0x3B4252, 0x4C566A, 0x88C0D0, 0xECEFF4, true),
            Self::OneDarkPro => {
                ThemePalette::new(0x282C34, 0x21252B, 0x3E4451, 0x61AFEF, 0xABB2BF, true)
            }
            Self::Monokai => {
                ThemePalette::new(0x272822, 0x3E3D32, 0x75715E, 0xA6E22E, 0xF8F8F2, true)
            }
            Self::SolarizedDark => {
                ThemePalette::new(0x002B36, 0x073642, 0x586E75, 0x268BD2, 0x839496, true)
            }
            Self::SolarizedLight => {
                ThemePalette::new(0xFDF6E3, 0xEEE8D5, 0x93A1A1, 0x268BD2, 0x657B83, false)
            }
            Self::GruvboxDark => {
                ThemePalette::new(0x282828, 0x3C3836, 0x504945, 0xFE8019, 0xEBDBB2, true)
            }
            Self::CatppuccinMocha => {
                ThemePalette::new(0x1E1E2E, 0x313244, 0x45475A, 0xCBA6F7, 0xCDD6F4, true)
            }
            Self::GitHubLight => {
                ThemePalette::new(0xFFFFFF, 0xF6F8FA, 0xDDF4FF, 0x0969DA, 0x1F2328, false)
            }
            // fire & hearth
            Self::ChimeneaCrepitante => {
                ThemePalette::new(0x2B1A12, 0x4A2C1A, 0x8C4A2F, 0xE07A3A, 0xF6D6A8, true)
            }
            Self::AtardecerDorado => {
                ThemePalette::new(0xF7E7C6, 0xF0D3A0, 0xE8A45C, 0xC96A3A, 0x5A3523, false)
            }
            Self::PanRecienHecho => {
                ThemePalette::new(0xF6EBDD, 0xE8D7BE, 0xD2B48C, 0xB08968, 0x4A3426, false)
            }
            // rain, night, solitude, calm
            Self::TardeDeLluvia => {
                ThemePalette::new(0x2F3A46, 0x3F4C59, 0x6C8096, 0x9FB7C9, 0xE6EEF3, true)
            }
            Self::NocheEstrellada => {
                ThemePalette::new(0x0B1020, 0x182038, 0x2C3E6B, 0x6FA8FF, 0xE8EEFF, true)
            }
            Self::BrumaMatinal => {
                ThemePalette::new(0xE9EEF2, 0xD7E1E8, 0xB7C7D3, 0x7E97AB, 0x2F3E4A, false)
            }
            Self::MarEnCalma => {
                ThemePalette::new(0x1E3A44, 0x2A505C, 0x4F8A93, 0x8ECED4, 0xEAF7F8, true)
            }
            Self::SoledadSerena => {
                ThemePalette::new(0x1C1F24, 0x2A2F36, 0x4A5563, 0x8A97A8, 0xD8DEE6, true)
            }
            Self::Luciernagas => {
                ThemePalette::new(0x101A16, 0x1C2E25, 0x355E43, 0xB7D96A, 0xF1F7D8, true)
            }
            // homey, traditional, craft
            Self::MantaDeLana => {
                ThemePalette::new(0xF3E8D7, 0xE6D2B8, 0xCFAE8A, 0x9C6B4F, 0x4B3326, false)
            }
            Self::CafeConLeche => {
                ThemePalette::new(0xF4E9DC, 0xE3CDB6, 0xC8A27A, 0x8B5E3C, 0x3E2720, false)
            }
            Self::BibliotecaSilenciosa => {
                ThemePalette::new(0x2A2118, 0x3C2F23, 0x6B4F34, 0xB88A4A, 0xF0E2C8, true)
            }
            Self::ArcillaYTierra => {
                ThemePalette::new(0xF0E2D2, 0xDFC7B0, 0xB8845A, 0x8A5A3C, 0x3D271C, false)
            }
            Self::CosturaALaLumbre => {
                ThemePalette::new(0xFBF2E8, 0xF0DCCB, 0xD8A1A6, 0xA35D6A, 0x4A2A32, false)
            }
            Self::CabañaEnElBosque => {
                ThemePalette::new(0x2C241C, 0x3E3326, 0x5E7A4A, 0xA3B86C, 0xF2E8D5, true)
            }
            // seasons
            Self::HuertoDeOtono => {
                ThemePalette::new(0xF5E6CF, 0xE7CFA8, 0xC97C3D, 0x7A4A2E, 0x3B2417, false)
            }
            Self::CerezoEnFlor => {
                ThemePalette::new(0xFFF5F7, 0xF7DEE6, 0xF0AFC2, 0xC97C95, 0x4A2C35, false)
            }
            Self::SolDeInvierno => {
                ThemePalette::new(0xF4F7FA, 0xE1EAF2, 0xC6D7EA, 0x7FA6D6, 0x2C3E57, false)
            }
            Self::PrimerasHojas => {
                ThemePalette::new(0xF4F7E8, 0xE3ECD0, 0xB9D18B, 0x6F9B4C, 0x304225, false)
            }
            Self::MermeladaDeMora => {
                ThemePalette::new(0x2A1626, 0x40203A, 0x7A3E6B, 0xC86BA3, 0xF6E7F2, true)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub bg: egui::Color32,
    pub bg_alt: egui::Color32,
    pub accent_soft: egui::Color32,
    pub accent: egui::Color32,
    pub ink: egui::Color32,
    pub dark_mode: bool,
}

impl ThemePalette {
    pub const fn new(
        bg: u32,
        bg_alt: u32,
        accent_soft: u32,
        accent: u32,
        ink: u32,
        dark_mode: bool,
    ) -> Self {
        Self {
            bg: rgb(bg),
            bg_alt: rgb(bg_alt),
            accent_soft: rgb(accent_soft),
            accent: rgb(accent),
            ink: rgb(ink),
            dark_mode,
        }
    }
}

const fn rgb(hex: u32) -> egui::Color32 {
    egui::Color32::from_rgb(
        ((hex >> 16) & 0xFF) as u8,
        ((hex >> 8) & 0xFF) as u8,
        (hex & 0xFF) as u8,
    )
}

pub fn configure_style(ctx: &egui::Context, theme: Theme) {
    let p = theme.palette();
    let mut style = (*ctx.style()).clone();
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
