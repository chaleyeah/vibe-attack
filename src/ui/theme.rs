use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Margin, Stroke, Style,
    Visuals,
};

// ── Palette ───────────────────────────────────────────────────────────────────

pub struct Palette;

impl Palette {
    // Surfaces
    pub const BG_EXTREME: Color32 = Color32::from_rgb(0x08, 0x09, 0x0a);
    pub const BG_WINDOW:  Color32 = Color32::from_rgb(0x0e, 0x10, 0x12);
    pub const BG_PANEL:   Color32 = Color32::from_rgb(0x14, 0x17, 0x1a);
    pub const BG_FAINT:   Color32 = Color32::from_rgb(0x1a, 0x1d, 0x20);
    pub const BG_RAISED:  Color32 = Color32::from_rgb(0x1f, 0x23, 0x27);
    pub const BG_HOVER:   Color32 = Color32::from_rgb(0x26, 0x2a, 0x2f);
    pub const BG_ACTIVE:  Color32 = Color32::from_rgb(0x2d, 0x32, 0x38);

    // Strokes
    pub const STROKE_FAINT:  Color32 = Color32::from_rgb(0x22, 0x26, 0x2b);
    pub const STROKE:        Color32 = Color32::from_rgb(0x2d, 0x32, 0x38);
    pub const STROKE_STRONG: Color32 = Color32::from_rgb(0x3a, 0x40, 0x47);
    pub const STROKE_BRIGHT: Color32 = Color32::from_rgb(0x4d, 0x55, 0x5e);

    // Text
    pub const FG_STRONG: Color32 = Color32::from_rgb(0xe6, 0xe7, 0xe8);
    pub const FG:        Color32 = Color32::from_rgb(0xc4, 0xc7, 0xcb);
    pub const FG_MUTED:  Color32 = Color32::from_rgb(0x8a, 0x90, 0x99);
    pub const FG_FAINT:  Color32 = Color32::from_rgb(0x5d, 0x64, 0x6d);
    pub const FG_DIM:    Color32 = Color32::from_rgb(0x44, 0x4a, 0x52);

    // Accent — amber
    pub const ACCENT:       Color32 = Color32::from_rgb(0xe8, 0xa3, 0x17);
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0xf5, 0xb7, 0x33);
    pub const ACCENT_FG:    Color32 = Color32::from_rgb(0x15, 0x11, 0x0a);

    pub fn accent_faint() -> Color32 {
        Color32::from_rgba_unmultiplied(0xe8, 0xa3, 0x17, 36) // ~0.14 alpha
    }
    pub fn accent_line() -> Color32 {
        Color32::from_rgba_unmultiplied(0xe8, 0xa3, 0x17, 107) // ~0.42 alpha
    }

    // Status
    pub const OK:   Color32 = Color32::from_rgb(0x5f, 0xd4, 0x7a);
    pub const WARN: Color32 = Color32::from_rgb(0xe8, 0xa3, 0x17);
    pub const ERR:  Color32 = Color32::from_rgb(0xe8, 0x5d, 0x3c);
    pub const INFO: Color32 = Color32::from_rgb(0x6a, 0xb8, 0xe8);

    pub fn ok_faint() -> Color32 {
        Color32::from_rgba_unmultiplied(0x5f, 0xd4, 0x7a, 36)
    }
    pub fn warn_faint() -> Color32 {
        Color32::from_rgba_unmultiplied(0xe8, 0xa3, 0x17, 36)
    }
    pub fn err_faint() -> Color32 {
        Color32::from_rgba_unmultiplied(0xe8, 0x5d, 0x3c, 36)
    }
    pub fn info_faint() -> Color32 {
        Color32::from_rgba_unmultiplied(0x6a, 0xb8, 0xe8, 36)
    }
}

// ── Font keys ─────────────────────────────────────────────────────────────────

pub const FONT_REGULAR: &str = "JetBrainsMono-Regular";
pub const FONT_MEDIUM:  &str = "JetBrainsMono-Medium";

// ── Font loading ──────────────────────────────────────────────────────────────

pub fn load_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        FONT_REGULAR.to_owned(),
        FontData::from_static(include_bytes!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/JetBrainsMono-Regular.ttf")
        )).into(),
    );
    fonts.font_data.insert(
        FONT_MEDIUM.to_owned(),
        FontData::from_static(include_bytes!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/JetBrainsMono-Medium.ttf")
        )).into(),
    );

    // Replace the default font families with JetBrains Mono.
    // Medium weight for headings (Proportional family), Regular for mono/body.
    fonts.families.insert(
        FontFamily::Proportional,
        vec![FONT_MEDIUM.to_owned(), FONT_REGULAR.to_owned()],
    );
    fonts.families.insert(
        FontFamily::Monospace,
        vec![FONT_REGULAR.to_owned(), FONT_MEDIUM.to_owned()],
    );

    ctx.set_fonts(fonts);
}

// ── Theme application ─────────────────────────────────────────────────────────

const R3: CornerRadius = CornerRadius::same(3);
const R4: CornerRadius = CornerRadius::same(4);

pub fn apply_theme(ctx: &egui::Context) {
    load_fonts(ctx);

    let mut visuals = Visuals::dark();

    // Window / panel backgrounds
    visuals.window_fill       = Palette::BG_WINDOW;
    visuals.panel_fill        = Palette::BG_PANEL;
    visuals.faint_bg_color    = Palette::BG_FAINT;
    visuals.extreme_bg_color  = Palette::BG_EXTREME;
    visuals.code_bg_color     = Palette::BG_EXTREME;

    // Window chrome
    visuals.window_corner_radius = R4;
    visuals.window_stroke = Stroke::new(1.0, Palette::STROKE);

    // Text
    visuals.override_text_color = Some(Palette::FG);
    visuals.hyperlink_color = Palette::ACCENT;

    // Selection
    visuals.selection.bg_fill  = Palette::accent_faint();
    visuals.selection.stroke   = Stroke::new(1.0, Palette::accent_line());

    // Widgets — noninteractive (labels, separators)
    visuals.widgets.noninteractive.bg_fill      = Palette::BG_RAISED;
    visuals.widgets.noninteractive.weak_bg_fill = Palette::BG_FAINT;
    visuals.widgets.noninteractive.bg_stroke    = Stroke::new(1.0, Palette::STROKE_FAINT);
    visuals.widgets.noninteractive.fg_stroke    = Stroke::new(1.0, Palette::FG_MUTED);
    visuals.widgets.noninteractive.corner_radius = R3;

    // Widgets — inactive (default button / combo state)
    visuals.widgets.inactive.bg_fill            = Palette::BG_RAISED;
    visuals.widgets.inactive.weak_bg_fill       = Palette::BG_FAINT;
    visuals.widgets.inactive.bg_stroke          = Stroke::new(1.0, Palette::STROKE);
    visuals.widgets.inactive.fg_stroke          = Stroke::new(1.0, Palette::FG);
    visuals.widgets.inactive.corner_radius      = R3;

    // Widgets — hovered
    visuals.widgets.hovered.bg_fill             = Palette::BG_HOVER;
    visuals.widgets.hovered.weak_bg_fill        = Palette::BG_FAINT;
    visuals.widgets.hovered.bg_stroke           = Stroke::new(1.0, Palette::STROKE_STRONG);
    visuals.widgets.hovered.fg_stroke           = Stroke::new(1.0, Palette::FG_STRONG);
    visuals.widgets.hovered.corner_radius       = R3;

    // Widgets — active (pressed)
    visuals.widgets.active.bg_fill              = Palette::BG_ACTIVE;
    visuals.widgets.active.weak_bg_fill         = Palette::BG_ACTIVE;
    visuals.widgets.active.bg_stroke            = Stroke::new(1.0, Palette::STROKE_BRIGHT);
    visuals.widgets.active.fg_stroke            = Stroke::new(1.0, Palette::ACCENT);
    visuals.widgets.active.corner_radius        = R3;

    // Widgets — open (dropdown open)
    visuals.widgets.open.bg_fill                = Palette::BG_ACTIVE;
    visuals.widgets.open.weak_bg_fill           = Palette::BG_FAINT;
    visuals.widgets.open.bg_stroke              = Stroke::new(1.0, Palette::accent_line());
    visuals.widgets.open.fg_stroke              = Stroke::new(1.0, Palette::ACCENT);
    visuals.widgets.open.corner_radius          = R3;

    ctx.set_visuals(visuals);

    // Style / spacing — Margin::symmetric takes i8 in egui 0.34
    #[allow(deprecated)]
    ctx.style_mut(|style: &mut Style| {
        style.spacing.item_spacing     = egui::vec2(8.0, 6.0);
        style.spacing.button_padding   = egui::vec2(12.0, 5.0);
        style.spacing.window_margin    = Margin::symmetric(24, 20);
        style.spacing.indent           = 16.0;
        style.spacing.slider_width     = 160.0;
        style.spacing.combo_width      = 160.0;
        style.spacing.scroll.bar_width = 8.0;
    });
}
