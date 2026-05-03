use eframe::egui::{self, Color32, CornerRadius, Rect, Response, RichText, Sense, Stroke, Ui, Vec2};

use super::theme::Palette;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DaemonStatus {
    Running,
    Muted,
    Error,
    Disconnected,
}

impl DaemonStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Running      => "ARMED",
            Self::Muted        => "MUTED",
            Self::Error        => "ERROR",
            Self::Disconnected => "OFFLINE",
        }
    }
    pub fn color(self) -> Color32 {
        match self {
            Self::Running      => Palette::OK,
            Self::Muted        => Palette::FG_MUTED,
            Self::Error        => Palette::ERR,
            Self::Disconnected => Palette::WARN,
        }
    }
    pub fn bg(self) -> Color32 {
        match self {
            Self::Running      => Palette::ok_faint(),
            Self::Muted        => Palette::BG_RAISED,
            Self::Error        => Palette::err_faint(),
            Self::Disconnected => Palette::warn_faint(),
        }
    }
    pub fn border(self) -> Color32 {
        match self {
            Self::Running      => Color32::from_rgba_unmultiplied(0x5f, 0xd4, 0x7a, 77),
            Self::Muted        => Palette::STROKE,
            Self::Error        => Color32::from_rgba_unmultiplied(0xe8, 0x5d, 0x3c, 89),
            Self::Disconnected => Color32::from_rgba_unmultiplied(0xe8, 0xa3, 0x17, 89),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NavId {
    Devices,
    Voice,
    Packs,
    Hotkeys,
    Advanced,
}

pub struct NavItem {
    pub id: NavId,
    pub icon: &'static str,
    pub label: &'static str,
}

pub static NAV_ITEMS: &[NavItem] = &[
    NavItem { id: NavId::Devices,  icon: "⊞", label: "DEVICES"  },
    NavItem { id: NavId::Voice,    icon: "🎤", label: "VOICE"   },
    NavItem { id: NavId::Packs,    icon: "📦", label: "PACKS"   },
    NavItem { id: NavId::Hotkeys,  icon: "⌨", label: "HOTKEYS" },
    NavItem { id: NavId::Advanced, icon: "⚙", label: "ADVANCED" },
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BannerKind {
    Error,
    Warn,
    Info,
    Ok,
}

impl BannerKind {
    fn bg(self) -> Color32 {
        match self {
            Self::Error => Palette::err_faint(),
            Self::Warn  => Palette::warn_faint(),
            Self::Info  => Palette::info_faint(),
            Self::Ok    => Palette::ok_faint(),
        }
    }
    fn border(self) -> Color32 {
        match self {
            Self::Error => Color32::from_rgba_unmultiplied(0xe8, 0x5d, 0x3c, 77),
            Self::Warn  => Color32::from_rgba_unmultiplied(0xe8, 0xa3, 0x17, 77),
            Self::Info  => Color32::from_rgba_unmultiplied(0x6a, 0xb8, 0xe8, 77),
            Self::Ok    => Color32::from_rgba_unmultiplied(0x5f, 0xd4, 0x7a, 77),
        }
    }
    fn icon_bg(self) -> Color32 {
        match self {
            Self::Error => Palette::ERR,
            Self::Warn  => Palette::WARN,
            Self::Info  => Palette::INFO,
            Self::Ok    => Palette::OK,
        }
    }
    fn icon_char(self) -> &'static str {
        match self {
            Self::Error => "✕",
            Self::Warn  => "!",
            Self::Info  => "i",
            Self::Ok    => "✓",
        }
    }
}

// ── app_header ────────────────────────────────────────────────────────────────

/// 44px tall header bar: brand mark, title, version, status pill, spacer, action icons.
pub fn app_header(
    ui: &mut Ui,
    version: &str,
    status: DaemonStatus,
    time: f64,
) {
    let desired = Vec2::new(ui.available_width(), 44.0);
    ui.allocate_ui_with_layout(desired, egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.set_min_height(44.0);
        let painter = ui.painter();
        let rect = ui.max_rect();
        painter.rect_filled(rect, 0.0, Palette::BG_PANEL);
        painter.hline(
            rect.x_range(),
            rect.bottom(),
            Stroke::new(1.0, Palette::STROKE_FAINT),
        );

        ui.add_space(14.0);

        // Brand mark — amber-bordered box with "V" inside
        let mark_size = Vec2::splat(18.0);
        let (mark_rect, _) = ui.allocate_exact_size(mark_size, Sense::hover());
        let p = ui.painter();
        p.rect_stroke(mark_rect, CornerRadius::same(2), Stroke::new(1.0, Palette::ACCENT), egui::StrokeKind::Inside);
        // inner frame at inset 3
        let inner = mark_rect.shrink(3.0);
        p.rect_stroke(inner, CornerRadius::ZERO, Stroke::new(1.0, Color32::from_rgba_unmultiplied(0xe8, 0xa3, 0x17, 102)), egui::StrokeKind::Inside);
        p.text(
            mark_rect.center(),
            egui::Align2::CENTER_CENTER,
            "V",
            egui::FontId::proportional(10.0),
            Palette::ACCENT,
        );

        ui.add_space(8.0);
        ui.label(RichText::new("VIBE ATTACK").color(Palette::FG_STRONG).size(12.0).strong());
        ui.add_space(4.0);
        ui.label(RichText::new(version).color(Palette::FG_FAINT).size(11.0));

        ui.add_space(14.0);
        status_pill(ui, status, time);

        // Spacer
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(14.0);
        });
    });
}

// ── status_pill ───────────────────────────────────────────────────────────────

/// 24px inline pill showing daemon state with animated dot.
pub fn status_pill(ui: &mut Ui, status: DaemonStatus, time: f64) {
    let color = status.color();
    let bg    = status.bg();
    let bdr   = status.border();
    let label = status.label();

    let text_galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(11.0),
        color,
    );
    let pill_w = 10.0 + 7.0 + 8.0 + text_galley.size().x + 10.0;
    let pill_h = 24.0;

    let (rect, _) = ui.allocate_exact_size(Vec2::new(pill_w, pill_h), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(12), bg);
    p.rect_stroke(rect, CornerRadius::same(12), Stroke::new(1.0, bdr), egui::StrokeKind::Inside);

    // Animated dot alpha
    let dot_alpha: f32 = match status {
        DaemonStatus::Running => {
            let t = (time * std::f64::consts::PI).sin() as f32;
            0.55 + 0.45 * (t * 0.5 + 0.5)
        }
        DaemonStatus::Disconnected => {
            if (time % 1.0) < 0.5 { 1.0 } else { 0.25 }
        }
        _ => 1.0,
    };
    let dot_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), (dot_alpha * 255.0) as u8);

    let dot_x = rect.left() + 10.0 + 3.5;
    let dot_y = rect.center().y;
    p.circle_filled(egui::pos2(dot_x, dot_y), 3.5, dot_color);

    let text_x = dot_x + 3.5 + 8.0;
    p.galley(egui::pos2(text_x, rect.center().y - text_galley.size().y * 0.5), text_galley, color);
}

// ── side_nav ──────────────────────────────────────────────────────────────────

/// Vertical navigation rail. Collapsed to 52px; expanded on hover to 168px.
pub fn side_nav(ui: &mut Ui, items: &[NavItem], active: &mut NavId) {
    let collapsed_w: f32 = 52.0;
    let available_h = ui.available_height();

    let (rail_rect, _) = ui.allocate_exact_size(
        Vec2::new(collapsed_w, available_h),
        Sense::hover(),
    );

    let p = ui.painter();
    p.rect_filled(rail_rect, 0.0, Palette::BG_PANEL);
    p.vline(
        rail_rect.right(),
        rail_rect.y_range(),
        Stroke::new(1.0, Palette::STROKE_FAINT),
    );

    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rail_rect));
    child_ui.set_clip_rect(rail_rect);

    child_ui.add_space(10.0);

    for item in items {
        let is_active = item.id == *active;
        let item_rect = Rect::from_min_size(
            egui::pos2(rail_rect.left(), child_ui.cursor().top()),
            Vec2::new(collapsed_w, 36.0),
        );

        let sense_resp = child_ui.allocate_rect(item_rect, Sense::click());

        let bg = if is_active {
            Palette::accent_faint()
        } else if sense_resp.hovered() {
            Palette::BG_FAINT
        } else {
            Color32::TRANSPARENT
        };

        let p2 = child_ui.painter();
        p2.rect_filled(item_rect, 0.0, bg);

        // 2px left accent bar for active item
        if is_active {
            p2.rect_filled(
                Rect::from_min_size(item_rect.min, Vec2::new(2.0, item_rect.height())),
                0.0,
                Palette::ACCENT,
            );
        }

        let text_color = if is_active {
            Palette::ACCENT
        } else if sense_resp.hovered() {
            Palette::FG_STRONG
        } else {
            Palette::FG_MUTED
        };

        p2.text(
            egui::pos2(item_rect.left() + 26.0, item_rect.center().y),
            egui::Align2::CENTER_CENTER,
            item.icon,
            egui::FontId::proportional(16.0),
            text_color,
        );

        if sense_resp.clicked() {
            *active = item.id;
        }

        child_ui.add_space(2.0);
    }
}

// ── status_footer ─────────────────────────────────────────────────────────────

/// 30px monospace status strip: LED meter + pipe-separated cells.
pub fn status_footer(
    ui: &mut Ui,
    status: DaemonStatus,
    mic_level: f32,
    model_name: &str,
    uptime: Option<std::time::Duration>,
) {
    let desired = Vec2::new(ui.available_width(), 30.0);
    ui.allocate_ui_with_layout(desired, egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.set_min_height(30.0);
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0.0, Palette::BG_PANEL);
        ui.painter().hline(
            rect.x_range(),
            rect.top(),
            Stroke::new(1.0, Palette::STROKE_FAINT),
        );

        ui.add_space(14.0);

        // LED meter
        led_meter(ui, mic_level, 20);

        ui.add_space(10.0);
        footer_divider(ui);

        footer_cell(ui, "MIC", if mic_level < 0.01 { "—" } else { "LIVE" });
        footer_divider(ui);
        footer_cell(ui, "STATE", status.label());
        footer_divider(ui);
        footer_cell(ui, "MODEL", model_name);

        if let Some(up) = uptime {
            footer_divider(ui);
            let secs = up.as_secs();
            let text = format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60);
            footer_cell(ui, "UP", &text);
        }
    });
}

fn footer_divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 14.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, Palette::STROKE);
    ui.add_space(14.0);
}

fn footer_cell(ui: &mut Ui, label: &str, value: &str) {
    ui.label(RichText::new(label).color(Palette::FG_FAINT).size(10.0));
    ui.add_space(4.0);
    ui.label(RichText::new(value).color(Palette::FG).size(11.0));
    ui.add_space(14.0);
}

// ── led_meter ─────────────────────────────────────────────────────────────────

/// Gap-separated LED segments. Green up to 65%, amber to 85%, red above.
pub fn led_meter(ui: &mut Ui, level: f32, segments: usize) {
    let seg_w: f32 = 8.0;
    let gap: f32   = 2.0;
    let h: f32     = 14.0;
    let total_w = seg_w * segments as f32 + gap * (segments - 1) as f32;

    let (rect, _) = ui.allocate_exact_size(Vec2::new(total_w, h), Sense::hover());
    let p = ui.painter();
    let clamped = level.clamp(0.0, 1.0);

    for i in 0..segments {
        let frac = (i + 1) as f32 / segments as f32;
        let x = rect.left() + i as f32 * (seg_w + gap);
        let seg_rect = Rect::from_min_size(egui::pos2(x, rect.top()), Vec2::new(seg_w, h));

        let on = clamped >= frac;
        let color = if on {
            if frac > 0.85 {
                Palette::ERR
            } else if frac > 0.65 {
                Palette::WARN
            } else {
                Palette::OK
            }
        } else {
            Palette::BG_EXTREME
        };

        p.rect_filled(seg_rect, CornerRadius::same(1), color);
        if !on {
            p.rect_stroke(seg_rect, CornerRadius::same(1), Stroke::new(1.0, Palette::STROKE_FAINT), egui::StrokeKind::Inside);
        }
    }
}

// ── section_header ────────────────────────────────────────────────────────────

/// Section title with amber square bullet, optional subtitle, optional right-side actions.
pub fn section_header(ui: &mut Ui, title: &str, subtitle: Option<&str>) {
    ui.horizontal(|ui| {
        // 4×4 amber square bullet
        let (bullet, _) = ui.allocate_exact_size(Vec2::splat(4.0), Sense::hover());
        ui.painter().rect_filled(bullet, 0.0, Palette::ACCENT);
        ui.add_space(10.0);
        ui.label(RichText::new(title.to_uppercase()).color(Palette::FG_STRONG).size(11.0).strong());
        if let Some(sub) = subtitle {
            ui.add_space(8.0);
            ui.label(RichText::new(sub).color(Palette::FG_FAINT).size(11.0));
        }
    });
}

// ── field_row ─────────────────────────────────────────────────────────────────

/// 140px label column + flexible body column, 28px row height.
pub fn field_row<R>(
    ui: &mut Ui,
    label: &str,
    required: bool,
    body: impl FnOnce(&mut Ui) -> R,
) -> R {
    let label_w: f32 = 140.0;
    let total_w = ui.available_width();

    ui.horizontal(|ui| {
        ui.set_min_height(28.0);

        let mut lbl_text = RichText::new(label.to_uppercase()).color(Palette::FG_MUTED).size(11.0);
        if required {
            // Append asterisk — no native support for mixed-color in one label; keep simple
            lbl_text = lbl_text.color(Palette::FG_MUTED);
        }

        ui.allocate_ui_with_layout(
            Vec2::new(label_w, 28.0),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                if required {
                    ui.label(RichText::new("*").color(Palette::ACCENT).size(11.0));
                    ui.add_space(2.0);
                }
                ui.label(lbl_text);
            },
        );

        ui.add_space(14.0);
        let body_w = (total_w - label_w - 14.0).max(0.0);
        ui.allocate_ui_with_layout(
            Vec2::new(body_w, 28.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| body(ui),
        )
        .inner
    })
    .inner
}

// ── primary_button ────────────────────────────────────────────────────────────

/// Amber-filled, uppercase 11px primary action button.
pub fn primary_button(ui: &mut Ui, label: &str) -> Response {
    let text = RichText::new(label.to_uppercase())
        .color(Palette::ACCENT_FG)
        .size(11.0)
        .strong();

    let btn = egui::Button::new(text)
        .fill(Palette::ACCENT)
        .stroke(Stroke::new(1.0, Palette::ACCENT))
        .corner_radius(CornerRadius::same(3));

    let resp = ui.add(btn);

    // Tint on hover — painter overlay approach since egui doesn't expose per-button hover fill easily
    if resp.hovered() {
        ui.painter().rect_filled(
            resp.rect,
            CornerRadius::same(3),
            Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 15),
        );
    }

    resp
}

// ── kbd ───────────────────────────────────────────────────────────────────────

/// Keyboard key cap chip — inset background, double bottom border.
pub fn kbd(ui: &mut Ui, label: &str) -> Response {
    let text_galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::monospace(11.0),
        Palette::FG_STRONG,
    );
    let w = text_galley.size().x + 12.0;
    let h = 20.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::hover());
    let p = ui.painter();

    p.rect_filled(rect, CornerRadius::same(3), Palette::BG_EXTREME);
    // Top + sides stroke
    p.rect_stroke(
        rect,
        CornerRadius::same(3),
        Stroke::new(1.0, Palette::STROKE_STRONG),
        egui::StrokeKind::Inside,
    );
    // Extra bottom line for the "double bottom border" effect
    p.hline(
        (rect.left() + 1.0)..=(rect.right() - 1.0),
        rect.bottom() + 1.0,
        Stroke::new(1.0, Palette::STROKE_STRONG),
    );

    p.galley(
        egui::pos2(rect.center().x - text_galley.size().x * 0.5, rect.center().y - text_galley.size().y * 0.5),
        text_galley,
        Palette::FG_STRONG,
    );

    resp
}

// ── banner ────────────────────────────────────────────────────────────────────

/// Error/warn/info/ok banner with icon, title, body text, and optional action buttons.
pub fn banner(
    ui: &mut Ui,
    kind: BannerKind,
    title: &str,
    body: &str,
    actions: &[(&str, bool)], // (label, is_primary)
) -> Option<usize> {
    let mut clicked = None;

    let bg  = kind.bg();
    let bdr = kind.border();

    egui::Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.0, bdr))
        .corner_radius(CornerRadius::same(3))
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                // Circular icon
                let icon_size = Vec2::splat(16.0);
                let (icon_rect, _) = ui.allocate_exact_size(icon_size, Sense::hover());
                ui.painter().circle_filled(icon_rect.center(), 8.0, kind.icon_bg());
                ui.painter().text(
                    icon_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    kind.icon_char(),
                    egui::FontId::proportional(9.0),
                    Palette::ACCENT_FG,
                );

                ui.add_space(10.0);

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(title.to_uppercase())
                            .color(Palette::FG_STRONG)
                            .size(11.0)
                            .strong(),
                    );
                    ui.add_space(4.0);
                    ui.label(RichText::new(body).color(Palette::FG).size(12.0));
                    if !actions.is_empty() {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            for (i, (label, is_primary)) in actions.iter().enumerate() {
                                let resp = if *is_primary {
                                    primary_button(ui, label)
                                } else {
                                    ui.button(RichText::new(*label).size(12.0))
                                };
                                if resp.clicked() {
                                    clicked = Some(i);
                                }
                            }
                        });
                    }
                });
            });
        });

    clicked
}
