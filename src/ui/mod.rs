use egui::{
    pos2, vec2, Align2, Color32, FontId, Id, Painter, Rect, Rounding, Sense, Shape, Stroke, Ui,
};
use crate::physics::{generate_notch_path_points, generate_squircle_points, IslandMorphSpring};
use crate::services::{PlaybackStatus, ServiceManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IslandMode {
    Compact,
    Expanded,
    Slim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardTab {
    Media,
    FileShelf,
    Telemetry,
    Controls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotchStyle {
    FloatingIsland,
    BezelNotch,
}

pub struct IslandUi {
    pub mode: IslandMode,
    pub previous_mode: IslandMode,
    pub tab: CardTab,
    pub notch_style: NotchStyle,
    pub morph: IslandMorphSpring,
    pub is_hovered: bool,
    pub anim_time: f32,
    pub auto_expand_on_hover: bool,
}

impl Default for IslandUi {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandUi {
    pub fn new() -> Self {
        // Initial dimensions for compact pill: 220 x 36 with 18px corner radius
        Self {
            mode: IslandMode::Compact,
            previous_mode: IslandMode::Compact,
            tab: CardTab::Media,
            notch_style: NotchStyle::FloatingIsland,
            morph: IslandMorphSpring::new(220.0, 36.0, 18.0),
            is_hovered: false,
            anim_time: 0.0,
            auto_expand_on_hover: true,
        }
    }

    /// Sets the target island mode and configures spring target geometry
    pub fn set_mode(&mut self, mode: IslandMode) {
        if self.mode != mode {
            self.previous_mode = self.mode;
            self.mode = mode;
        }

        match self.mode {
            IslandMode::Compact => {
                self.morph.set_target_geometry(220.0, 36.0, 18.0);
            }
            IslandMode::Expanded => {
                self.morph.set_target_geometry(480.0, 230.0, 24.0);
            }
            IslandMode::Slim => {
                self.morph.set_target_geometry(380.0, 28.0, 14.0);
            }
        }
    }

    /// Advances spring physics by dt seconds
    pub fn update_physics(&mut self, dt: f32) {
        self.anim_time += dt;
        self.morph.update(dt);
    }

    /// Primary render method called inside egui central panel
    pub fn render(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        let dt = ui.input(|i| i.unstable_dt).clamp(0.001, 0.050);
        self.update_physics(dt);

        // Center horizontally at top of the window
        let screen_rect = ui.available_rect_before_wrap();
        let current_size = self.morph.current_size();
        let island_top = if self.notch_style == NotchStyle::BezelNotch {
            0.0
        } else {
            8.0
        };
        let island_center_x = screen_rect.center().x;
        let island_rect = Rect::from_center_size(
            pos2(island_center_x, island_top + current_size.y * 0.5),
            current_size,
        );

        // Interactivity bounding response
        let island_id = Id::new("notch_island_interaction");
        let response = ui.interact(island_rect, island_id, Sense::click());
        self.is_hovered = response.hovered();

        // Optional hover-to-expand behavior for compact pill
        if self.auto_expand_on_hover {
            if self.mode == IslandMode::Compact && response.hovered() && !response.clicked() {
                // Expand when hovered
                self.set_mode(IslandMode::Expanded);
            }
        }

        if response.clicked() {
            match self.mode {
                IslandMode::Compact => self.set_mode(IslandMode::Expanded),
                IslandMode::Slim => self.set_mode(IslandMode::Compact),
                IslandMode::Expanded => {}
            }
        }

        let painter = ui.painter_at(island_rect.expand(10.0));

        // 1. Paint background with squircle continuous curvature
        self.paint_background(&painter, island_rect);

        // 2. Paint active view content
        let content_rect = island_rect.shrink2(vec2(10.0, 6.0));

        // Only show content if width and height are sufficiently open
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

        // Request continuous repaint while springs are settling
        if !self.morph.is_settled() || services.media.status == PlaybackStatus::Playing {
            ui.ctx().request_repaint();
        }
    }

    /// Paints the squircle continuous curvature or top notch background
    fn paint_background(&self, painter: &Painter, rect: Rect) {
        let corner_radius = self.morph.corner_radius.current;
        let opacity = self.morph.opacity.current.clamp(0.0, 1.0);

        let bg_color = Color32::from_rgba_premultiplied(12, 14, 20, (246.0 * opacity) as u8);
        let border_color = Color32::from_rgba_premultiplied(255, 255, 255, (32.0 * opacity) as u8);
        let shadow_color = Color32::from_rgba_premultiplied(0, 0, 0, (90.0 * opacity) as u8);

        // Ambient outer shadow for floating look
        if self.notch_style == NotchStyle::FloatingIsland {
            let shadow_rect = rect.expand(3.0).translate(vec2(0.0, 2.0));
            let shadow_pts = generate_squircle_points(shadow_rect, corner_radius + 2.0, 10);
            painter.add(Shape::convex_polygon(shadow_pts, shadow_color, Stroke::NONE));
        }

        // Generate geometry points based on chosen style
        let points = if self.notch_style == NotchStyle::BezelNotch {
            generate_notch_path_points(rect, 14.0, corner_radius, 10)
        } else {
            generate_squircle_points(rect, corner_radius, 12)
        };

        // Render filled polygon and specular border stroke
        painter.add(Shape::convex_polygon(
            points.clone(),
            bg_color,
            Stroke::new(1.2_f32, border_color),
        ));
    }

    /// Renders the Compact Pill View (music indicator, title, animated waveform)
    fn paint_compact_view(&self, painter: &Painter, rect: Rect, services: &ServiceManager) {
        let track = services.media.current_track();
        let is_playing = services.media.status == PlaybackStatus::Playing;

        // Left: Animated visualizer bars
        let left_center = pos2(rect.left() + 8.0, rect.center().y);
        let bar_accent = Color32::from_rgb(
            track.accent_color[0],
            track.accent_color[1],
            track.accent_color[2],
        );

        for i in 0..3 {
            let phase = self.anim_time * 6.0 + (i as f32) * 1.5;
            let bar_h = if is_playing {
                6.0 + 8.0 * (phase.sin().abs())
            } else {
                4.0
            };
            let bar_x = left_center.x + (i as f32) * 4.0;
            let bar_rect = Rect::from_center_size(pos2(bar_x, left_center.y), vec2(2.5, bar_h));
            painter.rect_filled(bar_rect, Rounding::same(1.0), bar_accent);
        }

        // Center: Track title
        let text_rect = Rect::from_min_max(
            pos2(rect.left() + 26.0, rect.top()),
            pos2(rect.right() - 28.0, rect.bottom()),
        );
        let title_snippet = if track.title.len() > 18 {
            format!("{}...", &track.title[..16])
        } else {
            track.title.clone()
        };

        painter.text(
            pos2(text_rect.left(), text_rect.center().y),
            Align2::LEFT_CENTER,
            title_snippet,
            FontId::proportional(12.5),
            Color32::from_rgb(240, 240, 245),
        );

        // Right: mini audio icon or glyph
        let right_center = pos2(rect.right() - 8.0, rect.center().y);
        let glyph = if services.controls.is_muted {
            "🔇"
        } else if is_playing {
            "🎵"
        } else {
            "⏸"
        };
        painter.text(
            right_center,
            Align2::RIGHT_CENTER,
            glyph,
            FontId::proportional(12.0),
            Color32::from_rgb(180, 185, 200),
        );
    }

    /// Renders the Slim Gaming HUD View (FPS, CPU, RAM strip)
    fn paint_slim_view(&self, painter: &Painter, rect: Rect, services: &ServiceManager) {
        let fps_text = format!("{:.0} FPS", services.telemetry.fps);
        let cpu_text = format!("CPU {:.0}%", services.telemetry.cpu_usage);
        let ram_text = format!("RAM {:.1}G", services.telemetry.ram_used_gb);

        let items = [
            ("⚡", fps_text, Color32::from_rgb(60, 240, 140)), // Neon green FPS
            ("💻", cpu_text, Color32::from_rgb(120, 200, 255)), // Cyan CPU
            ("🧠", ram_text, Color32::from_rgb(255, 180, 80)),  // Amber RAM
        ];

        let spacing = rect.width() / 3.0;
        for (i, (icon, text, color)) in items.iter().enumerate() {
            let cx = rect.left() + (i as f32 + 0.5) * spacing;
            let display_str = format!("{} {}", icon, text);
            painter.text(
                pos2(cx, rect.center().y),
                Align2::CENTER_CENTER,
                display_str,
                FontId::proportional(11.5),
                *color,
            );
        }
    }

    /// Renders the full interactive Expanded Card with multi-tab panels
    fn render_expanded_view(&mut self, ui: &mut Ui, _rect: Rect, services: &mut ServiceManager) {
        ui.vertical(|ui| {
            // --- Header Row: Branding, Tabs, and Collapse Button ---
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("⚡ NotchIsland")
                        .color(Color32::from_rgb(220, 225, 240))
                        .size(13.0)
                        .strong(),
                );

                ui.add_space(8.0);

                // Tab Switcher Pills
                let tabs = [
                    (CardTab::Media, "🎵 Media"),
                    (CardTab::FileShelf, "📁 Shelf"),
                    (CardTab::Telemetry, "📊 Stats"),
                    (CardTab::Controls, "⚙ Controls"),
                ];

                for (tab_kind, label) in tabs {
                    let is_active = self.tab == tab_kind;
                    let text_color = if is_active {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(150, 155, 170)
                    };
                    let bg = if is_active {
                        Color32::from_rgb(45, 50, 70)
                    } else {
                        Color32::TRANSPARENT
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(label).color(text_color).size(11.0),
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
                                egui::RichText::new("▲")
                                    .color(Color32::from_rgb(180, 185, 200))
                                    .size(12.0),
                            )
                            .fill(Color32::from_rgba_premultiplied(255, 255, 255, 15))
                            .rounding(Rounding::same(12.0)),
                        )
                        .on_hover_text("Collapse to compact pill")
                        .clicked()
                    {
                        self.set_mode(IslandMode::Compact);
                    }
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // --- Body: Render Selected Tab Content ---
            match self.tab {
                CardTab::Media => self.render_media_tab(ui, services),
                CardTab::FileShelf => self.render_file_shelf_tab(ui, services),
                CardTab::Telemetry => self.render_telemetry_tab(ui, services),
                CardTab::Controls => self.render_controls_tab(ui, services),
            }
        });
    }

    /// Tab 1: Media Player with album disc, scrubber, track metadata, and controls
    fn render_media_tab(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        let track = services.media.current_track().clone();
        let is_playing = services.media.status == PlaybackStatus::Playing;

        ui.horizontal(|ui| {
            // Album Artwork Vinyl Disc
            let disc_size = vec2(68.0, 68.0);
            let (disc_rect, _disc_resp) = ui.allocate_exact_size(disc_size, Sense::hover());
            let painter = ui.painter_at(disc_rect);

            let disc_center = disc_rect.center();
            let disc_radius = disc_size.x * 0.5;

            // Outer Vinyl Ring
            painter.circle_filled(
                disc_center,
                disc_radius,
                Color32::from_rgb(20, 22, 28),
            );
            painter.circle_stroke(
                disc_center,
                disc_radius,
                Stroke::new(1.0_f32, Color32::from_rgb(55, 60, 75)),
            );

            // Colored center label
            let accent_col = Color32::from_rgb(
                track.accent_color[0],
                track.accent_color[1],
                track.accent_color[2],
            );
            painter.circle_filled(disc_center, disc_radius * 0.45, accent_col);
            painter.circle_filled(
                disc_center,
                disc_radius * 0.12,
                Color32::from_rgb(12, 14, 18),
            );

            ui.add_space(8.0);

            // Metadata and playback controls
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(&track.title)
                        .color(Color32::WHITE)
                        .size(14.0)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("{} • {}", track.artist, track.album))
                        .color(Color32::from_rgb(160, 165, 185))
                        .size(11.5),
                );

                ui.add_space(4.0);

                // Scrubber Slider
                let mut progress = track.progress_ratio();
                let scrubber = egui::Slider::new(&mut progress, 0.0..=1.0)
                    .show_value(false)
                    .trailing_fill(true);
                if ui.add(scrubber).changed() {
                    services.media.seek_ratio(progress);
                }

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(track.formatted_current_time())
                            .color(Color32::from_rgb(140, 145, 160))
                            .size(10.5),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(track.formatted_duration())
                                .color(Color32::from_rgb(140, 145, 160))
                                .size(10.5),
                        );
                    });
                });

                ui.add_space(2.0);

                // Playback Control Buttons
                ui.horizontal(|ui| {
                    if ui.button("⏮ Prev").clicked() {
                        services.media.previous();
                    }

                    let play_pause_label = if is_playing { "⏸ Pause" } else { "▶ Play" };
                    let play_btn = egui::Button::new(
                        egui::RichText::new(play_pause_label).color(Color32::WHITE).strong(),
                    )
                    .fill(Color32::from_rgb(50, 90, 200));

                    if ui.add(play_btn).clicked() {
                        services.media.toggle_play_pause();
                    }

                    if ui.button("⏭ Next").clicked() {
                        services.media.next();
                    }

                    ui.add_space(10.0);
                    // Inline volume preview
                    ui.label(egui::RichText::new("🔊").size(12.0));
                    let mut vol = services.controls.volume;
                    if ui.add(egui::Slider::new(&mut vol, 0.0..=1.0).show_value(false)).changed() {
                        services.controls.set_volume(vol);
                    }
                });
            });
        });
    }

    /// Tab 2: File Shelf & Mobile QR Code Wi-Fi Sharing
    fn render_file_shelf_tab(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.horizontal(|ui| {
            // Left: QR Code Renderer (drawn using rectangles)
            ui.vertical(|ui| {
                let qr_matrix = &services.file_shelf.qr_matrix;
                let qr_size = 110.0;
                let (qr_rect, _qr_resp) = ui.allocate_exact_size(vec2(qr_size, qr_size), Sense::hover());
                let painter = ui.painter_at(qr_rect);

                // White backdrop for camera contrast
                painter.rect_filled(qr_rect, Rounding::same(6.0), Color32::WHITE);

                let n = qr_matrix.len();
                if n > 0 {
                    let cell_w = qr_size / n as f32;
                    for y in 0..n {
                        for x in 0..n {
                            if qr_matrix[y][x] {
                                let c_rect = Rect::from_min_size(
                                    pos2(qr_rect.left() + x as f32 * cell_w, qr_rect.top() + y as f32 * cell_w),
                                    vec2(cell_w + 0.3, cell_w + 0.3),
                                );
                                painter.rect_filled(c_rect, Rounding::ZERO, Color32::BLACK);
                            }
                        }
                    }
                }

                ui.label(
                    egui::RichText::new("📱 Scan to Download")
                        .color(Color32::from_rgb(180, 185, 200))
                        .size(10.5),
                );
            });

            ui.add_space(12.0);

            // Right: Staged files list and actions
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Staged File Shelf")
                            .color(Color32::WHITE)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑 Clear").clicked() {
                            services.file_shelf.clear();
                        }
                    });
                });

                let items = services.file_shelf.get_items();
                ui.label(
                    egui::RichText::new(format!("{} item(s) on Wi-Fi shelf • {}", items.len(), services.file_shelf.server_url))
                        .color(Color32::from_rgb(140, 145, 160))
                        .size(11.0),
                );

                ui.add_space(4.0);

                egui::ScrollArea::vertical().max_height(80.0).show(ui, |ui| {
                    for item in items {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("📄").size(11.0));
                            ui.label(
                                egui::RichText::new(&item.name)
                                    .color(Color32::from_rgb(220, 225, 240))
                                    .size(11.5),
                            );
                            ui.label(
                                egui::RichText::new(format!("({})", item.formatted_size()))
                                    .color(Color32::from_rgb(130, 135, 150))
                                    .size(10.5),
                            );
                        });
                    }
                });

                ui.add_space(4.0);
                if ui.button("➕ Stage Sample Screenshot").clicked() {
                    services.file_shelf.add_file(
                        "screenshot_notch.png",
                        vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
                    );
                }
            });
        });
    }

    /// Tab 3: Hardware Telemetry, CPU/RAM bars & Gaming Mode
    fn render_telemetry_tab(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.horizontal(|ui| {
            // FPS Counter Hero Badge
            let fps_color = if services.telemetry.fps >= 100.0 {
                Color32::from_rgb(60, 240, 140)
            } else if services.telemetry.fps >= 50.0 {
                Color32::from_rgb(120, 210, 255)
            } else {
                Color32::from_rgb(255, 120, 80)
            };

            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(format!("{:.0}", services.telemetry.fps))
                        .color(fps_color)
                        .size(34.0)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new("HARDWARE FPS")
                        .color(Color32::from_rgb(140, 145, 160))
                        .size(9.5),
                );
            });

            ui.add_space(20.0);

            // Gauges for CPU and RAM
            ui.vertical(|ui| {
                // CPU Progress
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("CPU Usage").color(Color32::WHITE).size(11.5));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{:.1}%", services.telemetry.cpu_usage))
                                .color(Color32::from_rgb(120, 200, 255))
                                .strong(),
                        );
                    });
                });
                let cpu_ratio = (services.telemetry.cpu_usage / 100.0).clamp(0.0, 1.0);
                ui.add(egui::ProgressBar::new(cpu_ratio).desired_height(6.0));

                ui.add_space(4.0);

                // RAM Progress
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("RAM Usage").color(Color32::WHITE).size(11.5));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{:.1} GB / {:.1} GB",
                                services.telemetry.ram_used_gb, services.telemetry.ram_total_gb
                            ))
                            .color(Color32::from_rgb(255, 180, 80)),
                        );
                    });
                });
                let ram_ratio = (services.telemetry.ram_usage_pct / 100.0).clamp(0.0, 1.0);
                ui.add(egui::ProgressBar::new(ram_ratio).desired_height(6.0));

                ui.add_space(6.0);

                // Switch to Slim Gaming HUD Button
                ui.horizontal(|ui| {
                    if ui
                        .button(egui::RichText::new("🎮 Switch to Slim Gaming HUD").size(11.0))
                        .on_hover_text("Shrinks to minimal top strip for borderless gaming")
                        .clicked()
                    {
                        self.set_mode(IslandMode::Slim);
                    }
                });
            });
        });
    }

    /// Tab 4: System Volume, Brightness, Microphone Mute & Island Appearance
    fn render_controls_tab(&mut self, ui: &mut Ui, services: &mut ServiceManager) {
        ui.vertical(|ui| {
            // Master Volume
            ui.horizontal(|ui| {
                let mute_icon = if services.controls.is_muted { "🔇" } else { "🔊" };
                if ui.button(mute_icon).clicked() {
                    services.controls.toggle_mute();
                }
                ui.label(egui::RichText::new("Master Volume").color(Color32::WHITE).size(12.0));
                let mut vol = services.controls.volume;
                if ui.add(egui::Slider::new(&mut vol, 0.0..=1.0).show_value(false)).changed() {
                    services.controls.set_volume(vol);
                }
                ui.label(
                    egui::RichText::new(format!("{}%", services.controls.volume_percent()))
                        .color(Color32::from_rgb(180, 185, 200)),
                );
            });

            ui.add_space(4.0);

            // Display Brightness
            ui.horizontal(|ui| {
                ui.label("🔆");
                ui.label(egui::RichText::new("Brightness").color(Color32::WHITE).size(12.0));
                let mut bright = services.controls.brightness;
                if ui.add(egui::Slider::new(&mut bright, 0.0..=1.0).show_value(false)).changed() {
                    services.controls.set_brightness(bright);
                }
                ui.label(
                    egui::RichText::new(format!("{}%", services.controls.brightness_percent()))
                        .color(Color32::from_rgb(180, 185, 200)),
                );
            });

            ui.add_space(4.0);

            // Microphone Mute
            ui.horizontal(|ui| {
                let (mic_label, mic_col) = if services.controls.is_mic_muted {
                    ("🔇 Microphone Muted (Ctrl+Alt+M)", Color32::from_rgb(255, 100, 100))
                } else {
                    ("🎙 Microphone Active", Color32::from_rgb(100, 230, 140))
                };

                let mic_btn = egui::Button::new(
                    egui::RichText::new(mic_label).color(mic_col).strong(),
                )
                .fill(Color32::from_rgba_premultiplied(255, 255, 255, 12));

                if ui.add(mic_btn).clicked() {
                    services.controls.toggle_mic_mute();
                }
            });

            ui.add_space(6.0);

            // Notch Shape Style
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Notch Geometry:").color(Color32::from_rgb(180, 185, 200)).size(11.5));
                if ui.selectable_label(self.notch_style == NotchStyle::FloatingIsland, "Floating Island").clicked() {
                    self.notch_style = NotchStyle::FloatingIsland;
                }
                if ui.selectable_label(self.notch_style == NotchStyle::BezelNotch, "Bezel Notch").clicked() {
                    self.notch_style = NotchStyle::BezelNotch;
                }
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
        assert!((ui.morph.width.target - 220.0).abs() < 1e-4);
        assert!((ui.morph.height.target - 36.0).abs() < 1e-4);
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
}
