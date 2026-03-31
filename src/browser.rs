use egui::{
    Align, Color32, FontFamily, FontId, FontTweak, Key, Layout, RichText, ScrollArea, Sense,
    Stroke, Ui, Vec2,
};

use crate::catalog::FontCategory;
use crate::state::FontBookState;

const UI_FONT_SIZE: f32 = 13.0;
const UI_FONT_FAMILY: &str = "FontBookUI";
const TWEAK_FONT_SIZE: f32 = 10.0;

fn selection_bg(ui: &Ui) -> Color32 {
    let bg = ui.visuals().panel_fill;
    let [r, g, b, a] = bg.to_array();
    Color32::from_rgba_premultiplied(
        r.saturating_add(10),
        g.saturating_add(10),
        b.saturating_add(10),
        a,
    )
}

/// Dimmer selection for the inactive list.
fn selection_bg_dim(ui: &Ui) -> Color32 {
    let bg = ui.visuals().panel_fill;
    let [r, g, b, a] = bg.to_array();
    Color32::from_rgba_premultiplied(
        r.saturating_add(5),
        g.saturating_add(5),
        b.saturating_add(5),
        a,
    )
}

fn ui_font() -> FontId {
    FontId::new(UI_FONT_SIZE, FontFamily::Name(UI_FONT_FAMILY.into()))
}

fn ui_text(text: &str) -> RichText {
    RichText::new(text).font(ui_font())
}

fn ui_text_small(text: &str) -> RichText {
    RichText::new(text).font(FontId::new(
        TWEAK_FONT_SIZE,
        FontFamily::Name(UI_FONT_FAMILY.into()),
    ))
}

pub struct FontBrowser<'a> {
    state: &'a mut FontBookState,
}

impl<'a> FontBrowser<'a> {
    pub fn new(state: &'a mut FontBookState) -> Self {
        Self { state }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();

        if !self.state.initialized {
            self.state.initialized = true;
            // Pick up the host app's default font size from the current Body text style.
            let host_size = ctx.global_style().text_styles
                .get(&egui::TextStyle::Body)
                .map(|id| id.size)
                .unwrap_or(self.state.global_font_size);
            if self.state.global_font_size == 14.0 {
                // Only override if still at the library default, not a restored value.
                self.state.global_font_size = host_size;
            }
            self.state.load_cache();
            self.state.auto_select_cached();
            self.state.apply_fonts(&ctx);
            self.state.fonts_dirty = true;
            ctx.request_repaint();
            return;
        }

        self.state.poll_downloads();

        if self.state.fonts_dirty {
            self.state.apply_fonts(&ctx);
            ctx.request_repaint();
        }

        // Clear selection_changed from last frame
        self.state.selection_changed = None;

        // ── Global keyboard navigation ──
        if !ctx.egui_wants_keyboard_input() {
            let cat = self.state.active_category;
            let count = self.state.indices_for(cat).len();
            let sel = self.state.selected_for(cat);
            let mut new_sel = sel;

            if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
                new_sel = (sel + 1).min(count.saturating_sub(1));
            }
            if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
                new_sel = sel.saturating_sub(1);
            }
            if ui.input(|i| i.key_pressed(Key::Tab)) {
                self.state.active_category = match cat {
                    FontCategory::Proportional => FontCategory::Monospace,
                    FontCategory::Monospace => FontCategory::Proportional,
                };
            }

            if new_sel != sel {
                *self.state.selected_for_mut(cat) = new_sel;
                self.state.selection_changed = Some(cat);
                let indices = self.state.indices_for(cat);
                if let Some(&cat_idx) = indices.get(new_sel) {
                    let name = self.state.catalog[cat_idx].name.to_string();
                    self.state.request_download(&name, &ctx);
                    self.state.apply_selected_for(cat);
                }
            }

            if ui.input(|i| i.key_pressed(Key::Enter)) {
                let cat = self.state.active_category;
                let sel = self.state.selected_for(cat);
                let indices = self.state.indices_for(cat);
                if let Some(&cat_idx) = indices.get(sel) {
                    let name = self.state.catalog[cat_idx].name.to_string();
                    self.state.request_download(&name, &ctx);
                    self.state.apply_selected_for(cat);
                }
            }
        }

        // ── Top bar: presets + global font size + load all ──
        ui.horizontal(|ui| {
            // Preset buttons (left side)
            for slot in 0..9usize {
                let label = format!("{}", slot + 1);
                let has_preset = self.state.presets[slot].is_some();
                let btn = if has_preset {
                    egui::Button::new(ui_text_small(&label))
                } else {
                    egui::Button::new(
                        RichText::new(&label)
                            .font(FontId::new(
                                TWEAK_FONT_SIZE,
                                FontFamily::Name(UI_FONT_FAMILY.into()),
                            ))
                            .weak(),
                    )
                };
                let response = ui.add(btn);
                if response.clicked() {
                    self.state.load_preset(slot);
                }
                if response.secondary_clicked() {
                    self.state.save_preset(slot);
                }
                if has_preset {
                    response.on_hover_text_at_pointer(format!(
                        "Click to load preset {}. Right-click to overwrite.",
                        slot + 1
                    ));
                } else {
                    response.on_hover_text_at_pointer(format!(
                        "Empty. Right-click to save current config as preset {}.",
                        slot + 1
                    ));
                }
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Load All button
                let all_downloaded = self.state.catalog.iter().all(|e| {
                    self.state.is_downloaded(&e.name) || self.state.is_downloading(&e.name)
                });
                if !all_downloaded {
                    if ui.small_button("Load all").clicked() {
                        self.state.request_download_all(&ctx);
                    }
                }

                // Global font size
                if ui
                    .add(
                        egui::DragValue::new(&mut self.state.global_font_size)
                            .range(8.0..=48.0)
                            .speed(0.025),
                    )
                    .changed()
                {
                    self.state.fonts_dirty = true;
                }
                ui.label(ui_text_small("size"));
            });
        });

        // Settings (collapsible)
        egui::CollapsingHeader::new(ui_text_small("sample text"))
            .id_salt("settings_text")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(ui_text_small("1"));
                    ui.text_edit_singleline(&mut self.state.sample_text_1);
                });
                ui.horizontal(|ui| {
                    ui.label(ui_text_small("2"));
                    ui.text_edit_singleline(&mut self.state.sample_text_2);
                });
            });

        ui.separator();

        // Split remaining height 50/50
        let avail = ui.available_height();
        let half = (avail * 0.5).max(100.0);

        ui.allocate_ui(Vec2::new(ui.available_width(), half), |ui| {
            self.show_font_section(ui, FontCategory::Proportional);
        });

        ui.separator();

        ui.allocate_ui(
            Vec2::new(ui.available_width(), ui.available_height()),
            |ui| {
                self.show_font_section(ui, FontCategory::Monospace);
            },
        );

        if self.state.fonts_dirty {
            self.state.apply_fonts(&ctx);
            ctx.request_repaint();
        }
    }

    fn show_tweaks_for(&mut self, ui: &mut Ui, font_name: &str) {
        let ctx = ui.ctx().clone();
        let tweak = self.state.tweak_for(font_name);
        let mut scale = tweak.scale;
        let mut y_offset = tweak.y_offset;
        let mut y_offset_factor = tweak.y_offset_factor;
        let mut size_off = self.state.size_offset_for(font_name);
        let mut changed = false;

        let tweak_font = FontId::new(TWEAK_FONT_SIZE, FontFamily::Name(UI_FONT_FAMILY.into()));

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.style_mut().override_font_id = Some(tweak_font.clone());

            ui.label(RichText::new("px").weak());
            changed |= ui
                .add(
                    egui::DragValue::new(&mut size_off)
                        .range(-10.0..=20.0)
                        .speed(0.05)
                        .custom_formatter(|v, _| {
                            if v >= 0.0 { format!("+{v:.0}") } else { format!("{v:.0}") }
                        }),
                )
                .changed();

            ui.add_space(4.0);
            ui.label(RichText::new("s").weak());
            changed |= ui
                .add(
                    egui::DragValue::new(&mut scale)
                        .range(0.5..=2.0)
                        .speed(0.001)
                        .custom_formatter(|v, _| format!("{v:.2}")),
                )
                .changed();

            ui.add_space(2.0);
            ui.label(RichText::new("y").weak());
            changed |= ui
                .add(
                    egui::DragValue::new(&mut y_offset)
                        .range(-10.0..=10.0)
                        .speed(0.01)
                        .custom_formatter(|v, _| format!("{v:.1}")),
                )
                .changed();

            ui.add_space(2.0);
            ui.label(RichText::new("yf").weak());
            changed |= ui
                .add(
                    egui::DragValue::new(&mut y_offset_factor)
                        .range(-1.0..=1.0)
                        .speed(0.001)
                        .custom_formatter(|v, _| format!("{v:.2}")),
                )
                .changed();
        });

        // Variant buttons + reset
        // Collect variant info before mutably borrowing state
        let entry = self.state.catalog.iter().find(|e| &*e.name == font_name);
        let variants: Vec<(usize, String, String)> = entry
            .map(|e| {
                e.variants
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (i, v.label.to_string(), v.cache_key.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let active_var = self.state.active_variant(font_name);

        if variants.len() > 1 {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (vi, vlabel, vcache_key) in &variants {
                    let is_active = *vi == active_var;
                    let is_cached = self.state.is_variant_downloaded(vcache_key);
                    let text = if is_active {
                        ui_text_small(vlabel).strong()
                    } else if is_cached {
                        ui_text_small(vlabel)
                    } else {
                        ui_text_small(vlabel).weak()
                    };
                    if ui.small_button(text).clicked() {
                        let name = font_name.to_string();
                        self.state.request_download_variant(&name, *vi, &ctx);
                    }
                }
            });
        }

        ui.horizontal(|ui| {
            if ui.small_button("Reset tweaks").clicked() {
                self.state.reset_tweak(font_name);
            }
        });

        if changed {
            let t = self.state.tweak_for_mut(font_name);
            t.scale = scale;
            t.y_offset = y_offset;
            t.y_offset_factor = y_offset_factor;
            if size_off != 0.0 {
                self.state.font_size_offsets.insert(font_name.to_string(), size_off);
            } else {
                self.state.font_size_offsets.remove(font_name);
            }
            self.state.fonts_dirty = true;
        }
    }

    fn show_font_section(&mut self, ui: &mut Ui, category: FontCategory) {
        let ctx = ui.ctx().clone();
        let label = match category {
            FontCategory::Proportional => "Proportional",
            FontCategory::Monospace => "Monospace",
        };

        let applied = match category {
            FontCategory::Monospace => self.state.applied_mono.clone(),
            FontCategory::Proportional => self.state.applied_prop.clone(),
        };
        let is_active = self.state.active_category == category;
        let should_scroll = self.state.selection_changed == Some(category);

        ui.horizontal(|ui| {
            let header = if is_active {
                RichText::new(label).font(ui_font()).strong().size(14.0)
            } else {
                RichText::new(label)
                    .font(ui_font())
                    .size(14.0)
                    .color(Color32::GRAY)
            };
            ui.label(header);
            if let Some(ref name) = applied {
                ui.label(
                    RichText::new(format!(" > {}", name))
                        .font(ui_font())
                        .color(Color32::LIGHT_GREEN),
                );
            }
        });
        ui.add_space(2.0);

        let indices = self.state.indices_for(category).to_vec();
        let current_selected = self.state.selected_for(category);

        ScrollArea::vertical()
            .id_salt(format!("scroll_{label}"))
            .show(ui, |ui| {
                for (list_idx, &cat_idx) in indices.iter().enumerate() {
                    // Copy name and variant label to owned strings so we don't hold
                    // an immutable borrow on self.state.catalog across mutable calls.
                    let name = self.state.catalog[cat_idx].name.to_string();
                    let name: &str = &name;
                    let is_selected = list_idx == current_selected;
                    let is_downloaded = self.state.is_downloaded(name);
                    let is_renderable = self.state.is_renderable(name);
                    let is_downloading = self.state.is_downloading(name);
                    // Check errors by active variant cache key
                    let has_error = self.state.active_cache_key(name)
                        .map(|ck| self.state.errors.contains_key(ck))
                        .unwrap_or(false);
                    let tweak = self.state.tweak_for(name);
                    let has_custom_tweak =
                        tweak != FontTweak::default() || self.state.size_offset_for(name) != 0.0;
                    let eff_size = self.state.effective_size(name);
                    let glyph_count = self.state.glyph_count(name);
                    let active_var = self.state.active_variant(name);
                    let variant_label = self.state.catalog[cat_idx].variants.get(active_var).map(|v| v.label.to_string()).unwrap_or_else(|| "Regular".into());

                    let item_response = ui.push_id(format!("{label}_{list_idx}"), |ui| {
                        let frame = if is_selected && is_active {
                            egui::Frame::NONE
                                .fill(selection_bg(ui))
                                .inner_margin(4.0)
                        } else if is_selected {
                            egui::Frame::NONE
                                .fill(selection_bg_dim(ui))
                                .inner_margin(4.0)
                        } else {
                            egui::Frame::NONE.inner_margin(4.0)
                        };

                        frame.show(ui, |ui| {
                            ui.set_min_width(ui.available_width());

                            ui.horizontal(|ui| {
                                let (marker, color) = if is_downloaded {
                                    ("*", Color32::LIGHT_GREEN)
                                } else if is_downloading {
                                    ("~", Color32::YELLOW)
                                } else if has_error {
                                    ("!", Color32::LIGHT_RED)
                                } else {
                                    ("-", Color32::GRAY)
                                };
                                ui.label(RichText::new(marker).font(ui_font()).color(color));

                                let name_text = if is_selected && is_active {
                                    RichText::new(name).font(ui_font()).strong()
                                } else {
                                    ui_text(name)
                                };
                                ui.label(name_text);

                                // Show active variant label if not Regular
                                if variant_label != "Regular" {
                                    ui.label(
                                        ui_text_small(&variant_label)
                                            .color(Color32::LIGHT_BLUE),
                                    );
                                }

                                if is_downloading {
                                    ui.spinner();
                                }

                                // Right side: glyph count + size info
                                if is_downloaded {
                                    ui.with_layout(
                                        Layout::right_to_left(Align::Center),
                                        |ui| {
                                            let mut info = if has_custom_tweak {
                                                format!(
                                                    "{:.0}px s:{:.2} y:{:.1}",
                                                    eff_size, tweak.scale, tweak.y_offset,
                                                )
                                            } else {
                                                format!("{:.0}px", eff_size)
                                            };
                                            if let Some(count) = glyph_count {
                                                info = format!("{count} glyphs  {info}");
                                            }
                                            ui.label(ui_text_small(&info).color(Color32::GRAY));
                                        },
                                    );
                                }
                            });

                            if is_renderable {
                                let sample_font =
                                    FontId::new(eff_size, FontFamily::Name(name.into()));
                                let row_height = ui.fonts_mut(|f| f.row_height(&sample_font));
                                ui.horizontal(|ui| {
                                    ui.label(
                                        ui_text_small(&format!("{:.1}", row_height))
                                            .color(Color32::GRAY),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                        ui.vertical(|ui| {
                                            ui.with_layout(
                                                Layout::right_to_left(Align::Min),
                                                |ui| {
                                                    ui.label(
                                                        RichText::new(&self.state.sample_text_1)
                                                            .font(sample_font.clone()),
                                                    );
                                                },
                                            );
                                            ui.with_layout(
                                                Layout::right_to_left(Align::Min),
                                                |ui| {
                                                    ui.label(
                                                        RichText::new(&self.state.sample_text_2)
                                                            .font(sample_font.clone()),
                                                    );
                                                },
                                            );
                                        });
                                    });
                                });
                            }

                            if is_selected && is_active && is_downloaded {
                                ui.add_space(2.0);
                                self.show_tweaks_for(ui, name);
                            }

                            if has_error {
                                if let Some(ck) = self.state.active_cache_key(name) {
                                    if let Some(err) = self.state.errors.get(ck) {
                                        ui.label(ui_text_small(err).color(Color32::LIGHT_RED));
                                    }
                                }
                            }
                        })
                    });

                    let item_rect = item_response.response.rect;
                    // Use hover sense when selected so inner buttons (tweaks, variants)
                    // can receive clicks. Use click sense otherwise to select the item.
                    let sense = if is_selected && is_active {
                        Sense::hover()
                    } else {
                        Sense::click()
                    };
                    let click = ui.interact(
                        item_rect,
                        ui.id().with(("click", label, list_idx)),
                        sense,
                    );
                    if click.clicked() {
                        self.state.active_category = category;
                        *self.state.selected_for_mut(category) = list_idx;
                        let name = self.state.catalog[cat_idx].name.to_string();
                        self.state.request_download(&name, &ctx);
                        self.state.apply_selected_for(category);
                    }

                    // Scroll to selected item on keyboard nav
                    if is_selected && should_scroll {
                        ui.scroll_to_rect(item_rect, Some(egui::Align::Center));
                    }

                    ui.add_space(1.0);
                    let sep_rect = ui.available_rect_before_wrap();
                    ui.painter().hline(
                        sep_rect.x_range(),
                        sep_rect.top(),
                        Stroke::new(
                            0.5,
                            ui.visuals()
                                .widgets
                                .noninteractive
                                .bg_stroke
                                .color
                                .gamma_multiply(0.3),
                        ),
                    );
                    ui.add_space(1.0);
                }
            });
    }
}
