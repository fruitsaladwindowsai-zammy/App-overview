// Savanna User Interface - macOS Launchpad Aesthetic for Windows
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Instant;

use egui::{
    pos2, vec2, Align2, Color32, ColorImage, FontId, Key, Rect, Rounding, Stroke,
    TextureHandle, TextureOptions,
};

use crate::app_scanner::scan_installed_apps;
use crate::config::Config;
use crate::hotkey::GestureCommand;
use crate::icon_extractor::{get_cached_or_extract, RawIcon};
use crate::launcher::launch_app;
use crate::model::{
    move_selection, page_count, per_page, score_entry, AppEntry, MoveDirection,
};
use crate::tray::TrayCommand;
use crate::wallpaper::{spawn_wallpaper_loader, BlurredWallpaper};

struct LoadedIcon {
    id: String,
    icon: RawIcon,
}

pub struct SavannaApp {
    pub config: Config,
    pub all_apps: Vec<AppEntry>,
    pub filtered_apps: Vec<AppEntry>,
    pub search_query: String,
    pub current_page: usize,
    pub selected_index: usize,

    // Textures & Wallpaper
    textures: HashMap<String, TextureHandle>,
    icon_receiver: Receiver<LoadedIcon>,
    wallpaper_texture: Option<TextureHandle>,
    wallpaper_receiver: Receiver<BlurredWallpaper>,

    // Spring & Bounce Animations
    anim_progress: f32,
    anim_velocity: f32,
    is_animating_out: bool,
    start_time: Instant,
    last_cursor_blink: Instant,
    cursor_visible: bool,

    // Gestures & Trackpad continuous page scrolling
    page_offset: f32,
    page_velocity: f32,
    target_page: usize,
    is_page_dragging: bool,
    vertical_drag_accum: f32,

    // Tray & Hotkey channels
    pub is_visible: bool,
    pub should_close: bool,
    tray_receiver: Receiver<TrayCommand>,
    hotkey_receiver: Receiver<GestureCommand>,
}

impl SavannaApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        config: Config,
        tray_rx: Receiver<TrayCommand>,
        hotkey_rx: Receiver<GestureCommand>,
    ) -> Self {
        let (icon_tx, icon_rx) = channel();
        let (wp_tx, wp_rx) = channel();

        // 1. Initial app scan
        let apps = scan_installed_apps();
        let filtered = apps.clone();

        // 2. Spawn background icon loader thread
        let apps_clone = apps.clone();
        let icon_size = config.icon_size as u32;
        thread::spawn(move || {
            unsafe {
                let _ = windows::Win32::System::Com::CoInitializeEx(
                    None,
                    windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
                );
            }
            for app in apps_clone {
                if let Some(raw) = get_cached_or_extract(&app.target, icon_size) {
                    let _ = icon_tx.send(LoadedIcon {
                        id: app.id.clone(),
                        icon: raw,
                    });
                }
            }
            unsafe {
                windows::Win32::System::Com::CoUninitialize();
            }
        });

        // 3. Spawn background wallpaper loader thread
        spawn_wallpaper_loader(move |blurred| {
            let _ = wp_tx.send(blurred);
        });

        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        Self {
            config,
            all_apps: apps,
            filtered_apps: filtered,
            search_query: String::new(),
            current_page: 0,
            selected_index: 0,
            textures: HashMap::new(),
            icon_receiver: icon_rx,
            wallpaper_texture: None,
            wallpaper_receiver: wp_rx,
            anim_progress: 0.0,
            anim_velocity: 0.0,
            is_animating_out: false,
            start_time: Instant::now(),
            last_cursor_blink: Instant::now(),
            cursor_visible: true,
            page_offset: 0.0,
            page_velocity: 0.0,
            target_page: 0,
            is_page_dragging: false,
            vertical_drag_accum: 0.0,
            is_visible: true,
            should_close: false,
            tray_receiver: tray_rx,
            hotkey_receiver: hotkey_rx,
        }
    }

    pub fn filter_apps(&mut self) {
        let q = self.search_query.trim();
        if q.is_empty() {
            self.filtered_apps = self.all_apps.clone();
        } else {
            let mut scored: Vec<(i32, AppEntry)> = self
                .all_apps
                .iter()
                .filter_map(|app| {
                    let s = score_entry(app, q);
                    if s >= 0 {
                        Some((s, app.clone()))
                    } else {
                        None
                    }
                })
                .collect();

            scored.sort_by(|a, b| {
                if a.0 != b.0 {
                    b.0.cmp(&a.0)
                } else {
                    a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase())
                }
            });

            self.filtered_apps = scored.into_iter().map(|(_, app)| app).collect();
        }

        self.current_page = 0;
        self.target_page = 0;
        self.page_offset = 0.0;
        self.page_velocity = 0.0;
        self.selected_index = 0;
    }

    fn accent_color(&self) -> Color32 {
        parse_hex_color(&self.config.accent_color).unwrap_or(Color32::from_rgb(137, 180, 250))
    }

    fn handle_external_messages(&mut self, ctx: &egui::Context) {
        // Hotkey & Precision Touchpad Gestures
        while let Ok(cmd) = self.hotkey_receiver.try_recv() {
            match cmd {
                GestureCommand::Toggle => {
                    self.toggle_visibility(ctx);
                }
                GestureCommand::Dismiss => {
                    if self.is_visible && !self.is_animating_out {
                        self.initiate_dismiss();
                    }
                }
            }
        }

        // Tray commands
        while let Ok(cmd) = self.tray_receiver.try_recv() {
            match cmd {
                TrayCommand::Toggle => {
                    self.toggle_visibility(ctx);
                }
                TrayCommand::Reload => {
                    self.all_apps = scan_installed_apps();
                    self.filter_apps();
                }
                TrayCommand::Exit => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    fn toggle_visibility(&mut self, ctx: &egui::Context) {
        if self.is_visible && !self.is_animating_out {
            self.initiate_dismiss();
        } else {
            self.is_visible = true;
            self.is_animating_out = false;
            crate::hotkey::IS_APP_VISIBLE.store(true, std::sync::atomic::Ordering::Relaxed);
            self.anim_progress = 0.0;
            self.anim_velocity = 0.0;
            self.search_query.clear();
            self.filter_apps();
            self.start_time = Instant::now();
            self.should_close = false;
            self.page_offset = 0.0;
            self.page_velocity = 0.0;
            self.target_page = 0;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
    }

    fn initiate_dismiss(&mut self) {
        if self.is_visible && !self.is_animating_out {
            self.is_animating_out = true;
            self.anim_velocity = 0.0;
        }
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        let total = self.filtered_apps.len();
        let pp = per_page(self.config.columns, self.config.rows);
        let pages = page_count(total, self.config.columns, self.config.rows);

        // Blinking cursor timer
        if self.last_cursor_blink.elapsed().as_millis() > 530 {
            self.cursor_visible = !self.cursor_visible;
            self.last_cursor_blink = Instant::now();
        }

        // Process egui input events
        ctx.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Text(text) => {
                        for c in text.chars() {
                            if !c.is_control() {
                                self.search_query.push(c);
                                self.filter_apps();
                            }
                        }
                    }
                    egui::Event::Key {
                        key,
                        pressed: true,
                        ..
                    } => match key {
                        Key::Backspace => {
                            if !self.search_query.is_empty() {
                                self.search_query.pop();
                                self.filter_apps();
                            }
                        }
                        Key::Escape => {
                            if !self.search_query.is_empty() {
                                self.search_query.clear();
                                self.filter_apps();
                            } else {
                                self.initiate_dismiss();
                            }
                        }
                        Key::Enter => {
                            if !self.filtered_apps.is_empty() {
                                let idx = self.selected_index.min(self.filtered_apps.len() - 1);
                                let app = &self.filtered_apps[idx];
                                launch_app(&app.target);
                                self.initiate_dismiss();
                            }
                        }
                        Key::ArrowRight => {
                            self.selected_index = move_selection(
                                self.selected_index,
                                MoveDirection::Right,
                                total,
                                self.config.columns,
                                self.config.rows,
                            );
                            self.target_page = self.selected_index / pp.max(1);
                        }
                        Key::ArrowLeft => {
                            self.selected_index = move_selection(
                                self.selected_index,
                                MoveDirection::Left,
                                total,
                                self.config.columns,
                                self.config.rows,
                            );
                            self.target_page = self.selected_index / pp.max(1);
                        }
                        Key::ArrowDown => {
                            self.selected_index = move_selection(
                                self.selected_index,
                                MoveDirection::Down,
                                total,
                                self.config.columns,
                                self.config.rows,
                            );
                            self.target_page = self.selected_index / pp.max(1);
                        }
                        Key::ArrowUp => {
                            self.selected_index = move_selection(
                                self.selected_index,
                                MoveDirection::Up,
                                total,
                                self.config.columns,
                                self.config.rows,
                            );
                            self.target_page = self.selected_index / pp.max(1);
                        }
                        Key::PageDown => {
                            if self.target_page < pages.saturating_sub(1) {
                                self.target_page += 1;
                                self.selected_index = (self.target_page * pp).min(total.saturating_sub(1));
                            }
                        }
                        Key::PageUp => {
                            if self.target_page > 0 {
                                self.target_page -= 1;
                                self.selected_index = (self.target_page * pp).min(total.saturating_sub(1));
                            }
                        }
                        _ => {}
                    },
                    egui::Event::MouseWheel { delta, .. } => {
                        // Two-finger horizontal trackpad swipe or mouse wheel
                        if delta.x.abs() > 0.4 {
                            self.page_offset -= delta.x * 0.012;
                            self.target_page = self.page_offset.round().clamp(0.0, (pages.saturating_sub(1)) as f32) as usize;
                        } else if delta.y > 18.0 {
                            // Upward vertical scroll
                            if self.target_page > 0 {
                                self.target_page -= 1;
                                self.selected_index = (self.target_page * pp).min(total.saturating_sub(1));
                            }
                        } else if delta.y < -18.0 {
                            // Downward vertical scroll -> if at bottom or gesture, dismiss
                            if self.target_page < pages.saturating_sub(1) {
                                self.target_page += 1;
                                self.selected_index = (self.target_page * pp).min(total.saturating_sub(1));
                            } else {
                                self.initiate_dismiss();
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Trackpad smooth continuous horizontal scrolling
            let smooth = i.smooth_scroll_delta;
            if smooth.x.abs() > 0.6 {
                self.page_offset -= smooth.x * 0.004;
                self.target_page = self.page_offset.round().clamp(0.0, (pages.saturating_sub(1)) as f32) as usize;
            }

            // Trackpad / Mouse Drag Swipe gesture (multi-touch or drag all around)
            if i.pointer.primary_down() {
                let p_delta = i.pointer.delta();

                // Vertical swipe down to dismiss
                self.vertical_drag_accum += p_delta.y;
                if self.vertical_drag_accum > 55.0 {
                    self.initiate_dismiss();
                    self.vertical_drag_accum = 0.0;
                }

                // Horizontal drag swipe across pages
                if p_delta.x.abs() > 0.4 {
                    self.is_page_dragging = true;
                    let screen_w = ctx.screen_rect().width().max(400.0);
                    self.page_offset -= p_delta.x / (screen_w * 0.85);
                    let dt = i.stable_dt.clamp(0.001, 0.05);
                    self.page_velocity = -p_delta.x / dt / (screen_w * 0.85);
                }
            } else {
                self.vertical_drag_accum = 0.0;
                if self.is_page_dragging {
                    self.is_page_dragging = false;
                    // Fling snapping with velocity
                    if self.page_velocity > 1.2 {
                        self.target_page = (self.target_page + 1).min(pages.saturating_sub(1));
                    } else if self.page_velocity < -1.2 {
                        self.target_page = self.target_page.saturating_sub(1);
                    } else {
                        self.target_page = self.page_offset.round().clamp(0.0, (pages.saturating_sub(1)) as f32) as usize;
                    }
                    self.selected_index = (self.target_page * pp).min(total.saturating_sub(1));
                }
            }
        });
    }

    fn receive_textures(&mut self, ctx: &egui::Context) {
        // Load incoming app icons
        while let Ok(loaded) = self.icon_receiver.try_recv() {
            let color_image = ColorImage::from_rgba_unmultiplied(
                [loaded.icon.width as usize, loaded.icon.height as usize],
                &loaded.icon.rgba,
            );
            let handle = ctx.load_texture(
                format!("icon_{}", loaded.id),
                color_image,
                TextureOptions::LINEAR,
            );
            self.textures.insert(loaded.id, handle);
        }

        // Load blurred wallpaper if newly ready
        while let Ok(wp) = self.wallpaper_receiver.try_recv() {
            let color_image = ColorImage::from_rgba_unmultiplied(
                [wp.width as usize, wp.height as usize],
                &wp.rgba,
            );
            let handle = ctx.load_texture("wallpaper_blur", color_image, TextureOptions::LINEAR);
            self.wallpaper_texture = Some(handle);
        }
    }
}

impl eframe::App for SavannaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_external_messages(ctx);

        if !self.is_visible {
            // Keep app alive in background, low frequency repaint
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
            return;
        }

        self.receive_textures(ctx);
        self.handle_input(ctx);

        // Continuous repaint during animation or dragging
        ctx.request_repaint();

        let dt = ctx.input(|i| i.stable_dt).clamp(0.001, 0.05);

        // 1. Spring & Bounce entrance and dismissal physics
        let anim_target = if self.is_animating_out { 0.0 } else { 1.0 };
        let disp = self.anim_progress - anim_target;
        // Underdamped harmonic spring for that signature macOS/iOS bounce
        let stiffness = 220.0;
        let damping = 17.5;
        let force = -stiffness * disp - damping * self.anim_velocity;
        self.anim_velocity += force * dt;
        self.anim_progress += self.anim_velocity * dt;

        // Check if dismissal animation finished
        if self.is_animating_out && self.anim_progress <= 0.04 && self.anim_velocity.abs() < 0.09 {
            self.is_visible = false;
            self.is_animating_out = false;
            crate::hotkey::IS_APP_VISIBLE.store(false, std::sync::atomic::Ordering::Relaxed);
            self.anim_progress = 0.0;
            self.anim_velocity = 0.0;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            return;
        }

        let current_alpha = self.anim_progress.clamp(0.0, 1.0);
        let current_scale = (0.91 + 0.09 * self.anim_progress).clamp(0.85, 1.06);

        // 2. Spring physics for page sliding
        let total = self.filtered_apps.len();
        let cols = self.config.columns.max(3);
        let rows = self.config.rows.max(2);
        let pp = per_page(cols, rows);
        let pages = page_count(total, cols, rows);

        if !self.is_page_dragging {
            let target_f = self.target_page as f32;
            let p_disp = self.page_offset - target_f;
            let p_stiffness = 180.0;
            let p_damping = 19.0;
            let p_force = -p_stiffness * p_disp - p_damping * self.page_velocity;
            self.page_velocity += p_force * dt;
            self.page_offset += self.page_velocity * dt;
        }
        self.current_page = self.target_page.min(pages.saturating_sub(1));

        let screen_rect = ctx.screen_rect();
        let painter = ctx.layer_painter(egui::LayerId::background());

        let is_centered = self.config.centered || self.config.backdrop.eq_ignore_ascii_case("Centered");
        let accent = self.accent_color();

        let surface_rect = if is_centered {
            let card_w = screen_rect.width() * self.config.card_width.clamp(0.4, 0.95);
            let card_h = screen_rect.height() * self.config.card_height.clamp(0.4, 0.95);
            let card_x = screen_rect.center().x - card_w / 2.0;
            let card_y = screen_rect.center().y - card_h / 2.0;
            Rect::from_min_size(pos2(card_x, card_y), vec2(card_w, card_h))
        } else {
            screen_rect
        };

        // 3. Draw Blurred Wallpaper & Frosted Backdrop
        if let Some(ref wp_tex) = self.wallpaper_texture {
            painter.image(
                wp_tex.id(),
                screen_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::from_rgba_unmultiplied(255, 255, 255, (255.0 * current_alpha) as u8),
            );
        } else {
            // Dark gradient backdrop fallback
            painter.rect_filled(
                screen_rect,
                Rounding::ZERO,
                Color32::from_rgba_unmultiplied(16, 20, 32, (240.0 * current_alpha) as u8),
            );
        }

        // Frosted Darkening Overlay
        if is_centered {
            painter.rect_filled(
                screen_rect,
                Rounding::ZERO,
                Color32::from_rgba_unmultiplied(0, 0, 0, (140.0 * current_alpha) as u8),
            );

            painter.rect(
                surface_rect,
                Rounding::same(24.0),
                Color32::from_rgba_unmultiplied(18, 24, 38, (240.0 * current_alpha) as u8),
                Stroke::new(1.8_f32, Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), (120.0 * current_alpha) as u8)),
            );
        } else {
            painter.rect_filled(
                surface_rect,
                Rounding::ZERO,
                Color32::from_rgba_unmultiplied(12, 16, 26, (140.0 * current_alpha) as u8),
            );

            let top_gradient = Rect::from_min_size(surface_rect.min, vec2(surface_rect.width(), 160.0));
            painter.rect_filled(
                top_gradient,
                Rounding::ZERO,
                Color32::from_rgba_unmultiplied(16, 22, 38, (90.0 * current_alpha) as u8),
            );
        }

        let bg_clicked = ctx.input(|i| i.pointer.primary_clicked());
        let mut clicked_any_tile = false;

        // 4. Centered Translucent Search Bar Pill
        let search_w: f32 = 480.0_f32.min(surface_rect.width() * 0.55);
        let search_h: f32 = 46.0;
        let search_x = surface_rect.center().x - search_w / 2.0;
        let search_y = surface_rect.top() + (if is_centered { 30.0 } else { 48.0 });
        let search_rect = Rect::from_min_size(pos2(search_x, search_y), vec2(search_w, search_h));

        let is_searching = !self.search_query.is_empty();

        if is_searching {
            let glow_rect = search_rect.expand(3.0);
            painter.rect_stroke(
                glow_rect,
                Rounding::same(search_h / 2.0),
                Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), (70.0 * current_alpha) as u8)),
            );
        }

        let pill_bg = if is_searching {
            Color32::from_rgba_unmultiplied(20, 26, 44, (230.0 * current_alpha) as u8)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, (22.0 * current_alpha) as u8)
        };
        let pill_border = if is_searching {
            Stroke::new(1.8_f32, Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), (255.0 * current_alpha) as u8))
        } else {
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, (45.0 * current_alpha) as u8))
        };

        painter.rect(
            search_rect,
            Rounding::same(search_h / 2.0),
            pill_bg,
            pill_border,
        );

        // Search Icon
        painter.text(
            pos2(search_rect.min.x + 22.0, search_rect.center().y),
            Align2::LEFT_CENTER,
            "🔍",
            FontId::proportional(15.0),
            if is_searching { accent } else { Color32::from_rgba_unmultiplied(255, 255, 255, (160.0 * current_alpha) as u8) },
        );

        // Search Text or Placeholder
        let text_x = search_rect.min.x + 48.0;
        if self.search_query.is_empty() {
            painter.text(
                pos2(text_x, search_rect.center().y),
                Align2::LEFT_CENTER,
                "Search applications...",
                FontId::proportional(15.0),
                Color32::from_rgba_unmultiplied(255, 255, 255, (110.0 * current_alpha) as u8),
            );
        } else {
            let galley = painter.layout_no_wrap(
                self.search_query.clone(),
                FontId::proportional(15.0),
                Color32::WHITE,
            );
            let text_pos = pos2(text_x, search_rect.center().y - galley.size().y / 2.0);
            painter.galley(text_pos, galley.clone(), Color32::WHITE);

            // Blinking cursor
            if self.cursor_visible {
                let cursor_x = text_x + galley.size().x + 2.0;
                let cursor_top = search_rect.center().y - 10.0;
                let cursor_bottom = search_rect.center().y + 10.0;
                painter.line_segment(
                    [pos2(cursor_x, cursor_top), pos2(cursor_x, cursor_bottom)],
                    Stroke::new(2.0_f32, accent),
                );
            }
        }

        // App count badge
        let count_str = if is_searching {
            format!("{} found", self.filtered_apps.len())
        } else {
            format!("{} apps", self.all_apps.len())
        };
        painter.text(
            pos2(search_rect.max.x - 18.0, search_rect.center().y),
            Align2::RIGHT_CENTER,
            count_str,
            FontId::proportional(13.0),
            Color32::from_rgba_unmultiplied(255, 255, 255, (130.0 * current_alpha) as u8),
        );

        // 5. Grid of Applications - Continuous Multi-Page Sliding
        let grid_top = search_y + search_h + 38.0;
        let grid_bottom = surface_rect.bottom() - (if is_centered { 55.0 } else { 85.0 });
        let grid_h = (grid_bottom - grid_top).max(200.0);
        let grid_w = (surface_rect.width() * 0.92).min(1800.0);
        let grid_left = surface_rect.center().x - grid_w / 2.0;

        let cell_w = grid_w / cols as f32;
        let cell_h = grid_h / rows as f32;

        let page_stride_x = grid_w + 120.0;
        let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
        let primary_clicked = ctx.input(|i| i.pointer.primary_clicked());

        // Render each page with horizontal offset for continuous swipe
        for page_idx in 0..pages {
            let page_shift_x = (page_idx as f32 - self.page_offset) * page_stride_x;

            // Only render pages that are visible on screen
            if page_shift_x.abs() < screen_rect.width() + 100.0 {
                let p_start = page_idx * pp;
                let p_end = (p_start + pp).min(total);
                let page_fade = (1.0 - (page_shift_x.abs() / (screen_rect.width() * 0.85))).clamp(0.0, 1.0);
                let page_alpha_u8 = (255.0 * current_alpha * page_fade) as u8;

                for i in p_start..p_end {
                    let local_idx = i - p_start;
                    let row = local_idx / cols;
                    let col = local_idx % cols;

                    let cell_x = grid_left + col as f32 * cell_w + page_shift_x;
                    let cell_y = grid_top + row as f32 * cell_h;
                    let cell_rect = Rect::from_min_size(pos2(cell_x, cell_y), vec2(cell_w, cell_h));

                    let app = &self.filtered_apps[i];
                    let is_selected = self.selected_index == i;
                    let is_hovered = pointer_pos.map_or(false, |p| cell_rect.contains(p));

                    if is_hovered && page_shift_x.abs() < 60.0 {
                        self.selected_index = i;
                    }

                    // Click to launch
                    if is_hovered && primary_clicked && page_shift_x.abs() < 60.0 {
                        clicked_any_tile = true;
                        launch_app(&app.target);
                        self.initiate_dismiss();
                        break;
                    }

                    // Icon scale & bounce
                    let icon_base_size = self.config.icon_size.clamp(64.0, 96.0);
                    let is_active = (is_hovered || is_selected) && page_shift_x.abs() < 60.0;
                    let icon_scale = if is_active { 1.10 } else { 1.0 };
                    let render_icon_size = icon_base_size * icon_scale * current_scale;

                    let icon_center = pos2(cell_rect.center().x, cell_rect.min.y + cell_h * 0.42);
                    let icon_rect = Rect::from_center_size(icon_center, vec2(render_icon_size, render_icon_size));

                    // BORDERLESS SELECTION: Soft ambient glass wash strictly behind the icon (NO BORDER)
                    if is_active {
                        let halo_size = render_icon_size + 14.0;
                        let halo_rect = Rect::from_center_size(icon_center, vec2(halo_size, halo_size));

                        let halo_alpha = (if is_selected { 36.0 } else { 22.0 } * page_fade) as u8;
                        painter.rect_filled(
                            halo_rect,
                            Rounding::same(halo_size * 0.32),
                            Color32::from_rgba_unmultiplied(255, 255, 255, halo_alpha),
                        );
                    }

                    // Draw High-Res Icon
                    if let Some(texture) = self.textures.get(&app.id) {
                        painter.image(
                            texture.id(),
                            icon_rect,
                            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                            Color32::from_rgba_unmultiplied(255, 255, 255, page_alpha_u8),
                        );
                    } else {
                        // Procedural icon with app initial
                        painter.rect_filled(
                            icon_rect,
                            Rounding::same(16.0),
                            Color32::from_rgba_unmultiplied(36, 46, 72, page_alpha_u8),
                        );
                        let initial = app.name.chars().next().unwrap_or('A').to_uppercase().to_string();
                        painter.text(
                            icon_rect.center(),
                            Align2::CENTER_CENTER,
                            initial,
                            FontId::proportional(render_icon_size * 0.45),
                            Color32::from_rgba_unmultiplied(255, 255, 255, page_alpha_u8),
                        );
                    }

                    // Draw App Label
                    if self.config.show_labels {
                        let label_y = icon_center.y + render_icon_size / 2.0 + 16.0;
                        let label_text = truncate_string(&app.name, 18);

                        let galley = painter.layout_no_wrap(
                            label_text.clone(),
                            FontId::proportional(13.5),
                            Color32::WHITE,
                        );

                        if is_selected && page_shift_x.abs() < 60.0 {
                            let pill_w = galley.size().x + 18.0;
                            let pill_h = galley.size().y + 6.0;
                            let pill_rect = Rect::from_center_size(pos2(cell_rect.center().x, label_y), vec2(pill_w, pill_h));
                            painter.rect_filled(
                                pill_rect,
                                Rounding::same(pill_h / 2.0),
                                Color32::from_rgba_unmultiplied(255, 255, 255, (30.0 * page_fade) as u8),
                            );
                        } else {
                            // Text drop shadow for contrast against wallpaper
                            painter.text(
                                pos2(cell_rect.center().x, label_y + 1.2),
                                Align2::CENTER_CENTER,
                                &label_text,
                                FontId::proportional(13.5),
                                Color32::from_rgba_unmultiplied(0, 0, 0, (210.0 * page_fade) as u8),
                            );
                        }

                        // Main text
                        painter.text(
                            pos2(cell_rect.center().x, label_y),
                            Align2::CENTER_CENTER,
                            &label_text,
                            FontId::proportional(13.5),
                            Color32::from_rgba_unmultiplied(240, 244, 255, page_alpha_u8),
                        );
                    }
                }
            }
        }

        // 6. Animated Continuous Pagination Indicator (macOS / Savanna style)
        if pages > 1 {
            let dot_y = surface_rect.bottom() - 36.0;
            let mut total_w = 0.0;
            for _ in 0..pages {
                total_w += 8.0 + 8.0;
            }
            total_w += 18.0 - 8.0; // active pill expansion width

            let start_x = surface_rect.center().x - total_w / 2.0;
            let mut cur_x = start_x;

            for page in 0..pages {
                let dist_to_page = (page as f32 - self.page_offset).abs();
                let active_weight = (1.0 - dist_to_page).clamp(0.0, 1.0);
                let dot_w = 8.0 + 18.0 * active_weight;
                let dot_h = 8.0;
                let dot_rect = Rect::from_center_size(pos2(cur_x + dot_w / 2.0, dot_y), vec2(dot_w, dot_h));

                // Click dot to jump directly to page
                if primary_clicked && pointer_pos.map_or(false, |p| dot_rect.expand(6.0).contains(p)) {
                    self.target_page = page;
                    self.selected_index = (page * pp).min(total.saturating_sub(1));
                }

                let r = (255.0 * (1.0 - active_weight) + accent.r() as f32 * active_weight) as u8;
                let g = (255.0 * (1.0 - active_weight) + accent.g() as f32 * active_weight) as u8;
                let b = (255.0 * (1.0 - active_weight) + accent.b() as f32 * active_weight) as u8;
                let a = (70.0 + 185.0 * active_weight) as u8;

                painter.rect_filled(
                    dot_rect,
                    Rounding::same(4.0),
                    Color32::from_rgba_unmultiplied(r, g, b, (a as f32 * current_alpha) as u8),
                );

                cur_x += dot_w + 8.0;
            }
        }

        // 7. Dismiss if clicked empty blurred backdrop outside tiles and search bar
        if bg_clicked && !clicked_any_tile && self.anim_progress > 0.8 && !self.is_page_dragging {
            if let Some(pos) = pointer_pos {
                if !search_rect.contains(pos) {
                    self.initiate_dismiss();
                }
            }
        }
    }
}

fn truncate_string(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars.saturating_sub(2)).collect();
        format!("{}…", truncated)
    }
}

fn parse_hex_color(hex: &str) -> Option<Color32> {
    let clean = hex.trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some(Color32::from_rgb(r, g, b))
    } else if clean.len() == 8 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        let a = u8::from_str_radix(&clean[6..8], 16).ok()?;
        Some(Color32::from_rgba_unmultiplied(r, g, b, a))
    } else {
        None
    }
}
