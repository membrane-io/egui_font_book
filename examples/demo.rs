#![allow(unused_must_use, deprecated)]
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Grid, Rect, RichText, ScrollArea, Stroke, StrokeKind, TextEdit, Ui};
use egui_dev_tools::WidgetInspect;
use egui_font_book::{FontBookState, FontBrowser, SavedState};

const STORAGE_KEY: &str = "font_book_state";

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Font Book",
        options,
        Box::new(|cc| {
            cc.egui_ctx.add_plugin(WidgetInspect::new());
            let mut app = DemoApp::new();
            if let Some(storage) = cc.storage {
                if let Some(saved) = eframe::get_value::<SavedState>(storage, STORAGE_KEY) {
                    app.font_state.restore_state(saved);
                }
            }
            Ok(Box::new(app))
        }),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(DemoApp::new()))),
            )
            .await
            .expect("failed to start eframe");
    });
}

// ── Demo App ──────────────────────────────────────────────────────────

#[derive(PartialEq)]
enum Mode {
    WidgetGallery,
    CodeEditor,
    Canvas,
    Unicode,
}

struct DemoApp {
    font_state: FontBookState,
    mode: Mode,
    gallery: GalleryState,
    code: String,
}

impl DemoApp {
    fn new() -> Self {
        Self {
            font_state: FontBookState::new(),
            mode: Mode::WidgetGallery,
            gallery: GalleryState::default(),
            code: TYPESCRIPT_SAMPLE.to_string(),
        }
    }
}

impl eframe::App for DemoApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, STORAGE_KEY, &self.font_state.save_state());
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        // Left panel: font browser — 700px default ≈ 50% of 1400px window.
        let total = ui.ctx().viewport_rect().size().x;
        egui::Panel::left("font_browser_v3")
            .default_size(total - 100.0)
            .min_size(700.0)
            .show_inside(ui, |ui| {
                FontBrowser::new(&mut self.font_state).show(ui);
            });

        // Top bar: mode tabs + active font info
        egui::Panel::top("mode_tabs").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.mode, Mode::WidgetGallery, "Widget Gallery");
                ui.selectable_value(&mut self.mode, Mode::CodeEditor, "Code Editor");
                ui.selectable_value(&mut self.mode, Mode::Canvas, "Canvas");
                ui.selectable_value(&mut self.mode, Mode::Unicode, "Unicode");

                ui.separator();

                // Show active fonts
                if let Some(name) = self.font_state.applied_prop_name() {
                    ui.label(RichText::new(format!("Prop: {name}")).small().weak());
                }
                if let Some(name) = self.font_state.applied_mono_name() {
                    ui.label(RichText::new(format!("Mono: {name}")).small().weak());
                }
            });
        });

        // Central panel: preview
        egui::CentralPanel::default().show_inside(ui, |ui| match self.mode {
            Mode::WidgetGallery => widget_gallery(ui, &mut self.gallery),
            Mode::CodeEditor => code_editor(ui, &mut self.code),
            Mode::Unicode => unicode_explorer(ui),
            Mode::Canvas => canvas_demo(ui),
        });
    }
}

// ── Widget Gallery ────────────────────────────────────────────────────
// Demonstrates how proportional and monospace fonts work together in a
// realistic UI. All standard egui widgets use the proportional font by
// default; explicit `FontId::monospace()` or `ui.monospace()` use the
// monospace font. Both change when you select fonts in the browser.

struct GalleryState {
    checkbox_a: bool,
    checkbox_b: bool,
    radio: usize,
    slider_f: f32,
    slider_i: i32,
    text_single: String,
    text_multi: String,
    combo_selected: usize,
    progress: f32,
    toggle: bool,
    name: String,
    email: String,
    age: f32,
    log_filter: String,
}

impl Default for GalleryState {
    fn default() -> Self {
        Self {
            checkbox_a: true,
            checkbox_b: false,
            radio: 0,
            slider_f: 42.0,
            slider_i: 7,
            text_single: "Hello, fonts!".into(),
            text_multi: "Multi-line\ntext editing\nwith your chosen font.".into(),
            combo_selected: 0,
            progress: 0.65,
            toggle: false,
            name: "Ada Lovelace".into(),
            email: "ada@example.com".into(),
            age: 36.0,
            log_filter: String::new(),
        }
    }
}

const COMBO_ITEMS: &[&str] = &[
    "Dashboard",
    "Analytics",
    "Settings",
    "Profile",
    "Notifications",
];

fn widget_gallery(ui: &mut egui::Ui, state: &mut GalleryState) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Widget Gallery");
        ui.label("Proportional font is used for all standard widgets. Monospace is used where explicitly set.");
        ui.separator();

        // ── Mixed Font Section ──
        // This section explicitly demonstrates both font types side by side.
        ui.label(RichText::new("Mixed Fonts: Proportional + Monospace").strong().size(16.0));
        ui.add_space(4.0);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label("This label uses the proportional font (default).");
            ui.monospace("This label uses the monospace font (ui.monospace).");
            ui.add_space(4.0);

            Grid::new("mixed_font_grid")
                .num_columns(2)
                .spacing(vec2(16.0, 4.0))
                .show(ui, |ui| {
                    ui.label("Property");
                    ui.label(RichText::new("Value (monospace)").font(FontId::monospace(14.0)));
                    ui.end_row();

                    ui.label("Host");
                    ui.monospace("api-prod-us-east-1.example.com");
                    ui.end_row();

                    ui.label("Email");
                    ui.monospace("ada.lovelace@analytical-engine.io");
                    ui.end_row();

                    ui.label("Endpoint");
                    ui.monospace("https://api.example.com/v2/dashboards?sort=created&limit=50");
                    ui.end_row();

                    ui.label("Request ID");
                    ui.monospace("req_7f3a2b1c-d4e5-6789-abcd-ef0123456789");
                    ui.end_row();

                    ui.label("Status");
                    ui.monospace("200 OK");
                    ui.end_row();

                    ui.label("Duration");
                    ui.monospace("142.3ms");
                    ui.end_row();

                    ui.label("Timestamp");
                    ui.monospace("2026-03-24T08:15:42.123Z");
                    ui.end_row();

                    ui.label("File Path");
                    ui.monospace("/var/log/app/2026-03-24/access.log.gz");
                    ui.end_row();

                    ui.label("IP Address");
                    ui.monospace("192.168.1.42:8080 -> 10.0.0.1:443");
                    ui.end_row();

                    ui.label("Cost");
                    ui.monospace("$1,234.56 (+12.3% \u{0394})");
                    ui.end_row();

                    ui.label("Expression");
                    ui.monospace("cpu_usage >= 80% && mem_free <= 512MB");
                    ui.end_row();

                    ui.label("Checksum");
                    ui.monospace("sha256:9f86d08...c150e04a");
                    ui.end_row();
                });
        });
        ui.add_space(8.0);

        // ── Log Viewer (monospace in proportional context) ──
        ui.label(RichText::new("Log Viewer").strong().size(16.0));
        ui.label("Filter and log lines use monospace; labels use proportional.");
        ui.add_space(2.0);

        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.add(
                TextEdit::singleline(&mut state.log_filter)
                    .font(FontId::monospace(13.0))
                    .desired_width(200.0),
            );
        });

        egui::Frame::group(ui.style()).show(ui, |ui| {
            let logs = [
                "[INFO ] 08:15:42.123  GET /api/dashboards           200  142ms",
                "[INFO ] 08:15:42.267  POST /api/generate             202   1.8s",
                "[WARN ] 08:15:43.001  GET /api/metrics?src=cpu       200  890ms  (slow)",
                "[ERROR] 08:15:43.512  POST /api/widgets              500   45ms  internal error",
                "[INFO ] 08:15:43.890  GET /api/health                200    2ms",
                "[DEBUG] 08:15:44.001  Cache hit for dashboard_7f3a   ratio=0.87",
                "[INFO ] 08:15:44.123  WS   /api/stream               101  upgrade",
            ];
            for line in logs {
                if state.log_filter.is_empty()
                    || line.to_lowercase().contains(&state.log_filter.to_lowercase())
                {
                    ui.monospace(line);
                }
            }
        });
        ui.add_space(12.0);

        // ── Inline Code in Prose ──
        ui.label(RichText::new("Inline Code in Documentation").strong().size(16.0));
        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0; // keep baseline aligned
            ui.label("Call");
            ui.monospace("generateDashboard(prompt)");
            ui.label("to create a new dashboard. The function returns a");
            ui.monospace("Promise<Dashboard>");
            ui.label("which resolves once the AI model has produced the layout. Use");
            ui.monospace("layout.place(widget)");
            ui.label("to position each widget on the grid.");
        });
        ui.add_space(12.0);

        ui.separator();

        // ── Standard Widget Gallery (all proportional) ──
        ui.label(RichText::new("Standard Widgets (Proportional Font)").strong().size(16.0));
        ui.add_space(4.0);

        // Buttons
        ui.label(RichText::new("Buttons").strong());
        ui.horizontal(|ui| {
            ui.button("Primary");
            ui.button("Secondary");
            ui.add_enabled(false, egui::Button::new("Disabled"));
        });
        ui.add_space(8.0);

        // Checkboxes
        ui.label(RichText::new("Checkboxes").strong());
        ui.checkbox(&mut state.checkbox_a, "Enable notifications");
        ui.checkbox(&mut state.checkbox_b, "Subscribe to newsletter");
        ui.add_space(8.0);

        // Radio
        ui.label(RichText::new("Radio Buttons").strong());
        ui.horizontal(|ui| {
            ui.radio_value(&mut state.radio, 0, "Low");
            ui.radio_value(&mut state.radio, 1, "Medium");
            ui.radio_value(&mut state.radio, 2, "High");
            ui.radio_value(&mut state.radio, 3, "Critical");
        });
        ui.add_space(8.0);

        // Sliders
        ui.label(RichText::new("Sliders").strong());
        ui.add(egui::Slider::new(&mut state.slider_f, 0.0..=100.0).text("Temperature"));
        ui.add(egui::Slider::new(&mut state.slider_i, 0..=20).text("Count"));
        ui.add_space(8.0);

        // Text Input
        ui.label(RichText::new("Text Input").strong());
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut state.text_single);
        });
        ui.label("Notes:");
        ui.add(TextEdit::multiline(&mut state.text_multi).desired_rows(3));
        ui.add_space(8.0);

        // ComboBox
        ui.label(RichText::new("Dropdown").strong());
        egui::ComboBox::from_label("Page")
            .selected_text(COMBO_ITEMS[state.combo_selected])
            .show_ui(ui, |ui| {
                for (i, item) in COMBO_ITEMS.iter().enumerate() {
                    ui.selectable_value(&mut state.combo_selected, i, *item);
                }
            });
        ui.add_space(8.0);

        // Progress
        ui.label(RichText::new("Progress").strong());
        ui.add(
            egui::ProgressBar::new(state.progress)
                .show_percentage()
                .animate(true),
        );
        ui.add(egui::Slider::new(&mut state.progress, 0.0..=1.0).text("Adjust"));
        ui.add_space(8.0);

        // Toggle
        ui.label(RichText::new("Toggle").strong());
        ui.horizontal(|ui| {
            ui.label("Dark mode:");
            let label = if state.toggle { "ON" } else { "OFF" };
            ui.toggle_value(&mut state.toggle, label);
        });
        ui.add_space(8.0);

        // Grid / Table
        ui.label(RichText::new("Data Grid").strong());
        Grid::new("demo_grid")
            .num_columns(3)
            .spacing(vec2(20.0, 4.0))
            .striped(true)
            .show(ui, |ui| {
                ui.label(RichText::new("Property").strong());
                ui.label(RichText::new("Value").strong());
                ui.label(RichText::new("Status").strong());
                ui.end_row();

                for (prop, val, status) in [
                    ("CPU Usage", "47%", "Normal"),
                    ("Memory", "8.2 GB / 16 GB", "Warning"),
                    ("Disk", "234 GB free", "OK"),
                    ("Network", "1.2 Gbps", "Excellent"),
                    ("Uptime", "14d 6h 32m", "OK"),
                ] {
                    ui.label(prop);
                    ui.label(val);
                    ui.label(status);
                    ui.end_row();
                }
            });
        ui.add_space(8.0);

        // Collapsing sections
        ui.label(RichText::new("Collapsing Sections").strong());
        ui.collapsing("System Information", |ui| {
            ui.label("OS: macOS 15.3");
            ui.label("Architecture: aarch64");
            ui.label("Rust: 1.82.0-nightly");
        });
        ui.collapsing("Network Configuration", |ui| {
            ui.label("Interface: en0");
            ui.label("IP: 192.168.1.42");
            ui.label("Gateway: 192.168.1.1");
            ui.label("DNS: 1.1.1.1, 8.8.8.8");
        });
        ui.add_space(8.0);

        // Form layout
        ui.label(RichText::new("Form Layout").strong());
        egui::Frame::group(ui.style()).show(ui, |ui| {
            Grid::new("form_grid")
                .num_columns(2)
                .spacing(vec2(12.0, 6.0))
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut state.name);
                    ui.end_row();

                    ui.label("Email:");
                    ui.text_edit_singleline(&mut state.email);
                    ui.end_row();

                    ui.label("Age:");
                    ui.add(egui::DragValue::new(&mut state.age).range(0.0..=150.0));
                    ui.end_row();
                });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.button("Submit");
                ui.button("Cancel");
                ui.button("Reset");
            });
        });
        ui.add_space(8.0);

        // Typography samples
        ui.label(RichText::new("Typography Samples").strong());
        ui.heading("The quick brown fox jumps over the lazy dog");
        ui.label("The quick brown fox jumps over the lazy dog — Proportional");
        ui.label(RichText::new("The quick brown fox jumps over the lazy dog — Bold").strong());
        ui.monospace("The quick brown fox jumps over the lazy dog — Monospace");
        ui.label(RichText::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ").size(16.0));
        ui.label(RichText::new("abcdefghijklmnopqrstuvwxyz").size(16.0));
        ui.label(RichText::new("0123456789 !@#$%^&*()-=[]{}|;':\",./<>?").size(16.0));
        ui.monospace("0123456789 !@#$%^&*()-=[]{}|;':\",./<>?");
        ui.add_space(16.0);
    });
}

// ── Code Editor ───────────────────────────────────────────────────────

// ── Canvas Demo ───────────────────────────────────────────────────────
// Painted shapes with text labels — tests how fonts render alongside
// vector graphics, which is common in node editors, diagrams, and dashboards.

fn canvas_demo(ui: &mut egui::Ui) {
    ui.heading("Canvas");
    ui.label("Shapes with text labels — previews fonts in a painted/rendered context.");
    ui.separator();

    let (response, painter) =
        ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::hover());
    let rect = response.rect;
    let c = rect.center();

    // Background grid
    let grid_color = Color32::from_gray(40);
    let step = 40.0;
    let mut x = rect.left();
    while x < rect.right() {
        painter.line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], Stroke::new(0.5, grid_color));
        x += step;
    }
    let mut y = rect.top();
    while y < rect.bottom() {
        painter.line_segment([pos2(rect.left(), y), pos2(rect.right(), y)], Stroke::new(0.5, grid_color));
        y += step;
    }

    let prop = |size: f32| FontId::new(size, egui::FontFamily::Proportional);
    let mono = |size: f32| FontId::new(size, egui::FontFamily::Monospace);

    // ── Node 1: "API Gateway" ──
    let n1 = Rect::from_center_size(c + vec2(-180.0, -120.0), vec2(200.0, 80.0));
    painter.rect_filled(n1, 6.0, Color32::from_rgb(35, 55, 80));
    painter.rect_stroke(n1, 6.0, Stroke::new(1.0, Color32::from_rgb(70, 130, 200)), StrokeKind::Inside);
    painter.text(n1.center_top() + vec2(0.0, 12.0), Align2::CENTER_TOP, "API Gateway", prop(16.0), Color32::WHITE);
    painter.text(n1.center_top() + vec2(0.0, 32.0), Align2::CENTER_TOP, "POST /api/generate", mono(11.0), Color32::from_rgb(150, 200, 255));
    painter.text(n1.center_top() + vec2(0.0, 48.0), Align2::CENTER_TOP, "latency: 142ms", mono(11.0), Color32::from_rgb(120, 170, 120));

    // ── Node 2: "Dashboard Engine" ──
    let n2 = Rect::from_center_size(c + vec2(140.0, -120.0), vec2(200.0, 80.0));
    painter.rect_filled(n2, 6.0, Color32::from_rgb(55, 35, 70));
    painter.rect_stroke(n2, 6.0, Stroke::new(1.0, Color32::from_rgb(160, 100, 200)), StrokeKind::Inside);
    painter.text(n2.center_top() + vec2(0.0, 12.0), Align2::CENTER_TOP, "Dashboard Engine", prop(16.0), Color32::WHITE);
    painter.text(n2.center_top() + vec2(0.0, 32.0), Align2::CENTER_TOP, "widgets: 12", mono(11.0), Color32::from_rgb(200, 170, 255));
    painter.text(n2.center_top() + vec2(0.0, 48.0), Align2::CENTER_TOP, "grid: 12×8", mono(11.0), Color32::from_rgb(200, 170, 255));

    // ── Connection arrow ──
    let from = pos2(n1.right(), n1.center().y);
    let to = pos2(n2.left(), n2.center().y);
    painter.line_segment([from, to], Stroke::new(2.0, Color32::from_rgb(100, 100, 100)));
    let mid = pos2((from.x + to.x) * 0.5, from.y - 14.0);
    painter.text(mid, Align2::CENTER_BOTTOM, "generate()", mono(10.0), Color32::from_rgb(180, 180, 180));

    // ── Node 3: "Metrics Store" ──
    let n3 = Rect::from_center_size(c + vec2(-180.0, 40.0), vec2(200.0, 80.0));
    painter.rect_filled(n3, 6.0, Color32::from_rgb(35, 60, 45));
    painter.rect_stroke(n3, 6.0, Stroke::new(1.0, Color32::from_rgb(80, 180, 100)), StrokeKind::Inside);
    painter.text(n3.center_top() + vec2(0.0, 12.0), Align2::CENTER_TOP, "Metrics Store", prop(16.0), Color32::WHITE);
    painter.text(n3.center_top() + vec2(0.0, 32.0), Align2::CENTER_TOP, "source: cpu, mem, net", mono(11.0), Color32::from_rgb(140, 220, 160));
    painter.text(n3.center_top() + vec2(0.0, 48.0), Align2::CENTER_TOP, "agg: p99 | avg | sum", mono(11.0), Color32::from_rgb(140, 220, 160));

    // ── Node 4: "Layout Grid" ──
    let n4 = Rect::from_center_size(c + vec2(140.0, 40.0), vec2(200.0, 80.0));
    painter.rect_filled(n4, 6.0, Color32::from_rgb(60, 50, 30));
    painter.rect_stroke(n4, 6.0, Stroke::new(1.0, Color32::from_rgb(200, 170, 80)), StrokeKind::Inside);
    painter.text(n4.center_top() + vec2(0.0, 12.0), Align2::CENTER_TOP, "Layout Grid", prop(16.0), Color32::WHITE);
    painter.text(n4.center_top() + vec2(0.0, 32.0), Align2::CENTER_TOP, "cols=12 rows=8", mono(11.0), Color32::from_rgb(230, 200, 120));
    painter.text(n4.center_top() + vec2(0.0, 48.0), Align2::CENTER_TOP, "placed: 4/12", mono(11.0), Color32::from_rgb(230, 200, 120));

    // ── Vertical connections ──
    painter.line_segment(
        [pos2(n1.center().x, n1.bottom()), pos2(n3.center().x, n3.top())],
        Stroke::new(1.5, Color32::from_rgb(80, 80, 80)),
    );
    painter.line_segment(
        [pos2(n2.center().x, n2.bottom()), pos2(n4.center().x, n4.top())],
        Stroke::new(1.5, Color32::from_rgb(80, 80, 80)),
    );
    painter.line_segment(
        [pos2(n3.right(), n3.center().y), pos2(n4.left(), n4.center().y)],
        Stroke::new(1.5, Color32::from_rgb(80, 80, 80)),
    );
    let mid2 = pos2((n3.right() + n4.left()) * 0.5, n3.center().y - 14.0);
    painter.text(mid2, Align2::CENTER_BOTTOM, "queryMetrics()", mono(10.0), Color32::from_rgb(180, 180, 180));

    // ── Title / legend at bottom ──
    let legend_y = n4.bottom() + 40.0;
    painter.text(
        pos2(c.x, legend_y),
        Align2::CENTER_TOP,
        "Dashboard Generation Pipeline",
        prop(20.0),
        Color32::from_rgb(220, 220, 220),
    );
    painter.text(
        pos2(c.x, legend_y + 28.0),
        Align2::CENTER_TOP,
        "Proportional font: node titles, heading  |  Monospace font: values, function names",
        prop(12.0),
        Color32::from_rgb(140, 140, 140),
    );

    // ── Miniature stat badges ──
    let badge_y = legend_y + 60.0;
    for (i, (label, value, color)) in [
        ("CPU", "47%", Color32::from_rgb(80, 180, 100)),
        ("MEM", "8.2G", Color32::from_rgb(100, 150, 220)),
        ("REQ/s", "1.2k", Color32::from_rgb(200, 170, 80)),
        ("ERR", "0.3%", Color32::from_rgb(200, 90, 90)),
        ("P99", "142ms", Color32::from_rgb(160, 100, 200)),
    ]
    .iter()
    .enumerate()
    {
        let bx = c.x - 200.0 + (i as f32) * 90.0;
        let badge = Rect::from_min_size(pos2(bx, badge_y), vec2(80.0, 36.0));
        painter.rect_filled(badge, 4.0, Color32::from_gray(30));
        painter.rect_stroke(badge, 4.0, Stroke::new(1.0, *color), StrokeKind::Inside);
        painter.text(badge.left_top() + vec2(6.0, 4.0), Align2::LEFT_TOP, label, prop(10.0), Color32::GRAY);
        painter.text(badge.left_top() + vec2(6.0, 17.0), Align2::LEFT_TOP, value, mono(14.0), *color);
    }
}

// ── Code Editor ───────────────────────────────────────────────────────

// ── Unicode Explorer ──────────────────────────────────────────────────

const UNICODE_RANGES: &[(&str, u32, u32)] = &[
    ("Basic Latin", 0x0020, 0x007E),
    ("Latin-1 Supplement", 0x00A0, 0x00FF),
    ("Latin Extended-A", 0x0100, 0x017F),
    ("Latin Extended-B", 0x0180, 0x024F),
    ("Greek and Coptic", 0x0370, 0x03FF),
    ("Cyrillic", 0x0400, 0x04FF),
    ("General Punctuation", 0x2000, 0x206F),
    ("Superscripts and Subscripts", 0x2070, 0x209F),
    ("Currency Symbols", 0x20A0, 0x20CF),
    ("Letterlike Symbols", 0x2100, 0x214F),
    ("Number Forms", 0x2150, 0x218F),
    ("Arrows", 0x2190, 0x21FF),
    ("Mathematical Operators", 0x2200, 0x22FF),
    ("Miscellaneous Technical", 0x2300, 0x23FF),
    ("Box Drawing", 0x2500, 0x257F),
    ("Block Elements", 0x2580, 0x259F),
    ("Geometric Shapes", 0x25A0, 0x25FF),
    ("Miscellaneous Symbols", 0x2600, 0x26FF),
    ("Dingbats", 0x2700, 0x27BF),
    ("Braille Patterns", 0x2800, 0x28FF),
    ("CJK Symbols and Punctuation", 0x3000, 0x303F),
    ("Enclosed Alphanumerics", 0x2460, 0x24FF),
];

fn unicode_explorer(ui: &mut egui::Ui) {
    ui.heading("Unicode Explorer");
    ui.label("Common Unicode ranges rendered in the currently selected fonts.");
    ui.separator();

    ScrollArea::vertical().show(ui, |ui| {
        for &(name, start, end) in UNICODE_RANGES {
            ui.label(
                RichText::new(format!("{name}  U+{start:04X}..U+{end:04X}")).strong(),
            );

            let cols: usize = 16;
            Grid::new(format!("unicode_{name}"))
                .num_columns(cols + 1)
                .spacing(vec2(2.0, 1.0))
                .min_col_width(0.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("").small());
                    for col in 0..cols {
                        ui.label(RichText::new(format!("_{col:X}")).small().weak());
                    }
                    ui.end_row();

                    let mut cp = start;
                    while cp <= end {
                        let row_base = cp & !0xF;
                        ui.label(
                            RichText::new(format!("{:04X}", row_base)).small().weak(),
                        );

                        for col in 0..cols as u32 {
                            let code = row_base + col;
                            if code >= start && code <= end {
                                if let Some(ch) = char::from_u32(code) {
                                    let r = ui.label(ch.to_string());
                                    r.on_hover_text(format!("U+{code:04X}"));
                                } else {
                                    ui.label(" ");
                                }
                            } else {
                                ui.label(" ");
                            }
                        }
                        ui.end_row();

                        cp = row_base + cols as u32;
                    }
                });

            // Monospace rendering of the same range
            ui.add_space(2.0);
            ui.label(RichText::new("Monospace:").small().weak());
            let line: String = (start..=end)
                .filter_map(char::from_u32)
                .filter(|c| !c.is_control())
                .collect();
            ui.monospace(&line);

            ui.add_space(12.0);
        }
    });
}

// ── Code Editor ───────────────────────────────────────────────────────

fn code_editor(ui: &mut egui::Ui, code: &mut String) {
    ui.heading("Code Editor");
    ui.label("Preview of monospace font in a code editing context.");
    ui.separator();

    ScrollArea::vertical().show(ui, |ui| {
        ui.add(
            TextEdit::multiline(code)
                .font(FontId::monospace(13.0))
                .desired_width(f32::INFINITY)
                .desired_rows(40)
                .code_editor(),
        );
    });
}

// ── Sample TypeScript code ────────────────────────────────────────────

const TYPESCRIPT_SAMPLE: &str = r#"import { z } from "zod";

// ── Types ─────────────────────────────────────────────────

interface Dashboard {
  id: string;
  title: string;
  widgets: Widget[];
  createdAt: Date;
  updatedAt: Date;
}

interface Widget {
  type: "chart" | "table" | "metric" | "log";
  position: { x: number; y: number };
  size: { width: number; height: number };
  config: Record<string, unknown>;
}

const WidgetSchema = z.object({
  type: z.enum(["chart", "table", "metric", "log"]),
  position: z.object({ x: z.number(), y: z.number() }),
  size: z.object({ width: z.number(), height: z.number() }),
  config: z.record(z.unknown()),
});

// ── Dashboard Generation ──────────────────────────────────

async function generateDashboard(prompt: string): Promise<Dashboard> {
  const response = await fetch("/api/generate", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ prompt, model: "claude-4" }),
  });

  if (!response.ok) {
    throw new Error(`Generation failed: ${response.statusText}`);
  }

  const data = await response.json();
  const widgets = data.widgets.map(parseWidget);

  return {
    id: crypto.randomUUID(),
    title: data.title ?? "Untitled Dashboard",
    widgets,
    createdAt: new Date(),
    updatedAt: new Date(),
  };
}

function parseWidget(raw: Record<string, unknown>): Widget {
  const parsed = WidgetSchema.parse(raw);
  return {
    type: parsed.type,
    position: parsed.position,
    size: parsed.size,
    config: parsed.config,
  };
}

// ── Layout Engine ─────────────────────────────────────────

class GridLayout {
  private cells: Map<string, Widget> = new Map();
  private columns: number;
  private rows: number;

  constructor(columns: number = 12, rows: number = 8) {
    this.columns = columns;
    this.rows = rows;
  }

  place(widget: Widget): { x: number; y: number } | null {
    for (let row = 0; row <= this.rows - widget.size.height; row++) {
      for (let col = 0; col <= this.columns - widget.size.width; col++) {
        if (this.canPlace(col, row, widget.size.width, widget.size.height)) {
          this.occupy(col, row, widget);
          return { x: col, y: row };
        }
      }
    }
    return null; // No space available
  }

  private canPlace(x: number, y: number, w: number, h: number): boolean {
    for (let row = y; row < y + h; row++) {
      for (let col = x; col < x + w; col++) {
        if (this.cells.has(`${col},${row}`)) return false;
      }
    }
    return true;
  }

  private occupy(x: number, y: number, widget: Widget): void {
    for (let row = y; row < y + widget.size.height; row++) {
      for (let col = x; col < x + widget.size.width; col++) {
        this.cells.set(`${col},${row}`, widget);
      }
    }
  }
}

// ── Metric Computation ────────────────────────────────────

type AggregationFn = "sum" | "avg" | "min" | "max" | "p99" | "count";

interface MetricQuery {
  source: string;
  field: string;
  aggregation: AggregationFn;
  groupBy?: string[];
  filters?: Record<string, string | number | boolean>;
  timeRange: { start: Date; end: Date };
}

async function queryMetrics(query: MetricQuery): Promise<number[]> {
  const params = new URLSearchParams({
    source: query.source,
    field: query.field,
    agg: query.aggregation,
    start: query.timeRange.start.toISOString(),
    end: query.timeRange.end.toISOString(),
  });

  if (query.groupBy?.length) {
    params.set("groupBy", query.groupBy.join(","));
  }

  if (query.filters) {
    for (const [key, value] of Object.entries(query.filters)) {
      params.set(`filter.${key}`, String(value));
    }
  }

  const res = await fetch(`/api/metrics?${params}`);
  const json = await res.json();
  return json.values as number[];
}

// ── Entry Point ───────────────────────────────────────────

async function main() {
  console.log("Generating dashboard...");

  const dashboard = await generateDashboard(
    "Create a monitoring dashboard for a web API with response times, " +
    "error rates, throughput, and server resource utilization"
  );

  const layout = new GridLayout();

  for (const widget of dashboard.widgets) {
    const position = layout.place(widget);
    if (position) {
      widget.position = position;
      console.log(`Placed ${widget.type} at (${position.x}, ${position.y})`);
    } else {
      console.warn(`Could not place widget: ${widget.type}`);
    }
  }

  console.log(`Dashboard "${dashboard.title}" ready with ${dashboard.widgets.length} widgets`);
}

main().catch(console.error);
"#;
