use crate::{
    audio::AudioPlayer,
    keymap::{modifier_spec, spec_for_key, spec_for_shifted_key, spec_for_text, KeySpec, Modifier},
    settings::{system_prefers_dark, Settings, ThemeMode},
    syskeys::{self, SysKeys},
};
use eframe::egui::{
    self, Align2, Color32, Event, FontId, Key, Layout, Pos2, RichText, Sense, Stroke, StrokeKind,
    Vec2, ViewportCommand,
};
use std::time::{Duration, Instant};

pub struct KeyboardApp {
    settings: Settings,
    audio: AudioPlayer,
    syskeys: SysKeys,
    current: Option<KeySpec>,
    pulse_until: Instant,
    last_modifiers: egui::Modifiers,
    system_dark: bool,
    next_theme_check: Instant,
    status: Option<(String, Instant)>,
}

impl KeyboardApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (settings, settings_warning) = Settings::load();
        let (syskeys, syskeys_warning) = syskeys::start(cc.egui_ctx.clone());
        let warning = settings_warning.or(syskeys_warning);
        Self {
            settings,
            audio: AudioPlayer::new(),
            syskeys,
            current: None,
            pulse_until: Instant::now(),
            last_modifiers: egui::Modifiers::default(),
            system_dark: system_prefers_dark(),
            next_theme_check: Instant::now() + Duration::from_secs(2),
            status: warning.map(|message| (message, Instant::now() + Duration::from_secs(8))),
        }
    }

    fn effective_dark(&self) -> bool {
        match self.settings.theme {
            ThemeMode::Auto => self.system_dark,
            ThemeMode::Light => false,
            ThemeMode::Dark => true,
        }
    }

    fn set_theme(&mut self, theme: ThemeMode) {
        self.settings.theme = theme;
        if let Err(error) = self.settings.save() {
            self.status = Some((
                format!("Could not save settings: {error}"),
                Instant::now() + Duration::from_secs(4),
            ));
        }
    }

    fn cycle_theme(&mut self) {
        let next = match self.settings.theme {
            ThemeMode::Auto => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Auto,
        };
        self.set_theme(next);
    }

    fn process_input(&mut self, ctx: &egui::Context) {
        let (keys, text_keys, modifiers, should_quit, cycle_theme) = ctx.input(|input| {
            let mut keys = Vec::new();
            let mut text_keys = Vec::new();
            let mut physical_keys = Vec::new();
            let mut should_quit = false;
            let mut cycle_theme = false;
            for event in &input.events {
                match event {
                    Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        modifiers: event_modifiers,
                        ..
                    } => {
                        if *key == Key::Q && input.modifiers.ctrl {
                            should_quit = true;
                        } else if *key == Key::T && input.modifiers.ctrl {
                            cycle_theme = true;
                        } else {
                            physical_keys.push((*key, *event_modifiers));
                        }
                    }
                    Event::Text(text) => {
                        if let Some(spec) = spec_for_text(text) {
                            text_keys.push(spec);
                        }
                    }
                    _ => {}
                }
            }
            if text_keys.is_empty() {
                for (key, event_modifiers) in physical_keys {
                    if event_modifiers.shift {
                        if let Some(spec) = spec_for_shifted_key(key) {
                            text_keys.push(spec);
                            continue;
                        }
                    }
                    if let Some(spec) = spec_for_key(key) {
                        keys.push(spec);
                    }
                }
            }
            (keys, text_keys, input.modifiers, should_quit, cycle_theme)
        });

        if should_quit {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }

        if cycle_theme {
            self.cycle_theme();
            return;
        }

        // Keys egui never reports (PrtSc/ScrLk/Pause/Caps/Num/Win/Fn) arrive
        // from the evdev reader on Linux. evdev is global, so these register
        // even when the window is unfocused — which is exactly when PrtSc's
        // desktop screenshot binding fires. The reader requests a repaint for
        // each event, so this drain runs immediately.
        while let Some(spec) = self.syskeys.try_recv() {
            self.show_key(spec);
        }

        // Text events contain the final character after keyboard layout and
        // modifiers have been applied. Prefer them for printable keys so
        // Shift+Comma is displayed and spoken as `<`, not as Shift then Comma.
        if !text_keys.is_empty() {
            self.last_modifiers = modifiers;
            for spec in text_keys {
                self.show_key(spec);
            }
            return;
        }

        let modifier_changes = [
            (
                modifiers.shift && !self.last_modifiers.shift,
                Modifier::Shift,
            ),
            (
                modifiers.ctrl && !self.last_modifiers.ctrl,
                Modifier::Control,
            ),
            (modifiers.alt && !self.last_modifiers.alt, Modifier::Alt),
            (
                modifiers.mac_cmd && !self.last_modifiers.mac_cmd,
                Modifier::Meta,
            ),
        ];
        self.last_modifiers = modifiers;
        for (pressed, modifier) in modifier_changes {
            if pressed {
                self.show_key(modifier_spec(modifier));
            }
        }
        for spec in keys {
            self.show_key(spec);
        }
    }

    fn show_key(&mut self, spec: KeySpec) {
        self.current = Some(spec);
        self.pulse_until = Instant::now() + Duration::from_millis(180);
        self.audio.speak(spec);
    }

    fn theme_button(&mut self, ui: &mut egui::Ui, theme: ThemeMode, tooltip: &str) {
        let selected = self.settings.theme == theme;
        // Sense::CLICK instead of Sense::click(): the theme buttons stay
        // clickable but are excluded from the Tab focus order, so Tab+Enter
        // keeps speaking keys instead of toggling the theme.
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(42.0), Sense::CLICK);
        let center = rect.center();
        let hovered = response.hovered();
        let dark = self.effective_dark();
        let button_fill = if selected {
            if dark {
                Color32::from_rgb(52, 91, 145)
            } else {
                Color32::from_rgb(215, 229, 250)
            }
        } else if hovered {
            if dark {
                Color32::from_rgb(43, 47, 55)
            } else {
                Color32::from_rgb(231, 234, 239)
            }
        } else {
            Color32::TRANSPARENT
        };
        let icon_color = if dark {
            Color32::from_rgb(229, 234, 242)
        } else {
            Color32::from_rgb(38, 44, 52)
        };
        let border_color = if selected {
            if dark {
                Color32::from_rgb(132, 179, 239)
            } else {
                Color32::from_rgb(57, 111, 190)
            }
        } else if dark {
            Color32::from_rgb(91, 98, 109)
        } else {
            Color32::from_rgb(178, 184, 193)
        };
        let painter = ui.painter();
        painter.circle_filled(center, 20.0, button_fill);
        painter.circle_stroke(center, 19.5, Stroke::new(1.2_f32, border_color));
        self.paint_theme_icon(painter, center, theme, icon_color, button_fill, dark);
        if response.on_hover_text(tooltip).clicked() {
            self.set_theme(theme);
        }
    }

    fn paint_theme_icon(
        &self,
        painter: &egui::Painter,
        center: Pos2,
        theme: ThemeMode,
        icon_color: Color32,
        button_fill: Color32,
        dark: bool,
    ) {
        match theme {
            ThemeMode::Auto => {
                painter.text(
                    center,
                    Align2::CENTER_CENTER,
                    "A",
                    FontId::proportional(16.0),
                    icon_color,
                );
            }
            ThemeMode::Light => {
                painter.circle_filled(center, 5.2, icon_color);
                for index in 0..8 {
                    let angle = std::f32::consts::TAU * index as f32 / 8.0;
                    let direction = Vec2::new(angle.cos(), angle.sin());
                    painter.line_segment(
                        [center + direction * 9.0, center + direction * 12.0],
                        Stroke::new(1.6_f32, icon_color),
                    );
                }
            }
            ThemeMode::Dark => {
                painter.circle_filled(center - Vec2::new(1.5, 0.0), 7.0, icon_color);
                let cutout = if button_fill == Color32::TRANSPARENT {
                    if dark {
                        Color32::from_rgb(18, 20, 24)
                    } else {
                        Color32::from_rgb(247, 248, 250)
                    }
                } else {
                    button_fill
                };
                painter.circle_filled(center + Vec2::new(2.5, -3.0), 7.0, cutout);
            }
        }
    }

    fn paint_keycap(&self, ui: &mut egui::Ui, id: &str, label: &str, dark: bool, active: bool) {
        let dimensions = Self::keycap_dimensions(ui.available_width(), id);
        let width = dimensions.x;
        let height = dimensions.y;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let painter = ui.painter();
        let shadow = if dark {
            Color32::from_rgba_unmultiplied(0, 0, 0, 120)
        } else {
            Color32::from_rgba_unmultiplied(44, 52, 64, 55)
        };
        let edge = if dark {
            Color32::from_rgb(13, 15, 18)
        } else {
            Color32::from_rgb(169, 175, 183)
        };
        let body = if dark {
            Color32::from_rgb(38, 42, 48)
        } else {
            Color32::from_rgb(211, 214, 219)
        };
        let top = if active {
            if dark {
                Color32::from_rgb(57, 107, 175)
            } else {
                Color32::from_rgb(224, 235, 252)
            }
        } else if dark {
            Color32::from_rgb(61, 66, 74)
        } else {
            Color32::from_rgb(244, 245, 247)
        };
        let text = if dark {
            Color32::from_rgb(247, 249, 252)
        } else {
            Color32::from_rgb(26, 31, 38)
        };
        painter.rect_filled(rect.translate(Vec2::new(0.0, 7.0)), 15.0, shadow);
        let radius = if id == "enter" { 4.0 } else { 15.0 };
        painter.rect_filled(rect, radius, body);
        let top_rect = egui::Rect::from_min_max(
            rect.min + Vec2::new(3.0, 2.0),
            rect.max - Vec2::new(3.0, 11.0),
        );
        painter.rect_filled(top_rect, if id == "enter" { 3.0 } else { 12.0 }, top);
        painter.rect_stroke(
            rect,
            radius,
            Stroke::new(1.0_f32, edge),
            StrokeKind::Outside,
        );
        painter.line_segment(
            [
                top_rect.left_top() + Vec2::new(12.0, 1.0),
                top_rect.right_top() - Vec2::new(12.0, 1.0),
            ],
            Stroke::new(
                1.0_f32,
                if dark {
                    Color32::from_rgb(92, 99, 110)
                } else {
                    Color32::WHITE
                },
            ),
        );
        let icon_direction = match id {
            "arrow_up" => Some(Vec2::new(0.0, -1.0)),
            "arrow_down" => Some(Vec2::new(0.0, 1.0)),
            "arrow_left" => Some(Vec2::new(-1.0, 0.0)),
            "arrow_right" => Some(Vec2::new(1.0, 0.0)),
            _ => None,
        };
        if let Some(direction) = icon_direction {
            let arrow_length = (height * 0.32).clamp(42.0, 72.0);
            let tail = top_rect.center() - direction * (arrow_length * 0.48);
            painter.arrow(
                tail,
                direction * arrow_length,
                Stroke::new((height * 0.035).clamp(3.0, 5.0), text),
            );
        } else {
            painter.text(
                top_rect.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional((height * 0.30).clamp(34.0, 72.0)),
                text,
            );
        }
    }

    fn keycap_dimensions(available_width: f32, id: &str) -> Vec2 {
        let base_width = (available_width * 0.38)
            .clamp(300.0, 456.0)
            .min(available_width);
        let max_width = (available_width * 0.88).max(base_width);
        let width = (base_width * Self::key_width_units(id)).min(max_width);
        Vec2::new(width, (base_width * 0.52).clamp(130.0, 236.0))
    }

    fn key_width_units(id: &str) -> f32 {
        match id {
            "space" => 6.25,
            "shift" => 2.25,
            "enter" => 1.65,
            "backspace" => 2.0,
            "tab" => 1.5,
            "control" | "alt" | "meta" => 1.25,
            _ => 1.0,
        }
    }
}

impl eframe::App for KeyboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        if now >= self.next_theme_check {
            self.system_dark = system_prefers_dark();
            self.next_theme_check = now + Duration::from_secs(2);
        }

        self.process_input(ctx);
        let now = Instant::now();
        let pulse_active = now < self.pulse_until;
        let dark = self.effective_dark();
        ctx.set_visuals(if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });
        let (background, foreground) = if dark {
            (
                Color32::from_rgb(18, 20, 24),
                Color32::from_rgb(242, 244, 247),
            )
        } else {
            (
                Color32::from_rgb(247, 248, 250),
                Color32::from_rgb(24, 28, 34),
            )
        };

        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::NONE
                    .fill(background)
                    .stroke(Stroke::NONE)
                    .inner_margin(egui::Margin::symmetric(18, 10)),
            )
            .show(ctx, |ui| {
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    self.theme_button(ui, ThemeMode::Dark, "Dark theme");
                    ui.add_space(7.0);
                    self.theme_button(ui, ThemeMode::Light, "Light theme");
                    ui.add_space(7.0);
                    self.theme_button(ui, ThemeMode::Auto, "Auto theme");
                });
            });

        let status_active = self
            .status
            .as_ref()
            .is_some_and(|(_, expires)| Instant::now() < *expires);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(background))
            .show(ctx, |ui| {
                ui.visuals_mut().override_text_color = Some(foreground);
                ui.vertical_centered(|ui| {
                    let (id, label, description) = self
                        .current
                        .map(|spec| (spec.id, spec.label, spec.speech))
                        .unwrap_or(("", "Press any key", "Press any key"));
                    let available_height = ui.available_height();
                    let keycap_height = Self::keycap_dimensions(ui.available_width(), id).y;
                    let size =
                        (ui.available_width().min(available_height) * 0.18).clamp(42.0, 180.0);
                    let status_height = if status_active { 44.0 } else { 0.0 };
                    let content_height = keycap_height + 22.0 + size + status_height;
                    ui.add_space(((available_height - content_height) * 0.5).max(24.0));
                    self.paint_keycap(ui, id, label, dark, pulse_active);
                    ui.add_space(22.0);
                    let color = if pulse_active {
                        if dark {
                            Color32::from_rgb(142, 190, 255)
                        } else {
                            Color32::from_rgb(35, 92, 180)
                        }
                    } else {
                        foreground
                    };
                    ui.label(RichText::new(description).size(size).strong().color(color));
                    if status_active {
                        if let Some((message, _)) = &self.status {
                            ui.add_space(24.0);
                            ui.label(
                                RichText::new(message)
                                    .size(14.0)
                                    .color(Color32::from_rgb(190, 90, 70)),
                            );
                        }
                    }
                });
            });

        if pulse_active || self.settings.theme == ThemeMode::Auto {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
}
