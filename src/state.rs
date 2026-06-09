use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use egui::{Context, FontData, FontDefinitions, FontFamily, FontId, FontTweak, TextStyle};

use crate::catalog::{catalog, FontCategory, FontEntry, FontVariant};

type SharedResult = Arc<Mutex<Option<Result<Vec<u8>, String>>>>;

/// Persistable subset of FontBookState.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct SavedState {
    pub applied_prop: Option<String>,
    pub applied_mono: Option<String>,
    pub global_font_size: f32,
    pub font_tweaks: HashMap<String, FontTweakSerde>,
    pub font_size_offsets: HashMap<String, f32>,
    pub presets: Vec<Option<FontPreset>>,
    pub sample_text_1: String,
    pub sample_text_2: String,
    pub active_category: FontCategory,
    pub selected_mono: usize,
    pub selected_prop: usize,
    /// Maps font name -> active variant index.
    pub active_variants: HashMap<String, usize>,
}

/// Serde-friendly mirror of egui::FontTweak.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct FontTweakSerde {
    pub scale: f32,
    pub y_offset: f32,
    pub y_offset_factor: f32,
}

impl From<FontTweak> for FontTweakSerde {
    fn from(t: FontTweak) -> Self {
        Self {
            scale: t.scale,
            y_offset: t.y_offset,
            y_offset_factor: t.y_offset_factor,
        }
    }
}

impl From<FontTweakSerde> for FontTweak {
    fn from(t: FontTweakSerde) -> Self {
        FontTweak {
            scale: t.scale,
            y_offset: t.y_offset,
            y_offset_factor: t.y_offset_factor,
            ..Default::default()
        }
    }
}

/// A saved preset: a snapshot of the current font configuration.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct FontPreset {
    pub applied_prop: Option<String>,
    pub applied_mono: Option<String>,
    pub global_font_size: f32,
    pub font_tweaks: HashMap<String, FontTweakSerde>,
    pub font_size_offsets: HashMap<String, f32>,
    pub active_variants: HashMap<String, usize>,
}

pub struct FontBookState {
    // ── Catalog ──
    pub(crate) catalog: Vec<FontEntry>,
    pub(crate) mono_indices: Vec<usize>,
    pub(crate) prop_indices: Vec<usize>,

    // ── Selection ──
    pub(crate) active_category: FontCategory,
    pub(crate) selected_mono: usize,
    pub(crate) selected_prop: usize,

    // ── Downloads ──
    /// Font data keyed by cache_key (variant-specific).
    pub(crate) font_bytes: HashMap<String, Vec<u8>>,
    pub(crate) pending: HashMap<String, SharedResult>,
    pub(crate) errors: HashMap<String, String>,

    // ── Configuration ──
    pub sample_text_1: String,
    pub sample_text_2: String,
    pub global_font_size: f32,
    pub font_tweaks: HashMap<String, FontTweak>,
    pub font_size_offsets: HashMap<String, f32>,

    // ── Variants ──
    /// Maps font name -> active variant index.
    pub(crate) active_variants: HashMap<String, usize>,

    // ── Presets ──
    /// Up to 9 saved presets (slots 1–9).
    pub presets: Vec<Option<FontPreset>>,

    // ── Computed ──
    /// Glyph count per cache_key, computed once on load.
    pub(crate) glyph_counts: HashMap<String, u16>,

    // ── Internal ──
    pub(crate) fonts_dirty: bool,
    pub(crate) initialized: bool,
    pub(crate) renderable: HashSet<String>,
    pub(crate) pending_renderable: HashSet<String>,
    pub(crate) applied_prop: Option<String>,
    pub(crate) applied_mono: Option<String>,
    /// Set to the category whose selection changed this frame (for scroll_to).
    pub(crate) selection_changed: Option<FontCategory>,
}

impl Default for FontBookState {
    fn default() -> Self {
        Self::new()
    }
}

impl FontBookState {
    pub fn new() -> Self {
        let catalog = catalog();
        let mono_indices: Vec<usize> = catalog
            .iter()
            .enumerate()
            .filter(|(_, e)| e.category == FontCategory::Monospace)
            .map(|(i, _)| i)
            .collect();
        let prop_indices: Vec<usize> = catalog
            .iter()
            .enumerate()
            .filter(|(_, e)| e.category == FontCategory::Proportional)
            .map(|(i, _)| i)
            .collect();

        Self {
            catalog,
            mono_indices,
            prop_indices,
            active_category: FontCategory::Proportional,
            selected_mono: 0,
            selected_prop: 0,
            font_bytes: HashMap::new(),
            pending: HashMap::new(),
            errors: HashMap::new(),
            sample_text_1: "Brane is generating a dashboard".into(),
            sample_text_2: "NEXT | PREVIOUS | ACCEPT | CANCEL".into(),
            global_font_size: 14.0,
            font_tweaks: HashMap::new(),
            font_size_offsets: HashMap::new(),
            active_variants: HashMap::new(),
            presets: vec![None; 9],
            glyph_counts: HashMap::new(),
            fonts_dirty: false,
            initialized: false,
            renderable: HashSet::new(),
            pending_renderable: HashSet::new(),
            applied_prop: None,
            applied_mono: None,
            selection_changed: None,
        }
    }

    /// Rebuild the category index vectors after modifying the catalog.
    fn rebuild_indices(&mut self) {
        self.mono_indices = self
            .catalog
            .iter()
            .enumerate()
            .filter(|(_, e)| e.category == FontCategory::Monospace)
            .map(|(i, _)| i)
            .collect();
        self.prop_indices = self
            .catalog
            .iter()
            .enumerate()
            .filter(|(_, e)| e.category == FontCategory::Proportional)
            .map(|(i, _)| i)
            .collect();
    }

    /// Add a custom font to the catalog.
    ///
    /// ```no_run
    /// use egui_font_book::{FontBookState, FontCategory};
    ///
    /// let mut state = FontBookState::new();
    /// state.add_font("Berkeley Mono", FontCategory::Monospace, "https://example.com/BerkeleyMono-Regular.ttf")
    ///     .variant("Light", "https://example.com/BerkeleyMono-Light.ttf")
    ///     .variant("Bold", "https://example.com/BerkeleyMono-Bold.ttf");
    /// ```
    pub fn add_font(
        &mut self,
        name: &str,
        category: FontCategory,
        url: &str,
    ) -> FontEntryBuilder<'_> {
        use std::borrow::Cow;
        let cache_key = name.to_string();
        self.catalog.push(FontEntry {
            name: Cow::Owned(name.to_string()),
            category,
            variants: vec![FontVariant {
                label: Cow::Borrowed("Regular"),
                url: Cow::Owned(url.to_string()),
                cache_key: Cow::Owned(cache_key),
            }],
        });
        self.rebuild_indices();
        let idx = self.catalog.len() - 1;
        FontEntryBuilder {
            state: self,
            index: idx,
        }
    }

    /// Get the active variant index for a font.
    pub fn active_variant(&self, name: &str) -> usize {
        self.active_variants.get(name).copied().unwrap_or(0)
    }

    /// Get the cache key for the currently active variant of a font.
    pub fn active_cache_key(&self, name: &str) -> Option<&str> {
        let entry = self.catalog.iter().find(|e| e.name == name)?;
        let idx = self.active_variant(name);
        entry.variants.get(idx).map(|v| &*v.cache_key)
    }

    pub fn tweak_for(&self, name: &str) -> FontTweak {
        self.font_tweaks.get(name).cloned().unwrap_or_default()
    }

    pub fn tweak_for_mut(&mut self, name: &str) -> &mut FontTweak {
        self.font_tweaks.entry(name.to_string()).or_default()
    }

    pub fn size_offset_for(&self, name: &str) -> f32 {
        self.font_size_offsets.get(name).copied().unwrap_or(0.0)
    }

    /// Returns true when fonts have been modified and need to be reapplied.
    pub fn is_dirty(&self) -> bool {
        self.fonts_dirty
    }

    pub fn effective_size(&self, name: &str) -> f32 {
        (self.global_font_size + self.size_offset_for(name)).max(6.0)
    }

    pub fn reset_tweak(&mut self, name: &str) {
        self.font_tweaks.remove(name);
        self.font_size_offsets.remove(name);
        self.fonts_dirty = true;
    }

    /// Count glyphs in a font file using ttf-parser.
    fn count_glyphs(bytes: &[u8]) -> Option<u16> {
        ttf_parser::Face::parse(bytes, 0)
            .ok()
            .map(|f| f.number_of_glyphs())
    }

    /// Check if a font's active variant is downloaded.
    pub fn is_downloaded(&self, name: &str) -> bool {
        self.active_cache_key(name)
            .map(|ck| self.font_bytes.contains_key(ck))
            .unwrap_or(false)
    }

    /// Check if a font's active variant is renderable.
    pub fn is_renderable(&self, name: &str) -> bool {
        self.active_cache_key(name)
            .map(|ck| self.renderable.contains(ck))
            .unwrap_or(false)
    }

    /// Check if a font's active variant is downloading.
    pub fn is_downloading(&self, name: &str) -> bool {
        self.active_cache_key(name)
            .map(|ck| self.pending.contains_key(ck))
            .unwrap_or(false)
    }

    /// Check if a specific variant cache key is downloaded.
    pub fn is_variant_downloaded(&self, cache_key: &str) -> bool {
        self.font_bytes.contains_key(cache_key)
    }

    pub fn load_cache(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(dir) = cache_dir() {
                for entry in &self.catalog {
                    for variant in &entry.variants {
                        let path = dir.join(format!("{}.ttf", variant.cache_key));
                        if let Ok(data) = std::fs::read(&path) {
                            log::info!("Loaded cached font: {} ({})", entry.name, variant.label);
                            if let Some(count) = Self::count_glyphs(&data) {
                                self.glyph_counts
                                    .insert(variant.cache_key.to_string(), count);
                            }
                            self.font_bytes.insert(variant.cache_key.to_string(), data);
                        }
                    }
                }
                if !self.font_bytes.is_empty() {
                    self.fonts_dirty = true;
                }
            }
        }
    }

    pub fn auto_select_cached(&mut self) {
        if self.applied_prop.is_none() {
            for &idx in &self.prop_indices {
                let name: &str = &self.catalog[idx].name;
                if self.is_downloaded(name) {
                    self.applied_prop = Some(name.to_string());
                    break;
                }
            }
        }
        if self.applied_mono.is_none() {
            for &idx in &self.mono_indices {
                let name: &str = &self.catalog[idx].name;
                if self.is_downloaded(name) {
                    self.applied_mono = Some(name.to_string());
                    break;
                }
            }
        }
    }

    /// Request download of the active variant of a font.
    pub fn request_download(&mut self, name: &str, ctx: &Context) {
        let Some(entry) = self.catalog.iter().find(|e| e.name == name) else {
            return;
        };
        let var_idx = self.active_variant(name);
        let Some(variant) = entry.variants.get(var_idx) else {
            return;
        };
        let cache_key = variant.cache_key.to_string();
        if self.font_bytes.contains_key(&cache_key) || self.pending.contains_key(&cache_key) {
            return;
        }
        let url = variant.url.to_string();
        #[allow(unused_variables)]
        let ck = cache_key.clone();
        let result: SharedResult = Arc::new(Mutex::new(None));
        let result_clone = result.clone();
        let ctx = ctx.clone();

        log::info!(
            "Downloading font: {} ({}) from {}",
            name,
            variant.label,
            url
        );
        let request = ehttp::Request::get(&url);
        ehttp::fetch(request, move |response| {
            let data = match response {
                Ok(resp) if resp.ok => Ok(resp.bytes),
                Ok(resp) => Err(format!("HTTP {}: {}", resp.status, resp.status_text)),
                Err(e) => Err(e),
            };
            #[cfg(not(target_arch = "wasm32"))]
            if let Ok(ref bytes) = data {
                save_to_cache(&ck, bytes);
            }
            *result_clone.lock().unwrap() = Some(data);
            ctx.request_repaint();
        });

        self.pending.insert(cache_key, result);
    }

    /// Request download of a specific variant by cache_key.
    pub fn request_download_variant(&mut self, name: &str, variant_idx: usize, ctx: &Context) {
        let Some(entry) = self.catalog.iter().find(|e| e.name == name) else {
            return;
        };
        let Some(variant) = entry.variants.get(variant_idx) else {
            return;
        };
        let cache_key = variant.cache_key.to_string();
        if self.font_bytes.contains_key(&cache_key) || self.pending.contains_key(&cache_key) {
            // Already have it, just switch
            self.active_variants.insert(name.to_string(), variant_idx);
            self.fonts_dirty = true;
            return;
        }
        let url = variant.url.to_string();
        #[allow(unused_variables)]
        let ck = cache_key.clone();
        let result: SharedResult = Arc::new(Mutex::new(None));
        let result_clone = result.clone();
        let ctx = ctx.clone();

        log::info!(
            "Downloading variant: {} ({}) from {}",
            name,
            variant.label,
            url
        );
        let request = ehttp::Request::get(&url);
        ehttp::fetch(request, move |response| {
            let data = match response {
                Ok(resp) if resp.ok => Ok(resp.bytes),
                Ok(resp) => Err(format!("HTTP {}: {}", resp.status, resp.status_text)),
                Err(e) => Err(e),
            };
            #[cfg(not(target_arch = "wasm32"))]
            if let Ok(ref bytes) = data {
                save_to_cache(&ck, bytes);
            }
            *result_clone.lock().unwrap() = Some(data);
            ctx.request_repaint();
        });

        self.active_variants.insert(name.to_string(), variant_idx);
        self.pending.insert(cache_key, result);
    }

    pub fn poll_downloads(&mut self) {
        let done: Vec<String> = self
            .pending
            .iter()
            .filter_map(|(name, shared)| {
                let lock = shared.lock().unwrap();
                if lock.is_some() {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        for cache_key in done {
            let shared = self.pending.remove(&cache_key).unwrap();
            let result = shared.lock().unwrap().take().unwrap();
            match result {
                Ok(bytes) => {
                    log::info!("Downloaded font: {} ({} bytes)", cache_key, bytes.len());
                    if let Some(count) = Self::count_glyphs(&bytes) {
                        self.glyph_counts.insert(cache_key.clone(), count);
                    }
                    self.font_bytes.insert(cache_key, bytes);
                    self.fonts_dirty = true;
                }
                Err(e) => {
                    log::error!("Failed to download font {}: {}", cache_key, e);
                    self.errors.insert(cache_key, e);
                }
            }
        }
    }

    pub fn apply_fonts(&mut self, ctx: &Context) {
        if !self.fonts_dirty {
            return;
        }
        self.fonts_dirty = false;

        self.renderable.extend(self.pending_renderable.drain());

        // Start from the current font definitions so we preserve any families/data
        // the host app has set up (e.g. custom "bold", icon fonts, etc.).
        let mut fonts = ctx.fonts(|f| f.definitions().clone());

        // Ensure a FontBookUI family exists for the browser's own UI text,
        // snapshotted from the current proportional family.
        if !fonts
            .families
            .contains_key(&FontFamily::Name("FontBookUI".into()))
        {
            let default_prop = fonts
                .families
                .get(&FontFamily::Proportional)
                .cloned()
                .unwrap_or_default();
            fonts
                .families
                .insert(FontFamily::Name("FontBookUI".into()), default_prop);
        }

        // Register each font under its name, using the active variant's bytes.
        for entry in &self.catalog {
            let var_idx = self.active_variant(&entry.name);
            let Some(variant) = entry.variants.get(var_idx) else {
                continue;
            };
            let Some(bytes) = self.font_bytes.get(&*variant.cache_key) else {
                continue;
            };
            let tweak = self.tweak_for(&entry.name);
            let font_name = entry.name.to_string();
            fonts.font_data.insert(
                font_name.clone(),
                Arc::new(FontData::from_owned(bytes.clone()).tweak(tweak)),
            );
            fonts.families.insert(
                FontFamily::Name(font_name.clone().into()),
                vec![font_name.clone()],
            );
            self.pending_renderable
                .insert(variant.cache_key.to_string());
        }

        if let Some(ref prop_name) = self.applied_prop {
            if fonts.font_data.contains_key(prop_name) {
                if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
                    family.insert(0, prop_name.clone());
                }
            }
        }

        if let Some(ref mono_name) = self.applied_mono {
            if fonts.font_data.contains_key(mono_name) {
                if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
                    family.insert(0, mono_name.clone());
                }
            }
        }

        ctx.set_fonts(fonts);

        // Update global text styles
        let base = self.global_font_size;
        let prop_off = self
            .applied_prop
            .as_deref()
            .map(|n| self.size_offset_for(n))
            .unwrap_or(0.0);
        let mono_off = self
            .applied_mono
            .as_deref()
            .map(|n| self.size_offset_for(n))
            .unwrap_or(0.0);

        let prop_size = (base + prop_off).max(6.0);
        let mono_size = (base + mono_off).max(6.0);

        ctx.global_style_mut(|style| {
            style.text_styles.insert(
                TextStyle::Small,
                FontId::new(prop_size * 0.75, FontFamily::Proportional),
            );
            style.text_styles.insert(
                TextStyle::Body,
                FontId::new(prop_size, FontFamily::Proportional),
            );
            style.text_styles.insert(
                TextStyle::Button,
                FontId::new(prop_size, FontFamily::Proportional),
            );
            style.text_styles.insert(
                TextStyle::Heading,
                FontId::new(prop_size * 1.4, FontFamily::Proportional),
            );
            style.text_styles.insert(
                TextStyle::Monospace,
                FontId::new(mono_size, FontFamily::Monospace),
            );
        });
    }

    pub fn selected_entry_for(&self, category: FontCategory) -> Option<&FontEntry> {
        let indices = match category {
            FontCategory::Monospace => &self.mono_indices,
            FontCategory::Proportional => &self.prop_indices,
        };
        let sel = match category {
            FontCategory::Monospace => self.selected_mono,
            FontCategory::Proportional => self.selected_prop,
        };
        indices.get(sel).map(|&i| &self.catalog[i])
    }

    pub fn selected_entry(&self) -> Option<&FontEntry> {
        self.selected_entry_for(self.active_category)
    }

    pub(crate) fn selected_for_mut(&mut self, category: FontCategory) -> &mut usize {
        match category {
            FontCategory::Monospace => &mut self.selected_mono,
            FontCategory::Proportional => &mut self.selected_prop,
        }
    }

    pub(crate) fn selected_for(&self, category: FontCategory) -> usize {
        match category {
            FontCategory::Monospace => self.selected_mono,
            FontCategory::Proportional => self.selected_prop,
        }
    }

    pub(crate) fn indices_for(&self, category: FontCategory) -> &[usize] {
        match category {
            FontCategory::Monospace => &self.mono_indices,
            FontCategory::Proportional => &self.prop_indices,
        }
    }

    pub fn apply_selected(&mut self) {
        self.apply_selected_for(self.active_category);
    }

    pub fn apply_selected_for(&mut self, category: FontCategory) {
        if let Some(entry) = self.selected_entry_for(category) {
            let name = entry.name.to_string();
            match category {
                FontCategory::Monospace => self.applied_mono = Some(name),
                FontCategory::Proportional => self.applied_prop = Some(name),
            }
            self.fonts_dirty = true;
        }
    }

    pub fn applied_prop_name(&self) -> Option<&str> {
        self.applied_prop.as_deref()
    }

    pub fn applied_mono_name(&self) -> Option<&str> {
        self.applied_mono.as_deref()
    }

    pub fn glyph_count(&self, name: &str) -> Option<u16> {
        let ck = self.active_cache_key(name)?;
        self.glyph_counts.get(ck).copied()
    }

    /// Download all fonts in the catalog that aren't already downloaded or pending.
    pub fn request_download_all(&mut self, ctx: &Context) {
        let names: Vec<String> = self.catalog.iter().map(|e| e.name.to_string()).collect();
        for name in names {
            self.request_download(&name, ctx);
        }
    }

    fn tweaks_to_serde(&self) -> HashMap<String, FontTweakSerde> {
        self.font_tweaks
            .iter()
            .map(|(k, v)| (k.clone(), FontTweakSerde::from(v.clone())))
            .collect()
    }

    fn tweaks_from_serde(m: HashMap<String, FontTweakSerde>) -> HashMap<String, FontTweak> {
        m.into_iter()
            .map(|(k, v)| (k, FontTweak::from(v)))
            .collect()
    }

    /// Save the current font configuration into preset slot (0-indexed).
    pub fn save_preset(&mut self, slot: usize) {
        if slot >= self.presets.len() {
            return;
        }
        self.presets[slot] = Some(FontPreset {
            applied_prop: self.applied_prop.clone(),
            applied_mono: self.applied_mono.clone(),
            global_font_size: self.global_font_size,
            font_tweaks: self.tweaks_to_serde(),
            font_size_offsets: self.font_size_offsets.clone(),
            active_variants: self.active_variants.clone(),
        });
    }

    /// Load a preset from the given slot (0-indexed). Returns false if slot is empty.
    pub fn load_preset(&mut self, slot: usize) -> bool {
        let Some(Some(preset)) = self.presets.get(slot).cloned() else {
            return false;
        };
        self.applied_prop = preset.applied_prop;
        self.applied_mono = preset.applied_mono;
        self.global_font_size = preset.global_font_size;
        self.font_tweaks = Self::tweaks_from_serde(preset.font_tweaks);
        self.font_size_offsets = preset.font_size_offsets;
        self.active_variants = preset.active_variants;
        self.fonts_dirty = true;
        true
    }

    /// Extract persistable state.
    pub fn save_state(&self) -> SavedState {
        SavedState {
            applied_prop: self.applied_prop.clone(),
            applied_mono: self.applied_mono.clone(),
            global_font_size: self.global_font_size,
            font_tweaks: self.tweaks_to_serde(),
            font_size_offsets: self.font_size_offsets.clone(),
            presets: self.presets.clone(),
            sample_text_1: self.sample_text_1.clone(),
            sample_text_2: self.sample_text_2.clone(),
            active_category: self.active_category,
            selected_mono: self.selected_mono,
            selected_prop: self.selected_prop,
            active_variants: self.active_variants.clone(),
        }
    }

    /// Restore state from a saved snapshot.
    pub fn restore_state(&mut self, saved: SavedState) {
        self.applied_prop = saved.applied_prop;
        self.applied_mono = saved.applied_mono;
        self.global_font_size = saved.global_font_size;
        self.font_tweaks = Self::tweaks_from_serde(saved.font_tweaks);
        self.font_size_offsets = saved.font_size_offsets;
        self.presets = saved.presets;
        self.sample_text_1 = saved.sample_text_1;
        self.sample_text_2 = saved.sample_text_2;
        self.active_category = saved.active_category;
        self.selected_mono = saved.selected_mono;
        self.selected_prop = saved.selected_prop;
        self.active_variants = saved.active_variants;
        self.fonts_dirty = true;
    }
}

/// Builder for adding variants to a newly added custom font.
pub struct FontEntryBuilder<'a> {
    state: &'a mut FontBookState,
    index: usize,
}

impl<'a> FontEntryBuilder<'a> {
    /// Add a variant to this font entry.
    pub fn variant(self, label: &str, url: &str) -> Self {
        use std::borrow::Cow;
        let name = &self.state.catalog[self.index].name;
        let cache_key = format!("{}-{}", name, label);
        self.state.catalog[self.index].variants.push(FontVariant {
            label: Cow::Owned(label.to_string()),
            url: Cow::Owned(url.to_string()),
            cache_key: Cow::Owned(cache_key),
        });
        self
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn cache_dir() -> Option<std::path::PathBuf> {
    dirs::cache_dir().map(|d| d.join("egui_font_book").join("fonts"))
}

#[cfg(not(target_arch = "wasm32"))]
fn save_to_cache(cache_key: &str, data: &[u8]) {
    if let Some(dir) = cache_dir() {
        if let Err(e) = std::fs::create_dir_all(&dir) {
            log::warn!("Failed to create cache dir: {}", e);
            return;
        }
        let path = dir.join(format!("{}.ttf", cache_key));
        if let Err(e) = std::fs::write(&path, data) {
            log::warn!("Failed to cache font {}: {}", cache_key, e);
        }
    }
}
