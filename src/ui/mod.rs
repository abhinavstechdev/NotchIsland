use egui::{
    pos2, vec2, Align2, Color32, FontData, FontDefinitions, FontFamily, FontId, Id, Painter,
    Rect, Rounding, Sense, Shape, Stroke, Ui,
};
use crate::physics::{generate_notch_path_points, generate_squircle_points, IslandMorphSpring};
use crate::services::{PlaybackStatus, ServiceManager, TimerPhase};

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

// Authentic NotchPixel Color Palette (from assets/deck/style.css :root)
pub const COLOR_ACCENT: Color32 = Color32::from_rgb(0x5e, 0xe9, 0xff); // #5ee9ff (Cyan)
pub const COLOR_BG_BLACK: Color32 = Color32::from_rgb(0x00, 0x00, 0x00); // #000000 (Pitch Black)
pub const COLOR_LINE: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 26); // rgba(255,255,255,0.1)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardTab {
    Media,
    Timer,
    FileShelf,
    Clipboard,
    Notes,
    Telemetry,
    Controls,
    Deck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotchStyle {
    FloatingIsland,
    BezelNotch,
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
    pub pet: DesktopPet,
    pub show_pet: bool,
    pub morph: IslandMorphSpring,
    pub is_hovered: bool,
    pub anim_time: f32,
    pub auto_expand_on_hover: bool,
    pub power_plan: usize, // 0 = Balanced, 1 = High Perf, 2 = Battery Saver
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
            tab: CardTab::Media,
            notch_style: NotchStyle::FloatingIsland,
            pet: DesktopPet::default(),
            show_pet: true,
            morph: IslandMorphSpring::new(230.0, 38.0, 19.0),
            is_hovered: false,
            anim_time: 0.0,
            auto_expand_on_hover: true,
            power_plan: 0,
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
                self.morph.set_target_geometry(480.0, 230.0, 24.0);
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

        // Interactivity bounding response
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

        // 2. Render Desktop Pet (sitting perched on top edge or left)
        if self.show_pet && self.mode != IslandMode::Slim {
            self.render_pet_badge(ui, &painter, island_rect);
        }

        // 3. Paint active view content
        let content_rect = island_rect.shrink2(vec2(12.0, 8.0));

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
                            self.render_expanded_view(child_ui, content_rect, services);
                        });
                    }
                }
            }
        }

        // Keep repainting if animations or playback or timer are active
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

        // Pitch Black matching NotchPixel
        let bg_color = Color32::from_rgba_premultiplied(0, 0, 0, (252.0 * opacity) as u8);
        let border_color = Color32::from_rgba_premultiplied(255, 255, 255, (28.0 * opacity) as u8);
        let shadow_color = Color32::from_rgba_premultiplied(0, 0, 0, (120.0 * opacity) as u8);

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
            pos2(island_rect.left() + 24.0, island_rect.top() - 6.0 + pet_bounce),
            vec2(72.0, 20.0),
        );

        let pet_resp = ui.interact(pet_rect, Id::new("desktop_pet_click"), Sense::click());
        if pet_resp.clicked() {
            self.pet.on_click();
        }

        let pet_bg = if pet_resp.hovered() {
            COLOR_GLASS_HI
        } else {
            COLOR_GLASS
        };

        painter.rect_filled(pet_rect, Rounding::same(10.0), pet_bg);
        painter.rect_stroke(pet_rect, Rounding::same(10.0), Stroke::new(1.0_f32, COLOR_LINE));

        let text_color = if self.pet.is_happy {
            COLOR_ACCENT
        } else {
            COLOR_INK
        };

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

        // Visualizer or timer on left
        let left_start = rect.left() + 10.0;
        if services.timer.is_running {
            // Display mini pomodoro timer countdown
            painter.text(
                pos2(left_start, rect.center().y),
                Align2::LEFT_CENTER,
                format!("⏱️ {}", services.timer.formatted_remaining()),
                FontId::monospace(11.5),
                COLOR_WARM,
            );
        } else {
            // Animated 3-bar music visualizer
            for i in 0..3 {
                let phase = self.anim_time * 6.0 + (i as f32) * 1.5;
                let bar_h = if is_playing {
                    6.0 + 8.0 * (phase.sin().abs())
                } else {
                    4.0
                };
                let bar_x = left_start + (i as f32) * 4.0;
                let bar_rect = Rect::from_center_size(pos2(bar_x, rect.center().y), vec2(2.5, bar_h));
                painter.rect_filled(bar_rect, Rounding::same(1.0), COLOR_ACCENT);
            }

            // Track title
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

        // Hardware indicator dots on right (Caps Lock / Mic / Vol)
        let right_center = pos2(rect.right() - 8.0, rect.center().y);
        let mut glyphs = String::new();
        if services.indicators.caps_lock {
            glyphs.push_str("🔠 ");
        }
        if services.indicators.mic_active {
            glyphs.push_str("🔴 ");
        }
        if services.controls.is_muted {
            glyphs.push_str("🔇");
        } else if is_playing {
            glyphs.push_str("🎵");
        } else {
            glyphs.push_str("⏸");
        }

        painter.text(
            right_center,
            Align2::RIGHT_CENTER,
            glyphs,
            FontId::proportional(11.0),
            COLOR_DIM,
        );
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

    fn render_expanded_view(&mut self, ui: &mut Ui, _rect: Rect, services: &mut ServiceManager) {
        ui.vertical(|ui| {
            // Header: App Logo, Tabs, and Collapse
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("⚡ NotchPixel")
                        .color(COLOR_ACCENT)
                        .size(13.0)
                        .strong(),
                );

                ui.add_space(4.0);

                // Tab Switcher
                let tabs = [
                    (CardTab::Media, "🎵"),
                    (CardTab::Timer, "⏱️"),
                    (CardTab::FileShelf, "📁"),
                    (CardTab::Clipboard, "📋"),
                    (CardTab::Notes, "📝"),
                    (CardTab::Telemetry, "⚡"),
                    (CardTab::Controls, "🎛️"),
                    (CardTab::Deck, "📱"),
                ];

                for (tab_kind, label) in tabs {
                    let is_active = self.tab == tab_kind;
                    let (bg, text_color) = if is_active {
                        (COLOR_GLASS_HI, COLOR_ACCENT)
                    } else {
                        (Color32::TRANSPARENT, COLOR_DIM)
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(label).color(text_color).size(12.0),
                    )
                    .fill(bg)
                    .rounding(Rounding::same(6.0));

                    if ui.add(btn).clicked() {
                        self.tab = tab_kind;
                    }
                }

                // Collapse Button
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("▲").color(COLOR_DIM).size(12.0),
                            )
                            .fill(COLOR_GLASS)
                            .rounding(Rounding::same(12.0)),
                        )
                        .on_hover_text("Collapse Notch")
                        .clicked()
                    {
                        self.set_mode(IslandMode::Compact);
                    }
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Active Tab View Content
            match self.tab {
                CardTab::Media => self.render_media_tab(ui, services),
                CardTab::Timer => self.render_timer_tab(ui, services),
                CardTab::FileShelf => self.render_file_shelf_tab(ui, services),
                CardTab::Clipboard => self.render_clipboard_tab(ui, services),
                CardTab::Notes => self.render_notes_tab(ui, services),
                CardTab::Telemetry => self.render_telemetry_tab(ui, services),
                CardTab::Controls => self.render_controls_tab(ui, services),
                CardTab::Deck => self.render_deck_tab(ui, services),
            }
        });
    }

    fn render_media_tab(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        let track = services.media.current_track().clone();
        let is_playing = services.media.status == PlaybackStatus::Playing;

        ui.horizontal(|ui| {
            // Vinyl Record Disc
            let disc_size = vec2(72.0, 72.0);
            let (disc_rect, _resp) = ui.allocate_exact_size(disc_size, Sense::hover());
            let painter = ui.painter_at(disc_rect);

            painter.circle_filled(disc_rect.center(), disc_size.x * 0.5, Color32::from_rgb(18, 18, 22));
            painter.circle_stroke(disc_rect.center(), disc_size.x * 0.5, Stroke::new(1.0_f32, COLOR_LINE));
            painter.circle_filled(disc_rect.center(), disc_size.x * 0.22, COLOR_ACCENT);
            painter.circle_filled(disc_rect.center(), disc_size.x * 0.06, Color32::BLACK);

            ui.add_space(10.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new(&track.title).color(COLOR_INK).size(14.0).strong());
                ui.label(egui::RichText::new(format!("{} • {}", track.artist, track.album)).color(COLOR_DIM).size(11.5));

                ui.add_space(4.0);

                // Scrubber slider
                let mut progress = track.progress_ratio();
                if ui.add(egui::Slider::new(&mut progress, 0.0..=1.0).show_value(false).trailing_fill(true)).changed() {
                    services.media.seek_ratio(progress);
                }

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(track.formatted_current_time()).color(COLOR_FAINT).size(10.5));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(track.formatted_duration()).color(COLOR_FAINT).size(10.5));
                    });
                });

                ui.add_space(2.0);

                // Controls row
                ui.horizontal(|ui| {
                    if ui.button("⏮").clicked() {
                        services.media.previous();
                    }
                    let play_label = if is_playing { "⏸" } else { "▶" };
                    if ui.add(egui::Button::new(egui::RichText::new(play_label).color(Color32::BLACK).strong()).fill(COLOR_ACCENT)).clicked() {
                        services.media.toggle_play_pause();
                    }
                    if ui.button("⏭").clicked() {
                        services.media.next();
                    }

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

    fn render_timer_tab(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.horizontal(|ui| {
            // Circular Ring Progress Graphic
            let ring_size = vec2(76.0, 76.0);
            let (ring_rect, _resp) = ui.allocate_exact_size(ring_size, Sense::hover());
            let painter = ui.painter_at(ring_rect);

            let center = ring_rect.center();
            let radius = ring_size.x * 0.44;
            painter.circle_stroke(center, radius, Stroke::new(4.0_f32, COLOR_LINE));

            let progress = services.timer.progress_ratio();
            let steps = 32;
            for i in 0..steps {
                let frac = i as f32 / steps as f32;
                if frac <= progress {
                    let a = -std::f32::consts::FRAC_PI_2 + frac * std::f32::consts::TAU;
                    let pt = center + vec2(a.cos(), a.sin()) * radius;
                    painter.circle_filled(pt, 2.5, COLOR_WARM);
                }
            }

            painter.text(
                center,
                Align2::CENTER_CENTER,
                services.timer.formatted_remaining(),
                FontId::monospace(14.0),
                COLOR_INK,
            );

            ui.add_space(12.0);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(services.timer.phase.label()).color(COLOR_WARM).strong());
                    ui.label(egui::RichText::new(format!("• 🍅 {} sessions", services.timer.completed_sessions)).color(COLOR_DIM).size(11.0));
                });

                ui.add_space(4.0);

                // Start/Pause & Reset buttons
                ui.horizontal(|ui| {
                    let start_label = if services.timer.is_running { "⏸ Pause" } else { "▶ Start" };
                    if ui.add(egui::Button::new(start_label).fill(COLOR_WARM)).clicked() {
                        services.timer.toggle_start_pause();
                    }
                    if ui.button("🔄 Reset").clicked() {
                        services.timer.reset();
                    }
                    if ui.button("+15m").clicked() {
                        services.timer.add_minutes(15.0);
                    }
                    if ui.button("+1h").clicked() {
                        services.timer.add_minutes(60.0);
                    }
                });

                ui.add_space(6.0);

                // Preset Phase Switches
                ui.horizontal(|ui| {
                    if ui.selectable_label(services.timer.phase == TimerPhase::Work, "25m Work").clicked() {
                        services.timer.set_phase(TimerPhase::Work);
                    }
                    if ui.selectable_label(services.timer.phase == TimerPhase::ShortBreak, "5m Break").clicked() {
                        services.timer.set_phase(TimerPhase::ShortBreak);
                    }
                    if ui.selectable_label(services.timer.phase == TimerPhase::LongBreak, "15m Long").clicked() {
                        services.timer.set_phase(TimerPhase::LongBreak);
                    }
                });
            });
        });
    }

    fn render_file_shelf_tab(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.horizontal(|ui| {
            // QR Code for Camera Pairing
            ui.vertical(|ui| {
                let qr_matrix = &services.file_shelf.qr_matrix;
                let qr_size = 90.0;
                let (qr_rect, _resp) = ui.allocate_exact_size(vec2(qr_size, qr_size), Sense::hover());
                let painter = ui.painter_at(qr_rect);
                painter.rect_filled(qr_rect, Rounding::same(6.0), Color32::WHITE);

                let n = qr_matrix.len();
                if n > 0 {
                    let cell = qr_size / n as f32;
                    for y in 0..n {
                        for x in 0..n {
                            if qr_matrix[y][x] {
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
                ui.label(egui::RichText::new("📱 Wi-Fi Download").color(COLOR_DIM).size(9.5));
            });

            ui.add_space(10.0);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Staged File Shelf").color(COLOR_INK).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑 Clear").clicked() {
                            services.file_shelf.clear();
                        }
                    });
                });

                let items = services.file_shelf.get_items();
                egui::ScrollArea::vertical().max_height(80.0).show(ui, |ui| {
                    for item in items {
                        ui.horizontal(|ui| {
                            ui.label("📄");
                            ui.label(egui::RichText::new(&item.name).color(COLOR_INK).size(11.0));
                            ui.label(egui::RichText::new(format!("({})", item.formatted_size())).color(COLOR_FAINT).size(10.0));
                        });
                    }
                });

                if ui.button("➕ Stage Screenshot").clicked() {
                    services.file_shelf.add_file("screenshot.png", vec![0x89, 0x50, 0x4E, 0x47]);
                }
            });
        });
    }

    fn render_clipboard_tab(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("📋 Clipboard History").color(COLOR_INK).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑 Clear Unpinned").clicked() {
                    services.clipboard.clear_unpinned();
                }
            });
        });

        ui.add_space(4.0);

        let snippets = services.clipboard.snippets.clone();
        egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
            if snippets.is_empty() {
                ui.label(egui::RichText::new("No copied items yet. Copy text anywhere in Windows.").color(COLOR_FAINT));
            }

            for snip in snippets {
                ui.horizontal(|ui| {
                    let pin_icon = if snip.is_pinned { "📌" } else { "📍" };
                    if ui.button(pin_icon).clicked() {
                        services.clipboard.toggle_pin(snip.id);
                    }

                    ui.label(egui::RichText::new(snip.preview(32)).color(COLOR_INK).size(11.5));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕").clicked() {
                            services.clipboard.remove(snip.id);
                        }
                        if ui.button("Copy").clicked() {
                            services.clipboard.copy_to_clipboard(snip.id);
                        }
                    });
                });
            }
        });
    }

    fn render_notes_tab(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("📝 Scratchpad").color(COLOR_INK).strong());
            ui.label(
                egui::RichText::new(format!(
                    "• {} chars, {} words",
                    services.notes.character_count(),
                    services.notes.word_count()
                ))
                .color(COLOR_FAINT)
                .size(10.5),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑 Clear").clicked() {
                    services.notes.clear();
                }
            });
        });

        ui.add_space(4.0);
        egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
            ui.text_edit_multiline(&mut services.notes.content);
        });
    }

    fn render_telemetry_tab(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.horizontal(|ui| {
            // FPS Hero Badge
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(format!("{:.0}", services.telemetry.fps)).color(COLOR_ACCENT).size(36.0).strong());
                ui.label(egui::RichText::new("HARDWARE FPS").color(COLOR_FAINT).size(9.0));
            });

            ui.add_space(16.0);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("CPU").color(COLOR_INK).size(11.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!("{:.1}%", services.telemetry.cpu_usage)).color(COLOR_ACCENT).strong());
                    });
                });
                ui.add(egui::ProgressBar::new(services.telemetry.cpu_usage / 100.0).desired_height(5.0));

                ui.add_space(2.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("RAM").color(COLOR_INK).size(11.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!("{:.1}G", services.telemetry.ram_used_gb)).color(COLOR_WARM));
                    });
                });
                ui.add(egui::ProgressBar::new(services.telemetry.ram_usage_pct / 100.0).desired_height(5.0));

                ui.add_space(4.0);

                // Power Plans
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Power:").color(COLOR_DIM).size(10.5));
                    if ui.selectable_label(self.power_plan == 0, "Balanced").clicked() {
                        self.power_plan = 0;
                    }
                    if ui.selectable_label(self.power_plan == 1, "⚡ High").clicked() {
                        self.power_plan = 1;
                    }
                    if ui.selectable_label(self.power_plan == 2, "🌱 Eco").clicked() {
                        self.power_plan = 2;
                    }
                });
            });
        });
    }

    fn render_controls_tab(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.vertical(|ui| {
            // Volume
            ui.horizontal(|ui| {
                let mute_icon = if services.controls.is_muted { "🔇" } else { "🔊" };
                if ui.button(mute_icon).clicked() {
                    services.controls.toggle_mute();
                }
                ui.label(egui::RichText::new("Volume").color(COLOR_INK).size(11.5));
                let mut vol = services.controls.volume;
                if ui.add(egui::Slider::new(&mut vol, 0.0..=1.0).show_value(false)).changed() {
                    services.controls.set_volume(vol);
                }
                ui.label(egui::RichText::new(format!("{}%", services.controls.volume_percent())).color(COLOR_DIM).size(10.5));
            });

            // Brightness
            ui.horizontal(|ui| {
                ui.label("🔆");
                ui.label(egui::RichText::new("Brightness").color(COLOR_INK).size(11.5));
                let mut b = services.controls.brightness;
                if ui.add(egui::Slider::new(&mut b, 0.0..=1.0).show_value(false)).changed() {
                    services.controls.set_brightness(b);
                }
                ui.label(egui::RichText::new(format!("{}%", services.controls.brightness_percent())).color(COLOR_DIM).size(10.5));
            });

            ui.add_space(4.0);

            // Microphone & CapsLock Indicators
            ui.horizontal(|ui| {
                let mic_label = if services.controls.is_mic_muted { "🔇 Mic Muted" } else { "🎙 Mic Active" };
                let mic_color = if services.controls.is_mic_muted { COLOR_HOT } else { COLOR_OK };
                if ui.add(egui::Button::new(egui::RichText::new(mic_label).color(mic_color)).fill(COLOR_GLASS)).clicked() {
                    services.controls.toggle_mic_mute();
                }

                let caps_label = if services.indicators.caps_lock { "🔠 CAPS ON" } else { "🔤 Caps Off" };
                ui.label(egui::RichText::new(caps_label).color(COLOR_DIM).size(11.0));
            });

            ui.add_space(4.0);

            // Notch geometry & pet toggles
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Style:").color(COLOR_DIM).size(11.0));
                if ui.selectable_label(self.notch_style == NotchStyle::FloatingIsland, "Island").clicked() {
                    self.notch_style = NotchStyle::FloatingIsland;
                }
                if ui.selectable_label(self.notch_style == NotchStyle::BezelNotch, "Bezel Notch").clicked() {
                    self.notch_style = NotchStyle::BezelNotch;
                }

                ui.add_space(10.0);
                ui.checkbox(&mut self.show_pet, "🐾 Desktop Pet");
                if self.show_pet {
                    if ui.selectable_label(self.pet.kind == PetKind::Cat, "🐱 Cat").clicked() {
                        self.pet.kind = PetKind::Cat;
                    }
                    if ui.selectable_label(self.pet.kind == PetKind::Bunny, "🐰 Bunny").clicked() {
                        self.pet.kind = PetKind::Bunny;
                    }
                }
            });
        });
    }

    fn render_deck_tab(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.horizontal(|ui| {
            // QR Code to pair mobile deck
            ui.vertical(|ui| {
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
                ui.label(egui::RichText::new("📱 Scan to Control").color(COLOR_DIM).size(9.5));
            });

            ui.add_space(12.0);

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("NotchPixel Mobile Deck").color(COLOR_INK).strong());
                ui.label(
                    egui::RichText::new(format!("Port {} • {}", services.deck_server.port, services.deck_server.server_url))
                        .color(COLOR_ACCENT)
                        .size(11.0),
                );

                ui.add_space(4.0);
                let connected = services
                    .deck_server
                    .connected_devices
                    .read()
                    .map(|c| *c > 0)
                    .unwrap_or(false);
                let status_text = if connected {
                    "🟢 Phone Controller Connected"
                } else {
                    "⚪ Waiting for Phone Connection"
                };
                ui.label(egui::RichText::new(status_text).color(COLOR_OK).size(11.5));

                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("Control media, sliders, haptics and macros directly from your smartphone browser.")
                        .color(COLOR_FAINT)
                        .size(10.5),
                );
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_initial_state() {
        let ui = IslandUi::new();
        assert_eq!(ui.mode, IslandMode::Compact);
        assert_eq!(ui.tab, CardTab::Media);
        assert_eq!(ui.notch_style, NotchStyle::FloatingIsland);
        assert!((ui.morph.width.target - 230.0).abs() < 1e-4);
        assert!((ui.morph.height.target - 38.0).abs() < 1e-4);
    }

    #[test]
    fn test_ui_mode_transitions() {
        let mut ui = IslandUi::new();
        ui.set_mode(IslandMode::Expanded);
        assert_eq!(ui.mode, IslandMode::Expanded);
        assert!((ui.morph.width.target - 480.0).abs() < 1e-4);
        assert!((ui.morph.height.target - 230.0).abs() < 1e-4);

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
