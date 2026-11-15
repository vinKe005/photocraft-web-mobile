//! Touch chrome for phones (the Android host and the mobile browser layout), independent of the
//! desktop dock preferences.
//!
//! Laid out like phone photo editors: a top bar with Close, Undo, Redo, Layers, More and a
//! prominent Export; and at the bottom, from the screen edge up, the category tabs (Draw,
//! Adjustments, Crop, Text, Selection), the category's tools and actions as icon tiles, and the
//! active tool's context: a large slider for the brush tools, Cancel / Crop while a crop is
//! pending, and the full tool options on request. Every action dispatches a command id (Rule 1).
use crate::{PhotocraftApp, Tool, icons, theme::Tokens};
use egui::{Align, FontId, Layout, Rect, Response, Sense, Vec2, vec2};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MobileUi {
    /// Enable the phone layout in offscreen previews, independent of the target OS.
    #[serde(default)]
    pub enabled: bool,
    pub layers_open: bool,
    /// The full desktop options bar for the active tool, under the top bar.
    pub options_open: bool,
    /// The category whose tools the bottom bar shows.
    #[serde(default)]
    pub tab: MobileTab,
    /// The brush setting the context slider edits.
    #[serde(default)]
    pub setting: BrushSetting,
}

impl MobileUi {
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

/// The bottom bar's categories.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MobileTab {
    #[default]
    Draw,
    Adjust,
    Crop,
    Text,
    Select,
}

impl MobileTab {
    pub const ALL: [MobileTab; 5] = [MobileTab::Draw, MobileTab::Adjust, MobileTab::Crop, MobileTab::Text, MobileTab::Select];

    /// English source text, translated by `tl!`.
    pub fn label(self) -> &'static str {
        match self {
            MobileTab::Draw => "Draw",
            MobileTab::Adjust => "Adjustments",
            MobileTab::Crop => "Crop",
            MobileTab::Text => "Text",
            MobileTab::Select => "Selection",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            MobileTab::Draw => "brush",
            MobileTab::Adjust => "sliders-horizontal",
            MobileTab::Crop => "crop",
            MobileTab::Text => "type",
            MobileTab::Select => "square-dashed",
        }
    }

    /// The tools and actions the category offers, in order.
    pub fn items(self) -> &'static [Item] {
        match self {
            MobileTab::Draw => DRAW,
            MobileTab::Adjust => ADJUST,
            MobileTab::Crop => CROP,
            MobileTab::Text => TEXT,
            MobileTab::Select => SELECT,
        }
    }

    /// The category that holds `tool`, if one does.
    pub fn of(tool: Tool) -> Option<MobileTab> {
        MobileTab::ALL.into_iter().find(|tab| tab.items().contains(&Item::Tool(tool)))
    }
}

/// A tile in the bottom bar: a tool to select, or a command to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Tool(Tool),
    /// Command id, its menu label (English source text) and its icon.
    Command(&'static str, &'static str, &'static str),
}

impl Item {
    fn icon(self) -> &'static str {
        match self {
            Item::Tool(tool) => icons::tool_icon(tool),
            Item::Command(_, _, icon) => icon,
        }
    }

    /// The label shown under the icon: translated, short (a tool's name without "Tool"), without
    /// a menu item's trailing ellipsis.
    fn label(self) -> &'static str {
        let label = match self {
            Item::Tool(tool) => short_tool_label(tool.label()),
            Item::Command(_, label, _) => crate::i18n::t(label),
        };
        label.trim_end_matches('…').trim_end_matches("...")
    }
}

/// `full` ("Brush Tool") as its translated short name ("Brush"), or the translated full name when
/// the catalog has no short one.
fn short_tool_label(full: &'static str) -> &'static str {
    let Some(short) = full.strip_suffix(" Tool") else { return crate::i18n::t(full) };
    let translated = crate::i18n::t(short);
    if translated == short && crate::i18n::t(full) != full { crate::i18n::t(full) } else { translated }
}

const DRAW: &[Item] = &[
    Item::Tool(Tool::Brush),
    Item::Tool(Tool::Eraser),
    Item::Tool(Tool::PaintBucket),
    Item::Tool(Tool::Gradient),
    Item::Tool(Tool::Eyedropper),
    Item::Tool(Tool::CloneStamp),
    Item::Tool(Tool::SpotHealing),
    Item::Tool(Tool::Smudge),
    Item::Tool(Tool::Blur),
    Item::Tool(Tool::Pencil),
];

const ADJUST: &[Item] = &[
    Item::Command("image.adjustments.brightnessContrast", "Brightness/Contrast…", "sun"),
    Item::Command("image.adjustments.exposure", "Exposure…", "gauge"),
    Item::Command("image.adjustments.vibrance", "Vibrance…", "flame"),
    Item::Command("image.adjustments.hueSaturation", "Hue/Saturation…", "droplet"),
    Item::Command("image.adjustments.levels", "Levels…", "sliders-horizontal"),
    Item::Command("image.adjustments.curves", "Curves…", "spline"),
    Item::Command("image.adjustments.colorBalance", "Color Balance…", "palette"),
    Item::Command("image.adjustments.shadowsHighlights", "Shadows/Highlights…", "moon"),
    Item::Command("image.adjustments.blackWhite", "Black & White…", "contrast"),
    Item::Command("image.adjustments.photoFilter", "Photo Filter…", "blend"),
];

const CROP: &[Item] = &[
    Item::Tool(Tool::Crop),
    Item::Command("image.imageRotation.90ccw", "90° Counter Clockwise", "rotate-ccw"),
    Item::Command("image.imageRotation.90cw", "90° Clockwise", "rotate-cw"),
    Item::Command("image.imageRotation.flipCanvasHorizontal", "Flip Horizontal", "symmetry-vertical"),
    Item::Command("image.imageRotation.flipCanvasVertical", "Flip Vertical", "symmetry-horizontal"),
    Item::Tool(Tool::Move),
    Item::Tool(Tool::Hand),
];

const TEXT: &[Item] = &[Item::Tool(Tool::Type), Item::Tool(Tool::VerticalType), Item::Tool(Tool::HorizontalTypeMask), Item::Tool(Tool::Move)];

const SELECT: &[Item] = &[
    Item::Command("select.subject", "Subject", "sparkles"),
    Item::Tool(Tool::QuickSelection),
    Item::Tool(Tool::MagicWand),
    Item::Tool(Tool::Lasso),
    Item::Tool(Tool::RectMarquee),
    Item::Tool(Tool::EllipseMarquee),
    Item::Command("select.all", "All", "maximize-2"),
    Item::Command("select.inverse", "Inverse", "squares-subtract"),
    Item::Command("select.deselect", "Deselect", "x"),
];

/// The brush settings the context slider can edit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrushSetting {
    #[default]
    Size,
    Opacity,
    Hardness,
    Flow,
}

impl BrushSetting {
    pub const ALL: [BrushSetting; 4] = [BrushSetting::Size, BrushSetting::Opacity, BrushSetting::Hardness, BrushSetting::Flow];

    pub fn label(self) -> &'static str {
        match self {
            BrushSetting::Size => "Size",
            BrushSetting::Opacity => "Opacity",
            BrushSetting::Hardness => "Hardness",
            BrushSetting::Flow => "Flow",
        }
    }

    /// The slider's range, in the units it shows (pixels, or whole percents).
    pub fn range(self) -> std::ops::RangeInclusive<f32> {
        match self {
            BrushSetting::Size => 1.0..=MAX_SIZE,
            _ => 1.0..=100.0,
        }
    }

    /// The setting's value in the slider's units.
    pub fn get(self, b: &photocraft_paint::BrushSettings) -> f32 {
        let v = match self {
            BrushSetting::Size => b.size,
            BrushSetting::Opacity => b.opacity * 100.0,
            BrushSetting::Hardness => b.hardness * 100.0,
            BrushSetting::Flow => b.flow * 100.0,
        };
        if v.is_finite() { v.clamp(*self.range().start(), *self.range().end()) } else { *self.range().start() }
    }

    /// Stores `v` (the slider's units, clamped to its range) into the brush.
    pub fn set(self, b: &mut photocraft_paint::BrushSettings, v: f32) {
        let r = self.range();
        let v = if v.is_finite() { v.clamp(*r.start(), *r.end()) } else { *r.start() };
        match self {
            BrushSetting::Size => b.size = v.round(),
            BrushSetting::Opacity => b.opacity = v.round() / 100.0,
            BrushSetting::Hardness => b.hardness = v.round() / 100.0,
            BrushSetting::Flow => b.flow = v.round() / 100.0,
        }
    }

    /// Hardness applies to computed round tips only; the others to every brush tool.
    fn applies(self, tool: Tool) -> bool {
        match self {
            BrushSetting::Flow => !matches!(tool, Tool::Pencil | Tool::Smudge | Tool::Blur),
            _ => true,
        }
    }
}

/// Largest size the slider offers, in pixels.
const MAX_SIZE: f32 = 2500.0;
/// Minimum touch target (Material and Apple guidance).
const TOUCH_TARGET: f32 = 48.0;
const TOP_BAR: f32 = 52.0;
const TAB_HEIGHT: f32 = 54.0;
const TILE: Vec2 = vec2(80.0, 66.0);

/// Height reserved for Layers while retaining a usable canvas.
pub fn layer_sheet_height(height: f32) -> f32 {
    // Leave space for the canvas even in landscape and with the keyboard visible.
    if height.is_finite() { (height * 0.45).clamp(0.0, 360.0) } else { 0.0 }
}

/// Runs a command (or a shell UI command) by id; a refusal shows in the status line.
fn run(app: &mut PhotocraftApp, ctx: &egui::Context, id: &str) {
    if let Err(error) = crate::menus::invoke(app, ctx, id, json!({})) {
        app.ui.status = error;
        app.ui.status_error = true;
    }
}

/// An icon with its label underneath, the size of a fingertip.
fn tile(ui: &mut egui::Ui, size: Vec2, icon: &str, label: &str, selected: bool) -> Response {
    let t = Tokens::get(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let response = response.on_hover_text(label);
    if ui.is_rect_visible(rect) {
        if response.is_pointer_button_down_on() {
            ui.painter().rect_filled(rect.shrink(2.0), t.radius, t.pressed);
        }
        let color = if selected { t.accent } else { t.text_dim };
        let icon_rect = Rect::from_center_size(rect.center_top() + vec2(0.0, 20.0), Vec2::splat(24.0));
        icons::paint(ui, icon_rect, icon, 24.0, color);
        let mut job = egui::text::LayoutJob::simple(label.to_owned(), FontId::proportional(11.0), color, rect.width() - 4.0);
        // Long names ("Pincel de recuperação para manchas") take two lines, then an ellipsis.
        job.wrap.max_rows = 2;
        job.wrap.overflow_character = Some('…');
        job.halign = Align::Center;
        let galley = ui.painter().layout_job(job);
        ui.painter().with_clip_rect(rect).galley(egui::pos2(rect.center().x, rect.top() + 36.0), galley, color);
    }
    response
}

/// A rounded chip that picks one of a few options.
fn chip(ui: &mut egui::Ui, label: &str, selected: bool) -> Response {
    let t = Tokens::get(ui.ctx());
    let (fill, text) = if selected { (t.accent, t.primary_text) } else { (t.field, t.text) };
    ui.add(egui::Button::new(egui::RichText::new(label).color(text)).fill(fill).corner_radius(16.0).min_size(vec2(0.0, 34.0)))
}

/// Close, Undo, Redo, Layers, More and Export; New and Open on the Home screen.
fn top_bar(app: &mut PhotocraftApp, ui: &mut egui::Ui, home: bool) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    egui::Panel::top("mobile_top").resizable(false).frame(egui::Frame::NONE.fill(t.chrome).inner_margin(egui::Margin::symmetric(6, 2))).show(ui, |ui| {
        ui.allocate_ui_with_layout(vec2(ui.available_width(), TOP_BAR), Layout::left_to_right(Align::Center), |ui| {
            if home {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("PhotoCraft").font(crate::theme::semibold(17.0)).color(t.text));
                return;
            }
            if icons::button(ui, "x", TOUCH_TARGET, false, tl!("Close")).clicked() {
                run(app, &ctx, "file.close");
            }
            if icons::button(ui, "undo-2", TOUCH_TARGET, false, tl!("Undo")).clicked() {
                run(app, &ctx, "edit.undo");
            }
            if icons::button(ui, "redo-2", TOUCH_TARGET, false, tl!("Redo")).clicked() {
                run(app, &ctx, "edit.redo");
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let export = egui::Button::new(egui::RichText::new(tl!("Export")).color(t.primary_text).strong())
                    .fill(t.accent)
                    .corner_radius(18.0)
                    .min_size(vec2(88.0, 36.0));
                if ui.add(export).clicked() {
                    run(app, &ctx, "file.export.exportAs");
                }
                let more = icons::button(ui, "ellipsis", TOUCH_TARGET, false, tl!("More"));
                egui::Popup::menu(&more).show(|ui| {
                    for (label, id) in
                        [(tl!("New…"), "file.new"), (tl!("Open…"), "file.open"), (tl!("Save As…"), "file.saveAs"), (tl!("Fit on Screen"), "view.fitOnScreen")]
                    {
                        if ui.add_sized([180.0, TOUCH_TARGET], egui::Button::new(label)).clicked() {
                            run(app, &ctx, id);
                            ui.close();
                        }
                    }
                    if ui.add_sized([180.0, TOUCH_TARGET], egui::Button::new(tl!("Tool Options")).selected(app.ui.mobile.options_open)).clicked() {
                        app.ui.mobile.options_open = !app.ui.mobile.options_open;
                        ui.close();
                    }
                });
                if icons::button(ui, "layers", TOUCH_TARGET, app.ui.mobile.layers_open, tl!("Layers")).clicked() {
                    app.ui.mobile.layers_open = !app.ui.mobile.layers_open;
                }
            });
        });
    });
    toast(app, &ctx);
}

/// The status line as a toast under the top bar that fades after a few seconds (an error stays
/// a little longer), instead of a line that keeps its height for good.
fn toast(app: &PhotocraftApp, ctx: &egui::Context) {
    let status = &app.ui.status;
    if status.is_empty() {
        return;
    }
    let id = egui::Id::unique("photocraft_mobile_toast");
    let now = ctx.input(|i| i.time);
    let (last, since): (String, f64) = ctx.data(|d| d.get_temp(id)).unwrap_or_default();
    let since = if *status == last {
        since
    } else {
        ctx.data_mut(|d| d.insert_temp(id, (status.clone(), now)));
        now
    };
    let left = toast_seconds(app.ui.status_error) - (now - since);
    if left.is_nan() || left <= 0.0 {
        return;
    }
    ctx.request_repaint_after(std::time::Duration::from_secs_f64(left));
    let t = Tokens::get(ctx);
    let width = (ctx.content_rect().width() - 48.0).max(TOUCH_TARGET);
    egui::Area::new(id.with("area")).order(egui::Order::Foreground).interactable(false).anchor(egui::Align2::CENTER_TOP, vec2(0.0, TOP_BAR + 14.0)).show(
        ctx,
        |ui| {
            let fill = egui::Color32::from_black_alpha(225);
            egui::Frame::NONE.fill(fill).corner_radius(18).inner_margin(egui::Margin::symmetric(14, 8)).show(ui, |ui| {
                ui.set_max_width(width);
                let color = if app.ui.status_error { t.danger } else { egui::Color32::WHITE };
                ui.label(egui::RichText::new(status).color(color));
            });
        },
    );
}

/// How long a toast shows, in seconds.
fn toast_seconds(error: bool) -> f64 {
    if error { 5.0 } else { 2.5 }
}

/// The category tabs along the bottom edge.
fn tabs(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::bottom("mobile_tabs").resizable(false).exact_size(TAB_HEIGHT).frame(egui::Frame::NONE.fill(t.chrome)).show(ui, |ui| {
        let width = (ui.available_width() / MobileTab::ALL.len() as f32).max(TOUCH_TARGET);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for tab in MobileTab::ALL {
                if tile(ui, vec2(width, TAB_HEIGHT), tab.icon(), tl!(tab.label()), app.ui.mobile.tab == tab).clicked() {
                    app.ui.mobile.tab = tab;
                }
            }
        });
    });
}

/// The selected category's tools and actions, scrolling sideways.
fn tools(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    let frame = egui::Frame::NONE.fill(t.card).inner_margin(egui::Margin::symmetric(4, 4));
    egui::Panel::bottom("mobile_tools").resizable(false).frame(frame).show(ui, |ui| {
        egui::ScrollArea::horizontal().id_salt(("mobile_tools_scroll", app.ui.mobile.tab)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for item in app.ui.mobile.tab.items() {
                    let selected = matches!(item, Item::Tool(tool) if app.ui.tool == *tool);
                    if tile(ui, TILE, item.icon(), item.label(), selected).clicked() {
                        match *item {
                            Item::Tool(tool) => crate::tool_memory::select(app, tool),
                            Item::Command(id, _, _) => run(app, &ctx, id),
                        }
                    }
                }
            });
        });
    });
}

/// Tools whose main settings are the brush's size, opacity and hardness.
fn brush_tool(tool: Tool) -> bool {
    crate::canvas::paint_mouse::has_brush_picker(tool) && tool != Tool::QuickSelection
}

/// The active tool's context above the tools: the brush slider, the crop's Cancel and Crop, or
/// the way to the full tool options.
fn context(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let tool = app.ui.tool;
    let crop = tool == Tool::Crop && crate::crop_ui::pending(app);
    if !brush_tool(tool) && !crop && MobileTab::of(tool) != Some(app.ui.mobile.tab) {
        return;
    }
    let frame = egui::Frame::NONE.fill(t.card).inner_margin(egui::Margin { left: 14, right: 14, top: 8, bottom: 2 });
    egui::Panel::bottom("mobile_context").resizable(false).frame(frame).show(ui, |ui| {
        if brush_tool(tool) {
            brush_slider(app, ui);
        } else {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(crate::i18n::t(tool.label())).color(t.text_dim));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if crop {
                        if chip(ui, tl!("Crop"), true).clicked() {
                            crate::canvas::commit_crop(app);
                        }
                        if chip(ui, tl!("Cancel"), false).clicked() {
                            crate::crop_ui::dismiss(app);
                        }
                    }
                    if chip(ui, tl!("Tool Options"), app.ui.mobile.options_open).clicked() {
                        app.ui.mobile.options_open = !app.ui.mobile.options_open;
                    }
                });
            });
        }
    });
}

/// One large slider for the chosen brush setting, with chips to choose it. Edits go through
/// `tools.setBrush`, one journal entry per drag (Rule 1).
fn brush_slider(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    // The tool's own brush, as the options bar would show it.
    crate::canvas::paint_mouse::sync_tool_brush(app);
    let tool = app.ui.tool;
    if !app.ui.mobile.setting.applies(tool) {
        app.ui.mobile.setting = BrushSetting::Size;
    }
    let setting = app.ui.mobile.setting;
    let before = app.session.tools.brush.clone();
    let mut brush = before.clone();
    let mut value = setting.get(&brush);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(tl!(setting.label())).color(t.text_dim));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let shown = if setting == BrushSetting::Size { format!("{value:.0} px") } else { format!("{value:.0}%") };
            ui.label(egui::RichText::new(shown).color(t.text).strong());
        });
    });
    ui.scope(|ui| {
        let width = ui.available_width().max(TOUCH_TARGET);
        let style = ui.spacing_mut();
        style.slider_width = width;
        style.interact_size.y = 36.0;
        let slider = egui::Slider::new(&mut value, setting.range())
            .show_value(false)
            .logarithmic(setting == BrushSetting::Size)
            .handle_shape(egui::style::HandleShape::Circle);
        ui.add(slider);
    });
    setting.set(&mut brush, value);
    crate::brush_panel::commit_gesture(app, ui.ctx(), &before, &brush);
    ui.horizontal(|ui| {
        for s in BrushSetting::ALL {
            if s.applies(tool) && chip(ui, tl!(s.label()), s == setting).clicked() {
                app.ui.mobile.setting = s;
            }
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if icons::button(ui, "settings", 36.0, app.ui.mobile.options_open, tl!("Tool Options")).clicked() {
                app.ui.mobile.options_open = !app.ui.mobile.options_open;
            }
        });
    });
}

pub(crate) fn chrome(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let home = app.home_screen_up();
    top_bar(app, ui, home);
    if home {
        return;
    }
    let t = Tokens::get(ui.ctx());
    // Bottom panels stack from the screen edge up, in the order they are added.
    tabs(app, ui);
    tools(app, ui);
    context(app, ui);
    if app.ui.mobile.layers_open {
        let height = layer_sheet_height(ui.available_height());
        egui::Panel::bottom("mobile_layers")
            .exact_size(height)
            .resizable(false)
            .frame(egui::Frame::NONE.fill(t.chrome).stroke(egui::Stroke::new(1.0, t.separator)))
            .show(ui, |ui| layer_sheet(app, ui));
    }
}

/// The Layers panel with rows a finger can hit: large thumbnails and taller controls. The
/// thumbnail size is the sheet's own; the desktop panel keeps its preference.
fn layer_sheet(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    use crate::state::LayerThumbnailSize;
    let saved = app.ui.layers_panel_options.thumbnail_size;
    app.ui.layers_panel_options.thumbnail_size = LayerThumbnailSize::Large;
    ui.spacing_mut().interact_size.y = 40.0;
    ui.spacing_mut().item_spacing.y = 6.0;
    super::layers::layers(app, ui);
    app.ui.layers_panel_options.thumbnail_size = saved;
}

/// Whether the phone layout is on: the Android host, or the mobile browser layout.
pub fn active(app: &PhotocraftApp) -> bool {
    cfg!(target_os = "android") || app.ui.mobile.enabled
}

/// Whether a dialog opens as a bottom sheet in the phone layout: the dialogs that preview on the
/// canvas (adjustments and filters), so the image stays visible above them.
pub(crate) fn sheet(app: &PhotocraftApp, d: &crate::state::Dialog) -> bool {
    active(app) && d.kind == crate::state::DialogKind::Command && d.fields.contains_key("__preview")
}

/// Before and after for a sheet: while held, the canvas shows the image without the dialog's
/// effect (its Preview off); on release, Preview is back as it was.
pub(crate) fn compare_button(ui: &mut egui::Ui, fields: &mut serde_json::Map<String, serde_json::Value>, id: egui::Id) {
    let held_id = id.with("photocraft_mobile_compare");
    let held: Option<bool> = ui.ctx().data(|d| d.get_temp(held_id));
    let response = icons::button(ui, "eye", 40.0, held.is_some(), tl!("Show Original"));
    let preview = fields.get("__preview").and_then(serde_json::Value::as_bool);
    let (held, set) = compare_step(response.is_pointer_button_down_on(), held, preview);
    ui.ctx().data_mut(|d| {
        if let Some(before) = held {
            d.insert_temp(held_id, before);
        } else {
            d.remove::<bool>(held_id);
        }
    });
    if let Some(on) = set {
        fields.insert("__preview".into(), json!(on));
    }
}

/// One frame of the compare button: whether it is `down`, the Preview it found when the hold
/// began (`held`), and the Preview now. Returns the hold to keep and the Preview to set, if any.
fn compare_step(down: bool, held: Option<bool>, preview: Option<bool>) -> (Option<bool>, Option<bool>) {
    match (down, held) {
        (true, None) => (Some(preview.unwrap_or(true)), Some(false)),
        (true, Some(before)) => (Some(before), None),
        (false, Some(before)) => (None, Some(before)),
        (false, None) => (None, None),
    }
}

/// The Home screen in the phone layout: the app's mark and two large actions, Open and New, then
/// the recent files where there are paths to reopen (not on the web).
pub(crate) fn home(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let area = ui.available_rect_before_wrap();
    let width = (area.width() - 48.0).clamp(0.0, 360.0);
    let column = Rect::from_min_size(egui::pos2(area.center().x - width / 2.0, area.top() + (area.height() * 0.14).min(120.0)), vec2(width, area.height()));
    ui.scope_builder(egui::UiBuilder::new().max_rect(column).layout(Layout::top_down(Align::Center)), |ui| {
        let (mark, _) = ui.allocate_exact_size(Vec2::splat(96.0), Sense::hover());
        super::brand::paint_mark(ui, mark);
        ui.add_space(14.0);
        ui.label(egui::RichText::new("PhotoCraft").font(crate::theme::semibold(26.0)).color(t.text));
        ui.add_space(32.0);
        if big_button(ui, width, "folder-open", tl!("Open…"), true).clicked()
            && let Err(error) = app.open_dialog_file()
            && error != crate::file_dialog::CANCELLED
        {
            app.ui.status = error;
        }
        ui.add_space(12.0);
        if big_button(ui, width, "file-plus", tl!("New file"), false).clicked() {
            let fields = app.new_document_fields();
            app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
        }
        let recent: Vec<String> = if cfg!(target_arch = "wasm32") || !app.session.prefs().interface.show_home_recent_files {
            Vec::new()
        } else {
            app.ui.recent_files.iter().take(6).cloned().collect()
        };
        if !recent.is_empty() {
            ui.add_space(28.0);
            egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| crate::home_recent_ui::show_home_recent(app, ui, &recent, width));
        }
    });
}

/// A full-width button with an icon, the height of a fingertip; `primary` fills it with the accent.
fn big_button(ui: &mut egui::Ui, width: f32, icon: &str, label: &str, primary: bool) -> Response {
    let t = Tokens::get(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(vec2(width, 52.0), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    if ui.is_rect_visible(rect) {
        let (fill, text) = if primary { (t.accent, t.primary_text) } else { (t.field, t.text) };
        let fill = if response.is_pointer_button_down_on() { fill.gamma_multiply(0.85) } else { fill };
        ui.painter().rect_filled(rect, 14.0, fill);
        let galley = ui.painter().layout_no_wrap(label.to_owned(), crate::theme::semibold(15.0), text);
        let icon_size = 22.0;
        let start = rect.center().x - (icon_size + 10.0 + galley.size().x) / 2.0;
        icons::paint(ui, Rect::from_center_size(egui::pos2(start + icon_size / 2.0, rect.center().y), Vec2::splat(icon_size)), icon, icon_size, text);
        ui.painter().galley(egui::pos2(start + icon_size + 10.0, rect.center().y - galley.size().y / 2.0), galley, text);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tab_has_items_and_their_tools_find_their_tab() {
        for tab in MobileTab::ALL {
            assert!(!tab.items().is_empty(), "{tab:?} is empty");
            for item in tab.items() {
                if let Item::Tool(tool) = item {
                    assert!(MobileTab::of(*tool).is_some());
                }
                assert!(icons::exists(item.icon()), "{tab:?}: icon {:?} is missing", item.icon());
            }
            assert!(icons::exists(tab.icon()));
        }
    }

    #[test]
    fn command_tiles_name_commands_in_the_menu_catalog() {
        let ids: Vec<&str> = crate::menu_catalog::CATALOG.iter().map(|row| row.3).collect();
        for tab in MobileTab::ALL {
            for item in tab.items() {
                if let Item::Command(id, _, _) = item {
                    assert!(ids.contains(id), "{id} is not a menu command");
                }
            }
        }
    }

    #[test]
    fn holding_compare_hides_the_preview_and_releasing_restores_it() {
        // Press: remember Preview on, turn it off.
        let (held, set) = compare_step(true, None, Some(true));
        assert_eq!((held, set), (Some(true), Some(false)));
        // Still held: nothing changes.
        assert_eq!(compare_step(true, held, Some(false)), (Some(true), None));
        // Release: back to what it was.
        assert_eq!(compare_step(false, held, Some(false)), (None, Some(true)));
        // A Preview the user had off stays off after the hold.
        let (held, _) = compare_step(true, None, Some(false));
        assert_eq!(compare_step(false, held, Some(false)), (None, Some(false)));
        assert_eq!(compare_step(false, None, Some(true)), (None, None));
    }

    #[test]
    fn toasts_fade_and_errors_stay_longer() {
        assert!(toast_seconds(true) > toast_seconds(false));
        assert!(toast_seconds(false) > 0.0);
    }

    #[test]
    fn tool_tiles_use_the_short_name() {
        assert_eq!(short_tool_label("Brush Tool"), "Brush");
        assert_eq!(short_tool_label("Brush"), "Brush");
    }

    #[test]
    fn brush_settings_round_trip_and_clamp() {
        let mut b = photocraft_paint::BrushSettings::default();
        BrushSetting::Opacity.set(&mut b, 40.0);
        assert!((b.opacity - 0.4).abs() < 1e-6);
        assert_eq!(BrushSetting::Opacity.get(&b), 40.0);
        BrushSetting::Size.set(&mut b, 1.0e9);
        assert_eq!(b.size, MAX_SIZE);
        BrushSetting::Hardness.set(&mut b, f32::NAN);
        assert_eq!(b.hardness, 0.01);
        b.flow = f32::INFINITY;
        assert_eq!(BrushSetting::Flow.get(&b), 1.0);
    }

    #[test]
    fn older_mobile_state_loads_with_the_new_fields_at_their_defaults() {
        let old: MobileUi = serde_json::from_value(json!({"enabled": true, "layers_open": false, "options_open": true})).unwrap();
        assert_eq!(old.tab, MobileTab::Draw);
        assert_eq!(old.setting, BrushSetting::Size);
        assert!(old.options_open);
    }
}
