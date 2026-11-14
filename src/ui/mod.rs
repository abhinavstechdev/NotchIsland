use egui::{
    pos2, vec2, Align2, Color32, ComboBox, FontData, FontDefinitions, FontFamily, FontId, Id,
    Painter, Rect, Rounding, Sense, Shape, Stroke, Ui,
};
use crate::physics::{generate_notch_path_points, generate_squircle_points, IslandMorphSpring};
use crate::services::{PlaybackStatus, ServiceManager};

/// Registers authentic Geist and Geist Mono font families extracted from NotchPixel
pub fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "Geist-Regular".to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Geist-Regular.ttf")),
    );
    fonts.font_data.insert(
        "Geist-Medium".to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Geist-Medium.ttf")),
    );
    fonts.font_data.insert(
        "Geist-Bold".to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Geist-Bold.ttf")),
    );
    fonts.font_data.insert(
        "GeistMono-Regular".to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/GeistMono-Regular.ttf")),
    );

    if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
        family.insert(0, "Geist-Regular".to_owned());
        family.insert(1, "Geist-Medium".to_owned());
    }
    if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
        family.insert(0, "GeistMono-Regular".to_owned());
    }

    ctx.set_fonts(fonts);
}

// Authentic NotchPixel Design System Tokens (from assets/deck/style.css)
pub const COLOR_ACCENT: Color32 = Color32::from_rgb(0x5e, 0xe9, 0xff); // #5ee9ff (Cyan)
pub const COLOR_BG_BLACK: Color32 = Color32::from_rgb(0x00, 0x00, 0x00); // #000000 (Pitch Black)
pub const COLOR_LINE: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 26); // rgba(255,255,255,0.1)
pub const COLOR_LINE_DASH: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 38);
pub const COLOR_GLASS: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 18); // rgba(255,255,255,0.07)
pub const COLOR_GLASS_HI: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 33); // rgba(255,255,255,0.13)
pub const COLOR_INK: Color32 = Color32::from_rgb(0xf4, 0xf4, 0xf5); // #f4f4f5 (White Ink)
pub const COLOR_DIM: Color32 = Color32::from_rgba_premultiplied(244, 244, 245, 142); // rgba(244,244,245,0.56)
pub const COLOR_FAINT: Color32 = Color32::from_rgba_premultiplied(244, 244, 245, 82); // rgba(244,244,245,0.32)
pub const COLOR_WARM: Color32 = Color32::from_rgb(0xf5, 0xb8, 0x4b); // #f5b84b (Amber)
pub const COLOR_OK: Color32 = Color32::from_rgb(0x4b, 0xe0, 0x8a); // #4be08a (Emerald Green)
pub const COLOR_HOT: Color32 = Color32::from_rgb(0xff, 0x5d, 0x5d); // #ff5d5d (Red)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IslandMode {
    Compact,
    Expanded,
    Slim,
}

/// The exact 10 primary tabs matching the authentic NotchPixel bar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardTab {
    Music,
    Websites,
    Notifications,
    Shelf,
    Clipboard,
    Widgets,
    Controls,
    Tools,
    Deck,
    Calls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotchStyle {
    FloatingIsland,
    BezelNotch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeSurface {
    Black,
    Graphite,
    Glass,
    Acrylic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetKind {
    Cat,
    Bunny,
}

#[derive(Debug, Clone)]
pub struct DesktopPet {
    pub kind: PetKind,
    pub is_happy: bool,
    pub heart_anim: f32,
    pub blink_timer: f32,
    pub is_blinking: bool,
}

impl Default for DesktopPet {
    fn default() -> Self {
        Self {
            kind: PetKind::Cat,
            is_happy: false,
            heart_anim: 0.0,
            blink_timer: 0.0,
            is_blinking: false,
        }
    }
}

impl DesktopPet {
    pub fn update(&mut self, dt: f32) {
        self.blink_timer += dt;
        if self.blink_timer > 3.5 {
            self.is_blinking = true;
            if self.blink_timer > 3.7 {
                self.is_blinking = false;
                self.blink_timer = 0.0;
            }
        }

        if self.is_happy {
            self.heart_anim += dt * 3.0;
            if self.heart_anim > 2.0 {
                self.is_happy = false;
                self.heart_anim = 0.0;
            }
        }
    }

    pub fn face_str(&self) -> &'static str {
        match (self.kind, self.is_happy, self.is_blinking) {
            (PetKind::Cat, true, _) => "( ≧ ﻌ ≦ ) ♡",
            (PetKind::Cat, false, true) => "( - ﻌ - )",
            (PetKind::Cat, false, false) => "( ^ ﻌ ^ )",
            (PetKind::Bunny, true, _) => "( ᐢ> ̫ <ᐢ ) ♡",
            (PetKind::Bunny, false, true) => "( ᐢ- ̫ -ᐢ )",
            (PetKind::Bunny, false, false) => "( ᐢ. ̫ .ᐢ )",
        }
    }

    pub fn on_click(&mut self) {
        self.is_happy = true;
        self.heart_anim = 0.0;
    }
}

pub struct IslandUi {
    pub mode: IslandMode,
    pub previous_mode: IslandMode,
    pub tab: CardTab,
    pub notch_style: NotchStyle,
    pub surface: ThemeSurface,
    pub pet: DesktopPet,
    pub show_pet: bool,
    pub morph: IslandMorphSpring,
    pub is_hovered: bool,
    pub anim_time: f32,
    pub auto_expand_on_hover: bool,
    pub show_settings: bool,
    pub settings_category: usize,

    // Toggles & state for exact tabs
    pub wifi_on: bool,
    pub bluetooth_on: bool,
    pub night_light_on: bool,
    pub focus_assist_on: bool,
    pub battery_saver_on: bool,
    pub power_mode: usize, // 0 = Best power, 1 = Balanced, 2 = Best performance
    pub clipboard_filter: usize, // 0 = All, 1 = Images, 2 = Snippets
    pub selected_speaker: usize,
    pub selected_mic: usize,
    pub speaker_tested: bool,
    pub mic_tested: bool,
}

impl Default for IslandUi {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandUi {
    pub fn new() -> Self {
        Self {
            mode: IslandMode::Compact,
            previous_mode: IslandMode::Compact,
            tab: CardTab::Music,
            notch_style: NotchStyle::FloatingIsland,
            surface: ThemeSurface::Black,
            pet: DesktopPet::default(),
            show_pet: true,
            morph: IslandMorphSpring::new(230.0, 38.0, 19.0),
            is_hovered: false,
            anim_time: 0.0,
            auto_expand_on_hover: true,
            show_settings: false,
            settings_category: 0,

            wifi_on: true,
            bluetooth_on: true,
            night_light_on: false,
            focus_assist_on: false,
            battery_saver_on: false,
            power_mode: 1, // Balanced
            clipboard_filter: 0,
            selected_speaker: 0,
            selected_mic: 0,
            speaker_tested: false,
            mic_tested: false,
        }
    }

    pub fn set_mode(&mut self, mode: IslandMode) {
        if self.mode != mode {
            self.previous_mode = self.mode;
            self.mode = mode;
        }

        match self.mode {
            IslandMode::Compact => {
                self.morph.set_target_geometry(230.0, 38.0, 19.0);
            }
            IslandMode::Expanded => {
                self.morph.set_target_geometry(540.0, 280.0, 24.0);
            }
            IslandMode::Slim => {
                self.morph.set_target_geometry(380.0, 28.0, 14.0);
            }
        }
    }

    pub fn update_physics(&mut self, dt: f32) {
        self.anim_time += dt;
        self.pet.update(dt);
        self.morph.update(dt);
    }

    pub fn render(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        let dt = ui.input(|i| i.unstable_dt).clamp(0.001, 0.050);
        self.update_physics(dt);

        let screen_rect = ui.available_rect_before_wrap();
        let current_size = self.morph.current_size();
        let island_top = if self.notch_style == NotchStyle::BezelNotch {
            0.0
        } else {
            10.0
        };
        let island_center_x = screen_rect.center().x;
        let island_rect = Rect::from_center_size(
            pos2(island_center_x, island_top + current_size.y * 0.5),
            current_size,
        );

        let island_id = Id::new("notch_island_interaction_main");
        let response = ui.interact(island_rect, island_id, Sense::click());
        self.is_hovered = response.hovered();

        if self.auto_expand_on_hover && self.mode == IslandMode::Compact && response.hovered() && !response.clicked() {
            self.set_mode(IslandMode::Expanded);
        }

        if response.clicked() {
            match self.mode {
                IslandMode::Compact => self.set_mode(IslandMode::Expanded),
                IslandMode::Slim => self.set_mode(IslandMode::Compact),
                IslandMode::Expanded => {}
            }
        }

        let painter = ui.painter_at(island_rect.expand(16.0));

        // 1. Paint background with squircle continuous curvature
        self.paint_background(&painter, island_rect);

        // 2. Render Desktop Pet (perched on top edge)
        if self.show_pet && self.mode != IslandMode::Slim && !self.show_settings {
            self.render_pet_badge(ui, &painter, island_rect);
        }

        // 3. Paint active view content
        let content_rect = island_rect.shrink2(vec2(14.0, 10.0));

        if current_size.y >= 26.0 {
            match self.mode {
                IslandMode::Compact => {
                    self.paint_compact_view(&painter, content_rect, services);
                }
                IslandMode::Slim => {
                    self.paint_slim_view(&painter, content_rect, services);
                }
                IslandMode::Expanded => {
                    if current_size.y > 100.0 {
                        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(content_rect), |child_ui| {
                            if self.show_settings {
                                self.render_settings_view(child_ui, services);
                            } else {
                                self.render_expanded_view(child_ui, content_rect, services);
                            }
                        });
                    }
                }
            }
        }

        if !self.morph.is_settled()
            || services.media.status == PlaybackStatus::Playing
            || services.timer.is_running
        {
            ui.ctx().request_repaint();
        }
    }

    fn paint_background(&self, painter: &Painter, rect: Rect) {
        let corner_radius = self.morph.corner_radius.current;
        let opacity = self.morph.opacity.current.clamp(0.0, 1.0);

        let bg_color = match self.surface {
            ThemeSurface::Black => Color32::from_rgba_premultiplied(0, 0, 0, (252.0 * opacity) as u8),
            ThemeSurface::Graphite => Color32::from_rgba_premultiplied(18, 18, 22, (248.0 * opacity) as u8),
            ThemeSurface::Glass => Color32::from_rgba_premultiplied(12, 16, 24, (230.0 * opacity) as u8),
            ThemeSurface::Acrylic => Color32::from_rgba_premultiplied(15, 15, 20, (215.0 * opacity) as u8),
        };

        let border_color = Color32::from_rgba_premultiplied(255, 255, 255, (26.0 * opacity) as u8);
        let shadow_color = Color32::from_rgba_premultiplied(0, 0, 0, (110.0 * opacity) as u8);

        if self.notch_style == NotchStyle::FloatingIsland {
            let shadow_rect = rect.expand(4.0).translate(vec2(0.0, 3.0));
            let shadow_pts = generate_squircle_points(shadow_rect, corner_radius + 2.0, 10);
            painter.add(Shape::convex_polygon(shadow_pts, shadow_color, Stroke::NONE));
        }

        let points = if self.notch_style == NotchStyle::BezelNotch {
            generate_notch_path_points(rect, 14.0, corner_radius, 10)
        } else {
            generate_squircle_points(rect, corner_radius, 12)
        };

        painter.add(Shape::convex_polygon(
            points,
            bg_color,
            Stroke::new(1.0_f32, border_color),
        ));
    }

    fn render_pet_badge(&mut self, ui: &mut Ui, painter: &Painter, island_rect: Rect) {
        let pet_bounce = (self.anim_time * 2.5).sin() * 1.5;
        let pet_rect = Rect::from_center_size(
            pos2(island_rect.left() + 26.0, island_rect.top() - 6.0 + pet_bounce),
            vec2(74.0, 20.0),
        );

        let pet_resp = ui.interact(pet_rect, Id::new("desktop_pet_click"), Sense::click());
        if pet_resp.clicked() {
            self.pet.on_click();
        }

        let pet_bg = if pet_resp.hovered() { COLOR_GLASS_HI } else { COLOR_GLASS };
        painter.rect_filled(pet_rect, Rounding::same(10.0), pet_bg);
        painter.rect_stroke(pet_rect, Rounding::same(10.0), Stroke::new(1.0_f32, COLOR_LINE));

        let text_color = if self.pet.is_happy { COLOR_ACCENT } else { COLOR_INK };
        painter.text(
            pet_rect.center(),
            Align2::CENTER_CENTER,
            self.pet.face_str(),
            FontId::proportional(10.5),
            text_color,
        );
    }

    fn paint_compact_view(&self, painter: &Painter, rect: Rect, services: &ServiceManager) {
        let track = services.media.current_track();
        let is_playing = services.media.status == PlaybackStatus::Playing;

        let left_start = rect.left() + 10.0;
        if services.timer.is_running {
            painter.text(
                pos2(left_start, rect.center().y),
                Align2::LEFT_CENTER,
                format!("⏱️ {}", services.timer.formatted_remaining()),
                FontId::monospace(11.5),
                COLOR_WARM,
            );
        } else {
            for i in 0..3 {
                let phase = self.anim_time * 6.0 + (i as f32) * 1.5;
                let bar_h = if is_playing { 6.0 + 8.0 * (phase.sin().abs()) } else { 4.0 };
                let bar_x = left_start + (i as f32) * 4.0;
                let bar_rect = Rect::from_center_size(pos2(bar_x, rect.center().y), vec2(2.5, bar_h));
                painter.rect_filled(bar_rect, Rounding::same(1.0), COLOR_ACCENT);
            }

            let text_rect_left = left_start + 18.0;
            let snippet = if track.title.len() > 16 {
                format!("{}...", &track.title[..14])
            } else {
                track.title.clone()
            };

            painter.text(
                pos2(text_rect_left, rect.center().y),
                Align2::LEFT_CENTER,
                snippet,
                FontId::proportional(12.0),
                COLOR_INK,
            );
        }

        let right_center = pos2(rect.right() - 8.0, rect.center().y);
        let mut glyphs = String::new();
        if services.indicators.caps_lock { glyphs.push_str("🔠 "); }
        if services.indicators.mic_active { glyphs.push_str("🔴 "); }
        if services.controls.is_muted {
            glyphs.push_str("🔇");
        } else if is_playing {
            glyphs.push_str("🎵");
        } else {
            glyphs.push_str("⏸");
        }

        painter.text(right_center, Align2::RIGHT_CENTER, glyphs, FontId::proportional(11.0), COLOR_DIM);
    }

    fn paint_slim_view(&self, painter: &Painter, rect: Rect, services: &ServiceManager) {
        let fps_text = format!("{:.0} FPS", services.telemetry.fps);
        let cpu_text = format!("CPU {:.0}%", services.telemetry.cpu_usage);
        let ram_text = format!("RAM {:.1}G", services.telemetry.ram_used_gb);

        let items = [
            ("⚡", fps_text, COLOR_ACCENT),
            ("💻", cpu_text, COLOR_INK),
            ("🧠", ram_text, COLOR_WARM),
        ];

        let spacing = rect.width() / 3.0;
        for (i, (icon, text, color)) in items.iter().enumerate() {
            let cx = rect.left() + (i as f32 + 0.5) * spacing;
            painter.text(
                pos2(cx, rect.center().y),
                Align2::CENTER_CENTER,
                format!("{} {}", icon, text),
                FontId::proportional(11.5),
                *color,
            );
        }
    }

    /// Renders the authentic expanded card with exact 10-tab bar + Settings and Theme on the right
    fn render_expanded_view(&mut self, ui: &mut Ui, _rect: Rect, services: &mut ServiceManager) {
        ui.vertical(|ui| {
            // --- Exact 10-Tab Navigation Bar ---
            ui.horizontal(|ui| {
                let tabs = [
                    (CardTab::Music, "🎵", "Music"),
                    (CardTab::Websites, "🌐", "Websites"),
                    (CardTab::Notifications, "🔔", "Notices"),
                    (CardTab::Shelf, "📁", "Shelf"),
                    (CardTab::Clipboard, "📋", "Clips"),
                    (CardTab::Widgets, "⊞", "Widgets"),
                    (CardTab::Controls, "🎚️", "Controls"),
                    (CardTab::Tools, "⠿", "Tools"),
                    (CardTab::Deck, "📱", "Deck"),
                    (CardTab::Calls, "🎧", "Calls"),
                ];

                for (tab_kind, icon, tooltip) in tabs {
                    let is_active = self.tab == tab_kind;
                    let (bg, text_col) = if is_active {
                        (COLOR_GLASS_HI, COLOR_ACCENT)
                    } else {
                        (Color32::TRANSPARENT, COLOR_DIM)
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(icon).color(text_col).size(13.0),
                    )
                    .fill(bg)
                    .rounding(Rounding::same(6.0));

                    if ui.add(btn).on_hover_text(tooltip).clicked() {
                        self.tab = tab_kind;
                    }
                }

                // Right side: Theme button, Gear (Settings), and Collapse
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Collapse button
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("▲").color(COLOR_DIM).size(11.0),
                            )
                            .fill(COLOR_GLASS)
                            .rounding(Rounding::same(12.0)),
                        )
                        .on_hover_text("Collapse Notch")
                        .clicked()
                    {
                        self.set_mode(IslandMode::Compact);
                    }

                    // Settings Gear Button
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("⚙").color(COLOR_DIM).size(13.0),
                            )
                            .fill(COLOR_GLASS)
                            .rounding(Rounding::same(6.0)),
                        )
                        .on_hover_text("Settings")
                        .clicked()
                    {
                        self.show_settings = true;
                    }

                    // Theme Surface Switcher Button
                    let theme_icon = match self.surface {
                        ThemeSurface::Black => "⬛",
                        ThemeSurface::Graphite => "◼",
                        ThemeSurface::Glass => "🔹",
                        ThemeSurface::Acrylic => "▫",
                    };
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(theme_icon).color(COLOR_DIM).size(11.0),
                            )
                            .fill(COLOR_GLASS)
                            .rounding(Rounding::same(6.0)),
                        )
                        .on_hover_text("Cycle Surface Theme: Black / Graphite / Glass / Acrylic")
                        .clicked()
                    {
                        self.surface = match self.surface {
                            ThemeSurface::Black => ThemeSurface::Graphite,
                            ThemeSurface::Graphite => ThemeSurface::Glass,
                            ThemeSurface::Glass => ThemeSurface::Acrylic,
                            ThemeSurface::Acrylic => ThemeSurface::Black,
                        };
                    }
                });
            });

            ui.add_space(3.0);
            ui.separator();
            ui.add_space(3.0);

            // --- Body: Render Selected Tab View ---
            match self.tab {
                CardTab::Music => self.render_tab_music(ui, services),
                CardTab::Websites => self.render_tab_websites(ui, services),
                CardTab::Notifications => self.render_tab_notifications(ui, services),
                CardTab::Shelf => self.render_tab_shelf(ui, services),
                CardTab::Clipboard => self.render_tab_clipboard(ui, services),
                CardTab::Widgets => self.render_tab_widgets(ui, services),
                CardTab::Controls => self.render_tab_controls(ui, services),
                CardTab::Tools => self.render_tab_tools(ui, services),
                CardTab::Deck => self.render_tab_deck(ui, services),
                CardTab::Calls => self.render_tab_calls(ui, services),
            }
        });
    }

    // ==========================================
    // TAB 1: MUSIC (Image 2)
    // ==========================================
    fn render_tab_music(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        let track = services.media.current_track().clone();
        let is_playing = services.media.status == PlaybackStatus::Playing;

        ui.horizontal(|ui| {
            // Album art with corner music badge
            let art_size = vec2(76.0, 76.0);
            let (art_rect, _resp) = ui.allocate_exact_size(art_size, Sense::hover());
            let painter = ui.painter_at(art_rect);

            painter.rect_filled(art_rect, Rounding::same(10.0), Color32::from_rgb(22, 22, 28));
            painter.rect_stroke(art_rect, Rounding::same(10.0), Stroke::new(1.0_f32, COLOR_LINE));

            let center = art_rect.center();
            painter.circle_filled(center, 22.0, Color32::from_rgb(35, 38, 50));
            painter.circle_filled(center, 8.0, COLOR_ACCENT);

            // Mini music note badge in bottom right corner
            let badge_rect = Rect::from_center_size(
                pos2(art_rect.right() - 8.0, art_rect.bottom() - 8.0),
                vec2(16.0, 16.0),
            );
            painter.rect_filled(badge_rect, Rounding::same(4.0), Color32::BLACK);
            painter.text(badge_rect.center(), Align2::CENTER_CENTER, "🎵", FontId::proportional(9.0), COLOR_INK);

            ui.add_space(10.0);

            // Metadata, Scrubber with elapsed & remaining, Transport controls
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(&track.title).color(COLOR_INK).size(14.0).strong());
                ui.label(egui::RichText::new(&track.artist).color(COLOR_DIM).size(12.0));

                ui.add_space(4.0);

                // Progress scrubber
                let mut progress = track.progress_ratio();
                if ui.add(egui::Slider::new(&mut progress, 0.0..=1.0).show_value(false).trailing_fill(true)).changed() {
                    services.media.seek_ratio(progress);
                }

                // Elapsed time and remaining time in negative format (-X:XX)
                let elapsed_str = track.formatted_current_time();
                let rem_secs = (track.duration_secs - track.current_pos_secs).max(0.0);
                let rem_str = format!("-{:02}:{:02}", (rem_secs / 60.0) as u32, (rem_secs % 60.0) as u32);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(elapsed_str).color(COLOR_FAINT).size(10.5));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(rem_str).color(COLOR_FAINT).size(10.5));
                    });
                });

                ui.add_space(2.0);

                // Transport Controls
                ui.horizontal(|ui| {
                    if ui.button("⏮").clicked() { services.media.previous(); }
                    let play_icon = if is_playing { "⏸" } else { "▶" };
                    if ui.add(egui::Button::new(egui::RichText::new(play_icon).color(Color32::BLACK).strong()).fill(COLOR_ACCENT)).clicked() {
                        services.media.toggle_play_pause();
                    }
                    if ui.button("⏭").clicked() { services.media.next(); }

                    ui.add_space(16.0);
                    ui.label("🔊");
                    let mut vol = services.controls.volume;
                    if ui.add(egui::Slider::new(&mut vol, 0.0..=1.0).show_value(false)).changed() {
                        services.controls.set_volume(vol);
                    }
                    ui.label(egui::RichText::new(format!("{}%", services.controls.volume_percent())).color(COLOR_DIM).size(10.5));
                });
            });
        });
    }

    // ==========================================
    // TAB 2: WEBSITES (Globe, Image 3)
    // ==========================================
    fn render_tab_websites(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Add a site: any page, signed in, one click away.").color(COLOR_DIM).size(12.0));
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let sites = services.websites.sites.clone();
                for site in &sites {
                    let card_btn = egui::Button::new(
                        egui::RichText::new(format!("{}\n{}", site.icon_letter, site.name)).color(COLOR_INK).size(11.5),
                    )
                    .fill(COLOR_GLASS)
                    .rounding(Rounding::same(8.0))
                    .min_size(vec2(95.0, 68.0));

                    if ui.add(card_btn).clicked() {
                        services.websites.launch(&site.url);
                    }
                    ui.add_space(6.0);
                }

                // Dashed "+ Add a site" card
                let add_btn = egui::Button::new(
                    egui::RichText::new("+\nAdd a site").color(COLOR_FAINT).size(11.0),
                )
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::new(1.0_f32, COLOR_LINE_DASH))
                .rounding(Rounding::same(8.0))
                .min_size(vec2(95.0, 68.0));

                if ui.add(add_btn).clicked() {
                    services.websites.add_site("GitHub", "https://github.com");
                }
            });
        });
    }

    // ==========================================
    // TAB 3: NOTIFICATIONS (Bell, Image 4)
    // ==========================================
    fn render_tab_notifications(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        if services.notifications.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(14.0);
                ui.label(egui::RichText::new("🔔").color(COLOR_DIM).size(36.0));
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Nothing yet").color(COLOR_INK).size(14.0).strong());
                ui.label(egui::RichText::new("What the island says lands here. Click one to open it.").color(COLOR_FAINT).size(11.5));
                ui.add_space(8.0);
                if ui.button(egui::RichText::new("Test notice").color(COLOR_ACCENT).size(11.0)).clicked() {
                    services.notifications.add("NotchPixel", "Welcome to NotchPixel", "All systems operational.", "✨");
                }
            });
        } else {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("Notifications ({})", services.notifications.items.len())).color(COLOR_INK).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Clear all").clicked() {
                            services.notifications.clear();
                        }
                    });
                });
                ui.add_space(4.0);
                let items = services.notifications.items.clone();
                egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                    for item in items {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&item.app_name).color(COLOR_ACCENT).size(10.5));
                            ui.label(egui::RichText::new(&item.title).color(COLOR_INK).size(11.5).strong());
                            ui.label(egui::RichText::new(&item.body).color(COLOR_DIM).size(11.0));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("✕").clicked() {
                                    services.notifications.remove(item.id);
                                }
                            });
                        });
                    }
                });
            });
        }
    }

    // ==========================================
    // TAB 4: SHELF (Folder, Image 5)
    // ==========================================
    fn render_tab_shelf(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Shelf").color(COLOR_INK).size(13.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("📱 Send to phone").clicked() {
                        self.tab = CardTab::Deck;
                    }
                });
            });

            ui.add_space(4.0);

            // Dashed drop zone box
            let drop_size = vec2(ui.available_width(), 76.0);
            let (drop_rect, _resp) = ui.allocate_exact_size(drop_size, Sense::hover());
            let painter = ui.painter_at(drop_rect);

            painter.rect_stroke(drop_rect, Rounding::same(8.0), Stroke::new(1.2_f32, COLOR_LINE_DASH));
            painter.text(
                pos2(drop_rect.center().x, drop_rect.center().y - 8.0),
                Align2::CENTER_CENTER,
                "Drop files here",
                FontId::proportional(12.5),
                COLOR_INK,
            );
            painter.text(
                pos2(drop_rect.center().x, drop_rect.center().y + 10.0),
                Align2::CENTER_CENTER,
                "They wait here until you need them",
                FontId::proportional(11.0),
                COLOR_FAINT,
            );

            ui.add_space(4.0);

            // Staged items preview
            let items = services.file_shelf.get_items();
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{} staged items", items.len())).color(COLOR_FAINT).size(10.5));
                if ui.button("➕ Sample").clicked() {
                    services.file_shelf.add_file("document.pdf", vec![0x25, 0x50, 0x44, 0x46]);
                }
                if ui.button("🗑 Clear").clicked() {
                    services.file_shelf.clear();
                }
            });
        });
    }

    // ==========================================
    // TAB 5: CLIPBOARD (Image 6)
    // ==========================================
    fn render_tab_clipboard(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.vertical(|ui| {
            // Filter Pills & Clear button
            ui.horizontal(|ui| {
                let filters = ["All", "Images", "Snippets"];
                for (idx, name) in filters.iter().enumerate() {
                    let is_active = self.clipboard_filter == idx;
                    let (bg, col) = if is_active { (COLOR_GLASS_HI, COLOR_ACCENT) } else { (Color32::TRANSPARENT, COLOR_DIM) };
                    if ui.add(egui::Button::new(egui::RichText::new(*name).color(col).size(11.0)).fill(bg)).clicked() {
                        self.clipboard_filter = idx;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🗑 Clear").clicked() {
                        services.clipboard.clear_unpinned();
                    }
                });
            });

            ui.add_space(4.0);

            // Snippet cards
            let snippets = services.clipboard.snippets.clone();
            egui::ScrollArea::vertical().max_height(130.0).show(ui, |ui| {
                if snippets.is_empty() {
                    ui.label(egui::RichText::new("Clipboard is empty.").color(COLOR_FAINT));
                }

                for snip in snippets {
                    ui.horizontal(|ui| {
                        let pin_icon = if snip.is_pinned { "📌" } else { "📍" };
                        if ui.button(pin_icon).clicked() {
                            services.clipboard.toggle_pin(snip.id);
                        }

                        ui.label(egui::RichText::new(snip.preview(28)).color(COLOR_INK).size(11.5));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("✕").clicked() { services.clipboard.remove(snip.id); }
                            if ui.button("Copy").clicked() { services.clipboard.copy_to_clipboard(snip.id); }
                        });
                    });
                }
            });
        });
    }

    // ==========================================
    // TAB 6: WIDGETS (4-Bento Grid, Image 7)
    // ==========================================
    fn render_tab_widgets(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        let bento_w = (ui.available_width() - 8.0) * 0.5;
        let bento_h = 70.0;

        ui.vertical(|ui| {
            // Row 1: Date & Focus Timer
            ui.horizontal(|ui| {
                // Bento 1: Date
                ui.group(|ui| {
                    ui.set_width(bento_w);
                    ui.set_height(bento_h);
                    ui.label(egui::RichText::new("Saturday, Nov 14").color(COLOR_INK).size(12.0).strong());
                    ui.horizontal(|ui| {
                        for d in ["M", "T", "W", "T", "F", "S", "S"] {
                            let is_today = d == "S";
                            let col = if is_today { COLOR_ACCENT } else { COLOR_FAINT };
                            ui.label(egui::RichText::new(d).color(col).size(10.5));
                        }
                    });
                });

                // Bento 2: Focus Timer
                ui.group(|ui| {
                    ui.set_width(bento_w);
                    ui.set_height(bento_h);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(services.timer.formatted_remaining()).color(COLOR_WARM).size(14.0).strong());
                        let p_label = if services.timer.is_running { "⏸" } else { "▶" };
                        if ui.button(p_label).clicked() { services.timer.toggle_start_pause(); }
                        if ui.button("+").clicked() { services.timer.add_minutes(5.0); }
                    });
                    ui.label(egui::RichText::new("25+5 min focus").color(COLOR_FAINT).size(10.5));
                });
            });

            ui.add_space(4.0);

            // Row 2: Weather & Telemetry
            ui.horizontal(|ui| {
                // Bento 3: Weather
                ui.group(|ui| {
                    ui.set_width(bento_w);
                    ui.set_height(bento_h);
                    ui.label(egui::RichText::new("⛅ 21°C Partly Cloudy").color(COLOR_INK).size(11.5));
                    ui.label(egui::RichText::new("Cached location").color(COLOR_FAINT).size(10.0));
                });

                // Bento 4: Telemetry Stats
                ui.group(|ui| {
                    ui.set_width(bento_w);
                    ui.set_height(bento_h);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("CPU {:.0}%", services.telemetry.cpu_usage)).color(COLOR_ACCENT).size(10.5));
                        ui.label(egui::RichText::new(format!("RAM {:.1}G", services.telemetry.ram_used_gb)).color(COLOR_WARM).size(10.5));
                        ui.label(egui::RichText::new(format!("{:.0} FPS", services.telemetry.fps)).color(COLOR_OK).size(10.5));
                    });
                    ui.label(egui::RichText::new("↓ 1.2 MB/s  ↑ 240 KB/s").color(COLOR_FAINT).size(10.0));
                });
            });
        });
    }

    // ==========================================
    // TAB 7: CONTROLS (Sliders & Toggles, Image 8)
    // ==========================================
    fn render_tab_controls(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.horizontal(|ui| {
            // Two vertical pill sliders (Brightness & Volume)
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("🔆").size(12.0));
                let mut b = services.controls.brightness;
                if ui.add(egui::Slider::new(&mut b, 0.0..=1.0).show_value(false).vertical().trailing_fill(true)).changed() {
                    services.controls.set_brightness(b);
                }
                ui.label(egui::RichText::new(format!("{}%", services.controls.brightness_percent())).color(COLOR_FAINT).size(9.5));
            });

            ui.add_space(4.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("🔊").size(12.0));
                let mut v = services.controls.volume;
                if ui.add(egui::Slider::new(&mut v, 0.0..=1.0).show_value(false).vertical().trailing_fill(true)).changed() {
                    services.controls.set_volume(v);
                }
                ui.label(egui::RichText::new(format!("{}%", services.controls.volume_percent())).color(COLOR_FAINT).size(9.5));
            });

            ui.add_space(10.0);

            // Right: 6 Toggle Tiles & Power Mode Pills
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    toggle_chip(ui, &mut self.wifi_on, "📶 Wi-Fi");
                    toggle_chip(ui, &mut self.bluetooth_on, "ᛒ Bluetooth");
                    toggle_chip(ui, &mut self.night_light_on, "🌙 Night");
                });
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    toggle_chip(ui, &mut self.focus_assist_on, "🎯 Focus");
                    let mut is_muted = services.controls.is_muted;
                    if toggle_chip(ui, &mut is_muted, "🔇 Mute") {
                        services.controls.toggle_mute();
                    }
                    toggle_chip(ui, &mut self.battery_saver_on, "🔋 Battery");
                });

                ui.add_space(6.0);

                // Power Mode Selector
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Power:").color(COLOR_FAINT).size(10.0));
                    if ui.selectable_label(self.power_mode == 0, "🌱 Eco").clicked() { self.power_mode = 0; }
                    if ui.selectable_label(self.power_mode == 1, "⚖ Balanced").clicked() { self.power_mode = 1; }
                    if ui.selectable_label(self.power_mode == 2, "⚡ Best").clicked() { self.power_mode = 2; }
                });
            });
        });
    }

    // ==========================================
    // TAB 8: TOOLS (6-Dots, Image 9)
    // ==========================================
    fn render_tab_tools(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.vertical(|ui| {
            // Quick action capture buttons
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("📸 Screenshot").fill(COLOR_GLASS).rounding(Rounding::same(8.0))).clicked() {
                    services.tools.take_screenshot();
                }
                if ui.add(egui::Button::new("✂ Snip").fill(COLOR_GLASS).rounding(Rounding::same(8.0))).clicked() {
                    services.tools.launch_snipping_tool();
                }
                let rec_label = if services.tools.is_recording {
                    format!("⏹ Stop ({:02}:{:02})", services.tools.recording_seconds / 60, services.tools.recording_seconds % 60)
                } else {
                    "🎥 Record".to_string()
                };
                let rec_color = if services.tools.is_recording { COLOR_HOT } else { COLOR_INK };
                if ui.add(egui::Button::new(egui::RichText::new(rec_label).color(rec_color)).fill(COLOR_GLASS).rounding(Rounding::same(8.0))).clicked() {
                    services.tools.toggle_recording();
                }
            });

            if let Some(status) = &services.tools.last_screenshot_status {
                ui.label(egui::RichText::new(status).color(COLOR_OK).size(10.5));
            }

            ui.add_space(4.0);

            // 6 Dashed App Slot Tiles (2x3)
            let slot_w = (ui.available_width() - 12.0) / 3.0;
            let slot_h = 36.0;

            for row in 0..2 {
                ui.horizontal(|ui| {
                    for col in 0..3 {
                        let idx = row * 3 + col;
                        let label = if idx < services.tools.custom_tool_slots.len() {
                            &services.tools.custom_tool_slots[idx]
                        } else {
                            "+ Add"
                        };
                        let btn = egui::Button::new(egui::RichText::new(label).color(COLOR_FAINT).size(10.5))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0_f32, COLOR_LINE_DASH))
                            .rounding(Rounding::same(6.0))
                            .min_size(vec2(slot_w, slot_h));
                        ui.add(btn);
                    }
                });
                ui.add_space(3.0);
            }
        });
    }

    // ==========================================
    // TAB 9: DECK (Phone, Image 10)
    // ==========================================
    fn render_tab_deck(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.horizontal(|ui| {
            let qr = &services.deck_server.qr_matrix;
            let qr_size = 90.0;
            let (qr_rect, _resp) = ui.allocate_exact_size(vec2(qr_size, qr_size), Sense::hover());
            let painter = ui.painter_at(qr_rect);
            painter.rect_filled(qr_rect, Rounding::same(6.0), Color32::WHITE);

            let n = qr.len();
            if n > 0 {
                let cell = qr_size / n as f32;
                for y in 0..n {
                    for x in 0..n {
                        if qr[y][x] {
                            painter.rect_filled(
                                Rect::from_min_size(
                                    pos2(qr_rect.left() + x as f32 * cell, qr_rect.top() + y as f32 * cell),
                                    vec2(cell + 0.3, cell + 0.3),
                                ),
                                Rounding::ZERO,
                                Color32::BLACK,
                            );
                        }
                    }
                }
            }

            ui.add_space(10.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Your phone, as a deck for this PC").color(COLOR_INK).size(12.5).strong());
                ui.label(
                    egui::RichText::new("Scan on the same Wi-Fi to control media, sliders, and hotkeys.")
                        .color(COLOR_FAINT)
                        .size(10.5),
                );
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("Turn on").fill(COLOR_ACCENT).rounding(Rounding::same(6.0))).clicked() {}
                    if ui.button("Get the app").clicked() {}
                });
            });
        });
    }

    // ==========================================
    // TAB 10: CALLS (Headset, Image 11)
    // ==========================================
    fn render_tab_calls(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.horizontal(|ui| {
            // Large Hero Microphone Button
            let is_mic_muted = services.calls.is_mic_muted;
            let mic_color = if is_mic_muted { COLOR_HOT } else { COLOR_OK };
            let mic_label = if is_mic_muted { "🔇 Mic Muted\nClick to unmute" } else { "🎙️ Mic On\nClick to mute" };

            let hero_btn = egui::Button::new(
                egui::RichText::new(mic_label).color(mic_color).size(11.5).strong(),
            )
            .fill(COLOR_GLASS)
            .stroke(Stroke::new(1.2_f32, mic_color))
            .rounding(Rounding::same(10.0))
            .min_size(vec2(130.0, 78.0));

            if ui.add(hero_btn).clicked() {
                services.calls.toggle_mic();
                services.controls.is_mic_muted = services.calls.is_mic_muted;
            }

            ui.add_space(10.0);

            // Audio Device Dropdowns
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Speakers:").color(COLOR_DIM).size(11.0));
                    let speaker_name = services.calls.speakers.get(services.calls.selected_speaker)
                        .map(|s| s.name.as_str())
                        .unwrap_or("Default Speaker");
                    ComboBox::from_id_salt("speaker_select")
                        .selected_text(speaker_name)
                        .show_ui(ui, |ui| {
                            for (idx, spk) in services.calls.speakers.iter().enumerate() {
                                ui.selectable_value(&mut services.calls.selected_speaker, idx, &spk.name);
                            }
                        });
                    let spk_test_label = if services.calls.is_testing_speaker { "Beep..." } else { "Test" };
                    if ui.button(spk_test_label).clicked() { services.calls.test_speaker(); }
                });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Mic Array:").color(COLOR_DIM).size(11.0));
                    let mic_name = services.calls.microphones.get(services.calls.selected_microphone)
                        .map(|m| m.name.as_str())
                        .unwrap_or("Default Mic");
                    ComboBox::from_id_salt("mic_select")
                        .selected_text(mic_name)
                        .show_ui(ui, |ui| {
                            for (idx, mic) in services.calls.microphones.iter().enumerate() {
                                ui.selectable_value(&mut services.calls.selected_microphone, idx, &mic.name);
                            }
                        });
                    let mic_test_label = if services.calls.is_testing_mic { "Listening" } else { "Test" };
                    if ui.button(mic_test_label).clicked() { services.calls.toggle_test_mic(); }
                });

                ui.add_space(4.0);
                let cam_label = if services.calls.camera_active { "📷 Camera Active" } else { "📷 Check camera" };
                if ui.button(cam_label).clicked() {
                    services.calls.check_camera();
                }
            });
        });
    }

    // ==========================================
    // SETTINGS VIEW (Image 12)
    // ==========================================
    fn render_settings_view(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.horizontal(|ui| {
            // Left Sidebar
            let categories = crate::services::SettingsCategory::all();

            ui.vertical(|ui| {
                ui.set_width(120.0);
                for cat in categories {
                    let is_active = services.settings.active_category == *cat;
                    let (bg, col) = if is_active { (COLOR_GLASS_HI, COLOR_ACCENT) } else { (Color32::TRANSPARENT, COLOR_DIM) };
                    let label = format!("{} {}", cat.icon(), cat.label());
                    if ui.add(egui::Button::new(egui::RichText::new(label).color(col).size(10.5)).fill(bg)).clicked() {
                        services.settings.active_category = *cat;
                    }
                }
            });

            ui.separator();

            // Right Panel (General Settings)
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("{} Settings", services.settings.active_category.label())).color(COLOR_INK).size(14.0).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕ Close").clicked() {
                            self.show_settings = false;
                            services.settings.is_window_open = false;
                        }
                    });
                });

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Language:");
                    ui.label(egui::RichText::new(&services.settings.settings.language).color(COLOR_DIM));
                });

                ui.add_space(6.0);
                if ui.checkbox(&mut services.settings.settings.run_at_startup, "Run NotchPixel at Windows sign-in").changed() {
                    services.settings.save();
                }
                ui.add_space(4.0);
                if ui.checkbox(&mut services.settings.settings.slim_under_maximized, "Show slim strip under maximized windows").changed() {
                    services.settings.save();
                }
                ui.add_space(4.0);
                ui.checkbox(&mut self.auto_expand_on_hover, "Open island on mouse hover");

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label("Surface Look:");
                    if ui.selectable_label(self.surface == ThemeSurface::Black, "Black").clicked() {
                        self.surface = ThemeSurface::Black;
                        services.settings.settings.theme = "Pure Black".to_string();
                        services.settings.save();
                    }
                    if ui.selectable_label(self.surface == ThemeSurface::Graphite, "Graphite").clicked() {
                        self.surface = ThemeSurface::Graphite;
                        services.settings.settings.theme = "Graphite".to_string();
                        services.settings.save();
                    }
                    if ui.selectable_label(self.surface == ThemeSurface::Glass, "Glass").clicked() {
                        self.surface = ThemeSurface::Glass;
                        services.settings.settings.theme = "Glass".to_string();
                        services.settings.save();
                    }
                    if ui.selectable_label(self.surface == ThemeSurface::Acrylic, "Acrylic").clicked() {
                        self.surface = ThemeSurface::Acrylic;
                        services.settings.settings.theme = "Acrylic".to_string();
                        services.settings.save();
                    }
                });
            });
        });
    }
}

fn toggle_chip(ui: &mut Ui, state: &mut bool, label: &str) -> bool {
    let (bg, col) = if *state { (COLOR_ACCENT, Color32::BLACK) } else { (COLOR_GLASS, COLOR_DIM) };
    let btn = egui::Button::new(egui::RichText::new(label).color(col).size(10.5))
        .fill(bg)
        .rounding(Rounding::same(6.0))
        .min_size(vec2(80.0, 24.0));
    let clicked = ui.add(btn).clicked();
    if clicked { *state = !*state; }
    clicked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_initial_state() {
        let ui = IslandUi::new();
        assert_eq!(ui.mode, IslandMode::Compact);
        assert_eq!(ui.tab, CardTab::Music);
        assert_eq!(ui.notch_style, NotchStyle::FloatingIsland);
        assert!((ui.morph.width.target - 230.0).abs() < 1e-4);
        assert!((ui.morph.height.target - 38.0).abs() < 1e-4);
    }

    #[test]
    fn test_ui_mode_transitions() {
        let mut ui = IslandUi::new();
        ui.set_mode(IslandMode::Expanded);
        assert_eq!(ui.mode, IslandMode::Expanded);
        assert!((ui.morph.width.target - 540.0).abs() < 1e-4);
        assert!((ui.morph.height.target - 280.0).abs() < 1e-4);

        ui.set_mode(IslandMode::Slim);
        assert_eq!(ui.mode, IslandMode::Slim);
        assert!((ui.morph.width.target - 380.0).abs() < 1e-4);
        assert!((ui.morph.height.target - 28.0).abs() < 1e-4);
    }

    #[test]
    fn test_desktop_pet_interactions() {
        let mut pet = DesktopPet::default();
        assert_eq!(pet.face_str(), "( ^ ﻌ ^ )");
        pet.on_click();
        assert!(pet.is_happy);
        assert_eq!(pet.face_str(), "( ≧ ﻌ ≦ ) ♡");
    }
}
