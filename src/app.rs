/// Main application state and UI for FocusFlow

use eframe::egui;
use std::path::PathBuf;

use crate::model::{FocusNode, FocusTree, ValidationResult};
use crate::parser;
use crate::writer;

/// Application view mode
#[derive(Debug, Clone, PartialEq)]
enum AppView {
    /// List view (table of focuses)
    List,
    /// Canvas view (visual node graph)
    Canvas,
}

/// Application state
pub struct FocusFlowApp {
    /// Currently loaded focus tree
    pub tree: Option<FocusTree>,
    
    /// Original tree (for diff)
    pub original_tree: Option<FocusTree>,
    
    /// Path to the loaded file
    pub file_path: Option<PathBuf>,
    
    /// Index of selected focus
    pub selected_focus_idx: Option<usize>,
    
    /// Search filter text
    pub search_filter: String,
    
    /// Category filter
    pub category_filter: String,
    
    /// Validation results
    pub validation: Option<ValidationResult>,
    
    /// Status message
    pub status_message: String,
    
    /// Whether the editor panel is visible
    pub show_editor: bool,
    
    /// Whether the validation panel is visible
    pub show_validation: bool,
    
    /// Current view mode
    pub view_mode: AppView,
    
    // === Editor state ===
    /// Currently editing focus (copy)
    pub editing_focus: Option<FocusNode>,
    
    /// Whether we're creating a new focus
    pub creating_new: bool,
    
    // === Canvas state ===
    /// Canvas zoom level
    pub canvas_zoom: f32,
    
    /// Canvas pan offset
    pub canvas_pan: egui::Vec2,
    
    /// Whether user is panning
    pub is_panning: bool,
    
    // === File dialog state ===
    /// Path typed by user
    pub file_path_input: String,
    
    /// Show save confirmation dialog
    pub show_save_dialog: bool,
    
    /// Show diff preview
    pub show_diff: bool,
    
    /// Undo history
    pub undo_stack: Vec<String>,
    pub redo_stack: Vec<String>,
}

impl FocusFlowApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure Vantary-inspired minimalist style
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 6.0);
        style.visuals.window_fill = egui::Color32::from_rgb(28, 32, 40);
        style.visuals.panel_fill = egui::Color32::from_rgb(32, 37, 45);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(45, 50, 60);
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(20, 24, 30);
        style.visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(16, 185, 129, 200);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(16, 185, 129));
        style.visuals.window_shadow = egui::epaint::Shadow {
            offset: [(0.0) as i8, (4.0) as i8],
            blur: 20.0 as u8,
            spread: 0.0 as u8,
            color: egui::Color32::from_black_alpha(80),
        };
        style.visuals.popup_shadow = egui::epaint::Shadow {
            offset: [(0.0) as i8, (4.0) as i8],
            blur: 16.0 as u8,
            spread: 0.0 as u8,
            color: egui::Color32::from_black_alpha(60),
        };
        cc.egui_ctx.set_style(style);
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        
        Self {
            tree: None,
            original_tree: None,
            file_path: None,
            selected_focus_idx: None,
            search_filter: String::new(),
            category_filter: String::new(),
            validation: None,
            status_message: "FocusFlow — HOI4 Focus Tree Editor".to_string(),
            show_editor: false,
            show_validation: false,
            view_mode: AppView::List,
            editing_focus: None,
            creating_new: false,
            canvas_zoom: 1.0,
            canvas_pan: egui::Vec2::ZERO,
            is_panning: false,
            file_path_input: String::new(),
            show_save_dialog: false,
            show_diff: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
    
    /// Save current tree state for undo
    fn save_undo(&mut self) {
        if let Some(tree) = &self.tree {
            if let Ok(json) = serde_json::to_string(tree) {
                self.undo_stack.push(json);
                if self.undo_stack.len() > 50 {
                    self.undo_stack.remove(0);
                }
                self.redo_stack.clear();
            }
        }
    }
    
    /// Undo last change
    fn undo(&mut self) {
        if let Some(json) = self.undo_stack.pop() {
            if let Some(current) = &self.tree {
                if let Ok(json) = serde_json::to_string(current) {
                    self.redo_stack.push(json);
                }
            }
            if let Ok(tree) = serde_json::from_str(&json) {
                self.tree = Some(tree);
                self.status_message = "↩️ Undo".to_string();
            }
        }
    }
    
    /// Redo last change
    fn redo(&mut self) {
        if let Some(json) = self.redo_stack.pop() {
            if let Some(current) = &self.tree {
                if let Ok(json) = serde_json::to_string(current) {
                    self.undo_stack.push(json);
                }
            }
            if let Ok(tree) = serde_json::from_str(&json) {
                self.tree = Some(tree);
                self.status_message = "↩️ Redo".to_string();
            }
        }
    }
    
    /// Try to load a focus tree from a file path
    pub fn load_file(&mut self, path: &PathBuf) {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                match parser::parse_focus_file(&content) {
                    Ok(tree) => {
                        self.original_tree = Some(tree.clone());
                        self.status_message = format!("✅ Loaded: {} — {} focuses", tree.id, tree.focuses.len());
                        self.tree = Some(tree);
                        self.file_path = Some(path.clone());
                        self.selected_focus_idx = None;
                        self.validation = None;
                        self.file_path_input = path.display().to_string();
                        self.undo_stack.clear();
                        self.redo_stack.clear();
                        self.show_diff = false;
                    }
                    Err(e) => {
                        self.status_message = format!("❌ Parse error: {}", e);
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("❌ File error: {}", e);
            }
        }
    }
    
    /// Save the current tree back to the file
    pub fn save_file(&mut self) {
        if let (Some(tree), Some(path)) = (&self.tree, &self.file_path) {
            // Create backup
            let backup_path = path.with_extension("txt.bak");
            let _ = std::fs::copy(path, &backup_path);
            
            let output = writer::write_focus_tree(tree);
            match std::fs::write(path, output.as_bytes()) {
                Ok(_) => {
                    self.status_message = format!("✅ Saved: {} (backup created)", path.display());
                    self.tree.as_mut().unwrap().modified = false;
                    self.original_tree = self.tree.clone();
                }
                Err(e) => {
                    self.status_message = format!("❌ Save error: {}", e);
                }
            }
        }
    }
    
    /// Run validation on the current tree
    pub fn run_validation(&mut self) {
        if let Some(tree) = &self.tree {
            let result = parser::validate_tree(tree);
            self.status_message = format!(
                "Validation: {} errors, {} warnings",
                result.errors.len(),
                result.warnings.len()
            );
            self.validation = Some(result);
            self.show_validation = true;
        }
    }
    
    /// Open the editor for the selected focus
    pub fn open_editor(&mut self) {
        if let (Some(ref tree), Some(idx)) = (&self.tree, self.selected_focus_idx) {
            if let Some(focus) = tree.focuses.get(idx) {
                self.editing_focus = Some(focus.clone());
                self.creating_new = false;
                self.show_editor = true;
            }
        }
    }
    
    /// Create a new blank focus
    pub fn create_new_focus(&mut self) {
        if let Some(tree) = &self.tree {
            let new_id = format!("VEN_new_focus_{}", tree.focuses.len() + 1);
            self.save_undo();
            self.editing_focus = Some(FocusNode {
                id: new_id,
                icon: Some("generic_industry".to_string()),
                x: 0,
                y: 0,
                relative_position_id: None,
                cost: Some(5.0),
                prerequisites: Vec::new(),
                mutually_exclusive: Vec::new(),
                bypass_if_unavailable: false,
                available_raw: None,
                completion_reward_raw: Some("{\n\t\t\tadd_political_power = 50\n\t\t}".to_string()),
                immediate_raw: None,
                ai_will_do_raw: Some("{\n\t\t\tbase = 10\n\t\t}".to_string()),
                search_filters: vec!["FOCUS_FILTER_POLITICAL".to_string()],
                bypass_raw: None,
            });
            self.creating_new = true;
            self.show_editor = true;
        }
    }
    
    /// Save the edited focus
    pub fn save_edited_focus(&mut self) {
        let edited = self.editing_focus.take();
        let creating_new = self.creating_new;
        let idx = self.selected_focus_idx;
        
        if let Some(edited) = edited {
            let focus_id = edited.id.clone();
            self.save_undo();
            
            if creating_new {
                if let Some(tree) = &mut self.tree {
                    tree.focuses.push(edited);
                    tree.modified = true;
                }
                self.status_message = format!("✅ Created new focus: {}", focus_id);
            } else if let Some(idx) = idx {
                if let Some(tree) = &mut self.tree {
                    if let Some(existing) = tree.focuses.get_mut(idx) {
                        *existing = edited;
                        tree.modified = true;
                    }
                    self.status_message = format!("✅ Updated focus: {}", focus_id);
                }
            }
            self.show_editor = false;
            self.creating_new = false;
        }
    }
    
    /// Delete the selected focus
    pub fn delete_selected_focus(&mut self) {
        let idx = self.selected_focus_idx;
        let focus_id = if let (Some(tree), Some(idx)) = (&self.tree, idx) {
            tree.focuses.get(idx).map(|f| f.id.clone())
        } else {
            None
        };
        
        if let (Some(focus_id), Some(idx)) = (focus_id, idx) {
            self.save_undo();
            if let Some(tree) = &mut self.tree {
                tree.focuses.remove(idx);
                tree.modified = true;
            }
            self.selected_focus_idx = None;
            self.status_message = format!("🗑️ Deleted focus: {}", focus_id);
        }
    }
    
    /// Duplicate the selected focus
    pub fn duplicate_selected_focus(&mut self) {
        let new_focus = if let (Some(tree), Some(idx)) = (&self.tree, self.selected_focus_idx) {
            tree.focuses.get(idx).map(|focus| {
                let mut new_focus = focus.clone();
                new_focus.id = format!("{}_copy", focus.id);
                new_focus.x += 1;
                new_focus
            })
        } else {
            None
        };
        
        if let Some(new_focus) = new_focus {
            let id = new_focus.id.clone();
            self.save_undo();
            if let Some(tree) = &mut self.tree {
                tree.focuses.push(new_focus);
                tree.modified = true;
            }
            self.status_message = format!("📋 Duplicated: {}", id);
        }
    }
    
    /// Get filtered list of focuses
    pub fn filtered_focuses(&self) -> Vec<usize> {
        if let Some(tree) = &self.tree {
            tree.focuses
                .iter()
                .enumerate()
                .filter(|(_, f)| {
                    let matches_search = self.search_filter.is_empty()
                        || f.id.to_lowercase().contains(&self.search_filter.to_lowercase())
                        || f.display_name().to_lowercase().contains(&self.search_filter.to_lowercase());
                    
                    let matches_category = self.category_filter.is_empty()
                        || self.category_filter == "All"
                        || f.category() == self.category_filter;
                    
                    matches_search && matches_category
                })
                .map(|(i, _)| i)
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get focus count by category
    pub fn category_counts(&self) -> Vec<(String, usize)> {
        if let Some(tree) = &self.tree {
            let mut counts = std::collections::HashMap::new();
            for f in &tree.focuses {
                *counts.entry(f.category().to_string()).or_insert(0) += 1;
            }
            let mut result: Vec<_> = counts.into_iter().collect();
            result.sort_by(|a, b| b.1.cmp(&a.1));
            result
        } else {
            Vec::new()
        }
    }
    
    /// Check for keyboard shortcuts
    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            self.save_file();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z)) {
            self.undo();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Y)) {
            self.redo();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            let path = self.file_path.clone();
            if let Some(path) = path {
                self.load_file(&path);
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
            self.delete_selected_focus();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::D)) {
            self.duplicate_selected_focus();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::E)) && !ctx.input(|i| i.modifiers.ctrl) {
            if self.selected_focus_idx.is_some() {
                self.open_editor();
            }
        }
    }
}

impl eframe::App for FocusFlowApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        self.handle_keyboard(ctx);
        
        // Floating file actions (top-left)
        if self.tree.is_none() || true {
            egui::Window::new("")
                .id(egui::Id::new("file_actions"))
                .fixed_pos(egui::pos2(12.0, 12.0))
                .auto_sized()
                .resizable(false)
                .frame(egui::Frame::window(&ctx.style())
                    .fill(egui::Color32::from_rgb(28, 32, 40))
                    .corner_radius(egui::CornerRadius::same(12))
                    .shadow(egui::epaint::Shadow {
                        offset: [(0.0) as i8, (4.0) as i8],
                        blur: 16.0 as u8,
                        spread: 0.0 as u8,
                        color: egui::Color32::from_black_alpha(60),
                    }))
                .title_bar(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.add_sized(egui::vec2(36.0, 36.0), egui::Button::new("📂"))
                            .on_hover_text("Open file (Ctrl+O)").clicked() 
                        {
                            // Open file dialog
                        }
                        if ui.add_sized(egui::vec2(36.0, 36.0), egui::Button::new("💾"))
                            .on_hover_text("Save (Ctrl+S)").clicked() 
                        {
                            self.save_file();
                        }
                        if ui.add_sized(egui::vec2(36.0, 36.0), egui::Button::new("🔄"))
                            .on_hover_text("Reload (F5)").clicked() 
                        {
                            let path = self.file_path.clone();
                            if let Some(path) = path {
                                self.load_file(&path);
                            }
                        }
                        ui.separator();
                        if ui.add_sized(egui::vec2(36.0, 36.0), egui::Button::new("✨"))
                            .on_hover_text("New focus").clicked() 
                        {
                            self.create_new_focus();
                        }
                        if ui.add_sized(egui::vec2(36.0, 36.0), egui::Button::new("✏️"))
                            .on_hover_text("Edit selected (E)").clicked() 
                        {
                            self.open_editor();
                        }
                        if ui.add_sized(egui::vec2(36.0, 36.0), egui::Button::new("🗑️"))
                            .on_hover_text("Delete (Del)").clicked() 
                        {
                            self.delete_selected_focus();
                        }
                    });
                });
        }
        
        // Mission status module (top-right)
        if let Some(tree) = &self.tree {
            let mut status_open = true;
            egui::Window::new("")
                .id(egui::Id::new("mission_status"))
                .fixed_pos(egui::pos2(ctx.screen_rect().right() - 220.0, 12.0))
                .fixed_size(egui::vec2(200.0, 80.0))
                .resizable(false)
                .movable(false)
                .frame(egui::Frame::window(&ctx.style())
                    .fill(egui::Color32::from_rgb(28, 32, 40))
                    .corner_radius(egui::CornerRadius::same(12))
                    .shadow(egui::epaint::Shadow {
                        offset: [(0.0) as i8, (4.0) as i8],
                        blur: 16.0 as u8,
                        spread: 0.0 as u8,
                        color: egui::Color32::from_black_alpha(60),
                    }))
                .title_bar(false)
                .open(&mut status_open)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("FocusFlow").size(16.0).color(egui::Color32::from_rgb(16, 185, 129)));
                        ui.label(egui::RichText::new(&tree.id).size(11.0).color(egui::Color32::from_gray(180)));
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("{} focuses", tree.focuses.len())).size(10.0).color(egui::Color32::from_gray(140)));
                            if tree.modified {
                                ui.label(egui::RichText::new("● Modified").size(10.0).color(egui::Color32::from_rgb(239, 68, 68)));
                            } else {
                                ui.label(egui::RichText::new("✓ Saved").size(10.0).color(egui::Color32::from_rgb(16, 185, 129)));
                            }
                        });
                    });
                });
        }
        
        // Floating toolbar (bottom-center)
        egui::Window::new("")
            .id(egui::Id::new("toolbar"))
            .fixed_pos(egui::pos2(ctx.screen_rect().center().x - 180.0, ctx.screen_rect().bottom() - 60.0))
            .fixed_size(egui::vec2(360.0, 44.0))
            .resizable(false)
            .movable(false)
            .frame(egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgb(28, 32, 40))
                .corner_radius(egui::CornerRadius::same(12))
                .shadow(egui::epaint::Shadow {
                    offset: [(0.0) as i8, (4.0) as i8],
                    blur: 16.0 as u8,
                    spread: 0.0 as u8,
                    color: egui::Color32::from_black_alpha(60),
                }))
            .title_bar(false)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if ui.add_sized(egui::vec2(70.0, 32.0), egui::Button::new("📋 List"))
                        .on_hover_text("List view").clicked() 
                    {
                        self.view_mode = AppView::List;
                    }
                    if ui.add_sized(egui::vec2(70.0, 32.0), egui::Button::new("🌐 Canvas"))
                        .on_hover_text("Canvas view").clicked() 
                    {
                        self.view_mode = AppView::Canvas;
                    }
                    ui.separator();
                    if ui.add_sized(egui::vec2(36.0, 32.0), egui::Button::new("↩️"))
                        .on_hover_text("Undo (Ctrl+Z)").clicked() 
                    {
                        self.undo();
                    }
                    if ui.add_sized(egui::vec2(36.0, 32.0), egui::Button::new("↪️"))
                        .on_hover_text("Redo (Ctrl+Y)").clicked() 
                    {
                        self.redo();
                    }
                    ui.separator();
                    if ui.add_sized(egui::vec2(36.0, 32.0), egui::Button::new("🔍"))
                        .on_hover_text("Validate").clicked() 
                    {
                        self.run_validation();
                    }
                    if ui.add_sized(egui::vec2(36.0, 32.0), egui::Button::new("📊"))
                        .on_hover_text("Diff preview").clicked() 
                    {
                        self.show_diff = !self.show_diff;
                    }
                });
            });
        
        // Top menu bar (minimalist)
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("📂 Open File...").clicked() {
                        // Focus on path input
                    }
                    if ui.button("💾 Save").clicked() {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.button("💾 Save As...").clicked() {
                        self.show_save_dialog = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🔄 Reload").clicked() {
                        let path = self.file_path.clone();
                        if let Some(path) = path {
                            self.load_file(&path);
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
                
                ui.menu_button("Edit", |ui| {
                    if ui.button("✨ New Focus").clicked() {
                        self.create_new_focus();
                        ui.close_menu();
                    }
                    if ui.button("✏️ Edit Selected (E)").clicked() {
                        self.open_editor();
                        ui.close_menu();
                    }
                    if ui.button("📋 Duplicate (Ctrl+D)").clicked() {
                        self.duplicate_selected_focus();
                        ui.close_menu();
                    }
                    if ui.button("🗑️ Delete (Del)").clicked() {
                        self.delete_selected_focus();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("↩️ Undo (Ctrl+Z)").clicked() {
                        self.undo();
                        ui.close_menu();
                    }
                    if ui.button("↩️ Redo (Ctrl+Y)").clicked() {
                        self.redo();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🔍 Run Validation").clicked() {
                        self.run_validation();
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("View", |ui| {
                    if ui.selectable_label(self.view_mode == AppView::List, "📋 List View").clicked() {
                        self.view_mode = AppView::List;
                        ui.close_menu();
                    }
                    if ui.selectable_label(self.view_mode == AppView::Canvas, "🌐 Canvas View").clicked() {
                        self.view_mode = AppView::Canvas;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.checkbox(&mut self.show_editor, "Editor Panel").clicked() {
                        if !self.show_editor {
                            self.show_editor = false;
                        }
                    }
                    if ui.checkbox(&mut self.show_validation, "Validation Panel").clicked() {
                        if !self.show_validation {
                            self.validation = None;
                        }
                    }
                    ui.separator();
                    if ui.button("📊 Diff Preview").clicked() {
                        self.show_diff = !self.show_diff;
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("⌨️ Keyboard Shortcuts").clicked() {
                        self.status_message = "Ctrl+S: Save | Ctrl+Z: Undo | Ctrl+Y: Redo | E: Edit | Del: Delete | Ctrl+D: Duplicate | F5: Reload".to_string();
                        ui.close_menu();
                    }
                });
            });
            ui.add_space(2.0);
        });
        
        // Status bar at bottom
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.separator();
            ui.label(&self.status_message);
        });
        
        // Left panel
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                self.ui_left_panel(ui);
            });
        
        // Center panel depends on view mode
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.view_mode {
                AppView::List => self.ui_list_view(ui),
                AppView::Canvas => self.ui_canvas_view(ui),
            }
        });
        
        // Floating editor panel (right side)
        if self.show_editor {
            let mut editor_open = true;
            egui::Window::new("✏️ Editor de Foco")
                .id(egui::Id::new("editor_panel"))
                .fixed_pos(egui::pos2(ctx.screen_rect().right() - 440.0, 100.0))
                .fixed_size(egui::vec2(420.0, ctx.screen_rect().height() - 180.0))
                .resizable(false)
                .frame(egui::Frame::window(&ctx.style())
                    .fill(egui::Color32::from_rgb(28, 32, 40))
                    .corner_radius(egui::CornerRadius::same(12))
                    .shadow(egui::epaint::Shadow {
                        offset: [(0.0) as i8, (4.0) as i8],
                        blur: 16.0 as u8,
                        spread: 0.0 as u8,
                        color: egui::Color32::from_black_alpha(60),
                    }))
                .open(&mut editor_open)
                .show(ctx, |ui| {
                    self.ui_editor_panel(ui);
                });
            self.show_editor = editor_open;
        }
        
        // Floating validation panel (bottom-right)
        if self.show_validation {
            let mut validation_open = true;
            egui::Window::new("🔍 Validación")
                .id(egui::Id::new("validation_panel"))
                .fixed_pos(egui::pos2(ctx.screen_rect().right() - 380.0, ctx.screen_rect().bottom() - 280.0))
                .fixed_size(egui::vec2(360.0, 260.0))
                .resizable(false)
                .frame(egui::Frame::window(&ctx.style())
                    .fill(egui::Color32::from_rgb(28, 32, 40))
                    .corner_radius(egui::CornerRadius::same(12))
                    .shadow(egui::epaint::Shadow {
                        offset: [(0.0) as i8, (4.0) as i8],
                        blur: 16.0 as u8,
                        spread: 0.0 as u8,
                        color: egui::Color32::from_black_alpha(60),
                    }))
                .open(&mut validation_open)
                .show(ctx, |ui| {
                    self.ui_validation_panel(ui);
                });
            self.show_validation = validation_open;
        }
        
        // Diff preview window (floating)
        if self.show_diff {
            let diff_text = if let (Some(old), Some(new)) = (&self.original_tree, &self.tree) {
                writer::generate_diff(old, new)
            } else {
                "No original version to compare".to_string()
            };
            
            let mut diff_open = true;
            egui::Window::new("📊 Diff Preview")
                .id(egui::Id::new("diff_preview"))
                .fixed_pos(egui::pos2(ctx.screen_rect().center().x - 300.0, ctx.screen_rect().center().y - 200.0))
                .fixed_size(egui::vec2(600.0, 400.0))
                .resizable(false)
                .frame(egui::Frame::window(&ctx.style())
                    .fill(egui::Color32::from_rgb(28, 32, 40))
                    .corner_radius(egui::CornerRadius::same(12))
                    .shadow(egui::epaint::Shadow {
                        offset: [(0.0) as i8, (4.0) as i8],
                        blur: 16.0 as u8,
                        spread: 0.0 as u8,
                        color: egui::Color32::from_black_alpha(60),
                    }))
                .open(&mut diff_open)
                .show(ctx, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut diff_text.clone()).code_editor().desired_width(f32::INFINITY));
                });
            self.show_diff = diff_open;
        }
    }
}

impl FocusFlowApp {
    /// Left panel: file load, search, focus list
    fn ui_left_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("FocusFlow");
        ui.separator();
        
        // File path input
        ui.horizontal(|ui| {
            ui.label("📂");
            if ui.text_edit_singleline(&mut self.file_path_input).lost_focus() 
                && ui.input(|i| i.key_pressed(egui::Key::Enter)) 
            {
                let path = PathBuf::from(&self.file_path_input);
                if path.exists() {
                    self.load_file(&path);
                }
            }
            if ui.button("Load").clicked() {
                let path = PathBuf::from(&self.file_path_input);
                if path.exists() {
                    self.load_file(&path);
                }
            }
        });
        
        // Quick load button
        let md_path = r"C:\Users\armon\Documents\Paradox Interactive\Hearts of Iron IV\mod\MD\common\national_focus\venezuela.txt";
        if ui.button("🇻🇪 Load Venezuela").clicked() {
            let path = PathBuf::from(md_path);
            if path.exists() {
                self.load_file(&path);
            }
        }
        
        // Also show Colombia and Brazil if they exist
        for (country, flag) in &[("colombia.txt", "🇨🇴"), ("brazil.txt", "🇧🇷")] {
            let path_str = md_path.replace("venezuela.txt", country);
            if ui.button(format!("{} Load {}", flag, country.strip_suffix(".txt").unwrap())).clicked() {
                let path = PathBuf::from(&path_str);
                if path.exists() {
                    self.load_file(&path);
                }
            }
        }
        
        ui.separator();
        
        if self.tree.is_none() {
            ui.label("No focus tree loaded");
            ui.label("");
            ui.label("Enter a file path or click");
            ui.label("a quick-load button above");
            return;
        }
        
        let tree = self.tree.as_ref().unwrap();
        ui.label(format!("📄 {} ({} focuses)", tree.id, tree.focuses.len()));
        
        // Category breakdown
        let counts = self.category_counts();
        if !counts.is_empty() {
            ui.horizontal_wrapped(|ui| {
                for (cat, count) in &counts {
                    let icon = match cat.as_str() {
                        "Industry" => "🏭",
                        "Military" => "⚔️",
                        "Political" => "🏛️",
                        "Research" => "🔬",
                        "Foreign" => "🌍",
                        _ => "📋",
                    };
                    ui.small_button(format!("{} {}", icon, count))
                        .on_hover_text(format!("{}: {} focuses", cat, count));
                }
            });
        }
        
        ui.separator();
        
        // Search box
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.search_filter);
            if !self.search_filter.is_empty() && ui.small_button("✖").clicked() {
                self.search_filter.clear();
            }
        });
        
        // Category filter
        egui::ComboBox::from_label("")
            .selected_text(&self.category_filter)
            .show_ui(ui, |ui| {
                for cat in &["All", "Industry", "Military", "Political", "Research", "Foreign", "Other"] {
                    ui.selectable_value(&mut self.category_filter, cat.to_string(), *cat);
                }
            });
        
        ui.separator();
        
        // Focus list
        let filtered = self.filtered_focuses();
        let tree = self.tree.clone().unwrap();
        
        ui.label(format!("Showing {}/{} focuses", filtered.len(), tree.focuses.len()));
        
        egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
            for &idx in &filtered {
                if let Some(focus) = tree.focuses.get(idx) {
                    let selected = self.selected_focus_idx == Some(idx);
                    let icon = focus.category_icon();
                    let response = ui.selectable_label(selected, format!("{} {}", icon, focus.display_name()));
                    
                    if response.clicked() {
                        self.selected_focus_idx = Some(idx);
                    }
                    if response.double_clicked() {
                        self.selected_focus_idx = Some(idx);
                        self.open_editor();
                    }
                    
                    // Show cost and icon inline on next line
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        if let Some(ref icon_name) = focus.icon {
                            ui.label(format!("📷 {} | 💰 {:.1}d", icon_name, focus.cost.unwrap_or(0.0)));
                        } else {
                            ui.label(format!("💰 {:.1}d", focus.cost.unwrap_or(0.0)));
                        }
                        if !focus.prerequisites.is_empty() {
                            ui.label(format!("← {}", focus.prerequisites.len()));
                        }
                    });
                }
            }
        });
    }
    
    /// List view: details of selected focus
    fn ui_list_view(&mut self, ui: &mut egui::Ui) {
        if self.tree.is_none() {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("🪟 FocusFlow");
                    ui.label("HOI4 Focus Tree Editor for Millennium Dawn");
                    ui.label("");
                    ui.label("Load a focus tree to get started");
                });
            });
            return;
        }
        
        if let Some(idx) = self.selected_focus_idx {
            if let Some(tree) = &self.tree {
                if let Some(focus) = tree.focuses.get(idx) {
                    ui.heading(format!("{} {}", focus.category_icon(), focus.id));
                    ui.separator();
                    
                    // Info grid
                    egui::Grid::new("focus_info").striped(true).show(ui, |ui| {
                        ui.label("Icon:");
                        ui.label(focus.icon.as_deref().unwrap_or("(none)"));
                        ui.end_row();
                        
                        ui.label("Position:");
                        ui.label(format!("({}, {})", focus.x, focus.y));
                        ui.end_row();
                        
                        if let Some(ref rel) = focus.relative_position_id {
                            ui.label("Relative to:");
                            ui.label(rel);
                            ui.end_row();
                        }
                        
                        ui.label("Cost:");
                        ui.label(format!("{:.1} days", focus.cost.unwrap_or(0.0)));
                        ui.end_row();
                        
                        ui.label("Category:");
                        ui.colored_label(egui::Color32::from_rgb(
                            (focus.category_color()[0] * 255.0) as u8,
                            (focus.category_color()[1] * 255.0) as u8,
                            (focus.category_color()[2] * 255.0) as u8,
                        ), focus.category());
                        ui.end_row();
                        
                        ui.label("Bypass:");
                        ui.label(if focus.bypass_if_unavailable { "Yes" } else { "No" });
                        ui.end_row();
                        
                        if !focus.prerequisites.is_empty() {
                            ui.label("Prerequisites:");
                            ui.label(&focus.prerequisites.join(", "));
                            ui.end_row();
                        }
                        
                        if !focus.mutually_exclusive.is_empty() {
                            ui.label("Mutually Excl:");
                            ui.label(&focus.mutually_exclusive.join(", "));
                            ui.end_row();
                        }
                        
                        if !focus.search_filters.is_empty() {
                            ui.label("Filters:");
                            ui.label(&focus.search_filters.join(", "));
                            ui.end_row();
                        }
                    });
                    
                    ui.separator();
                    
                    // Preview of completion_reward
                    if let Some(ref reward) = focus.completion_reward_raw {
                        ui.label("Completion Reward:");
                        egui::Frame::dark_canvas(&ui.style()).show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(
                                    &mut reward.clone()
                                )
                                .code_editor()
                                .desired_rows(6)
                                .lock_focus(true)
                            );
                        });
                    }
                    
                    // Preview of ai_will_do
                    if let Some(ref ai) = focus.ai_will_do_raw {
                        ui.label("AI Will Do:");
                        egui::Frame::dark_canvas(&ui.style()).show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(
                                    &mut ai.clone()
                                )
                                .code_editor()
                                .desired_rows(3)
                                .lock_focus(true)
                            );
                        });
                    }
                    
                    ui.separator();
                    
                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("✏️ Edit (E)").clicked() {
                            self.open_editor();
                        }
                        if ui.button("📋 Duplicate").clicked() {
                            self.duplicate_selected_focus();
                        }
                        if ui.button("🗑️ Delete").clicked() {
                            self.delete_selected_focus();
                        }
                    });
                }
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("← Select a focus from the list to view details");
            });
        }
    }
    
    /// Canvas view: visual node graph
    fn ui_canvas_view(&mut self, ui: &mut egui::Ui) {
        if self.tree.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label("Load a focus tree to view the canvas");
            });
            return;
        }
        
        let tree = self.tree.as_ref().unwrap().clone();
        
        // Canvas controls
        ui.horizontal(|ui| {
            ui.label("Zoom:");
            if ui.add(egui::Slider::new(&mut self.canvas_zoom, 0.1..=3.0).text("Zoom")).changed() {
                // Zoom changed
            }
            ui.label(format!("{:.0}%", self.canvas_zoom * 100.0));
            ui.separator();
            if ui.small_button("Reset View").clicked() {
                self.canvas_zoom = 1.0;
                self.canvas_pan = egui::Vec2::ZERO;
            }
            if ui.small_button("Fit All").clicked() {
                // Auto-fit logic would go here
                self.canvas_zoom = 0.5;
                self.canvas_pan = egui::Vec2::new(100.0, 50.0);
            }
        });
        
        ui.separator();
        
        // Draw the canvas
        let (response, painter) = ui.allocate_painter(
            ui.available_size(),
            egui::Sense::click_and_drag(),
        );
        
        // Handle panning
        if response.dragged_by(egui::PointerButton::Secondary) {
            self.canvas_pan += response.drag_delta();
        }
        
        // Handle zoom with scroll
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta != 0.0 {
                let zoom_factor = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
                self.canvas_zoom = (self.canvas_zoom * zoom_factor).clamp(0.1, 3.0);
            }
        }
        
        let grid_w = 96.0 * self.canvas_zoom;
        let grid_h = 130.0 * self.canvas_zoom;
        let offset = self.canvas_pan;
        
        // Draw grid lines
        let rect = response.rect;
        for x in 0..=(rect.width() / grid_w) as i32 {
            let px = x as f32 * grid_w + offset.x;
            painter.line_segment(
                [egui::pos2(px, 0.0), egui::pos2(px, rect.height())],
                egui::Stroke::new(0.5, egui::Color32::from_gray(40)),
            );
        }
        for y in 0..=(rect.height() / grid_h) as i32 {
            let py = y as f32 * grid_h + offset.y;
            painter.line_segment(
                [egui::pos2(0.0, py), egui::pos2(rect.width(), py)],
                egui::Stroke::new(0.5, egui::Color32::from_gray(40)),
            );
        }
        
        // Draw connections first
        for focus in &tree.focuses {
            let (fx, fy) = focus.pixel_position(&tree, grid_w, grid_h);
            let screen_pos = egui::pos2(fx + offset.x, fy + offset.y);
            
            // Draw prerequisite connections
            for prereq_id in &focus.prerequisites {
                if let Some(prereq) = tree.focuses.iter().find(|f| f.id == *prereq_id) {
                    let (px, py) = prereq.pixel_position(&tree, grid_w, grid_h);
                    let prereq_pos = egui::pos2(px + offset.x, py + offset.y);
                    
                    painter.line_segment(
                        [prereq_pos, screen_pos],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
                    );
                    
                    // Arrow head
                    let dir = (screen_pos - prereq_pos).normalized();
                    let arrow_size = 8.0;
                    let tip = screen_pos - dir * 20.0 * self.canvas_zoom;
                    let perp = egui::vec2(-dir.y, dir.x) * arrow_size;
                    painter.line_segment(
                        [tip, tip - dir * arrow_size + perp],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
                    );
                    painter.line_segment(
                        [tip, tip - dir * arrow_size - perp],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
                    );
                }
            }
            
            // Draw mutually exclusive connections
            for me_id in &focus.mutually_exclusive {
                if let Some(me) = tree.focuses.iter().find(|f| f.id == *me_id) {
                    let (mx, my) = me.pixel_position(&tree, grid_w, grid_h);
                    let me_pos = egui::pos2(mx + offset.x, my + offset.y);
                    
                    // Dashed line effect (draw multiple segments)
                    let steps = 10;
                    for i in 0..steps {
                        if i % 2 == 0 {
                            let t1 = i as f32 / steps as f32;
                            let t2 = (i + 1) as f32 / steps as f32;
                            let p1 = screen_pos.lerp(me_pos, t1);
                            let p2 = screen_pos.lerp(me_pos, t2);
                            painter.line_segment(
                                [p1, p2],
                                egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 80, 80)),
                            );
                        }
                    }
                }
            }
        }
        
        // Draw focus nodes
        let node_w = 80.0 * self.canvas_zoom;
        let node_h = 30.0 * self.canvas_zoom;
        
        for focus in &tree.focuses {
            let (fx, fy) = focus.pixel_position(&tree, grid_w, grid_h);
            let pos = egui::pos2(fx + offset.x, fy + offset.y);
            let node_rect = egui::Rect::from_min_size(pos, egui::vec2(node_w, node_h));
            
            let selected = self.selected_focus_idx.map_or(false, |idx| 
                tree.focuses.get(idx).map_or(false, |f| f.id == focus.id)
            );
            
            let color = egui::Color32::from_rgb(
                (focus.category_color()[0] * 255.0) as u8,
                (focus.category_color()[1] * 255.0) as u8,
                (focus.category_color()[2] * 255.0) as u8,
            );
            
            let fill_color = if selected {
                color.linear_multiply(1.5)
            } else {
                color.linear_multiply(0.6)
            };
            
            painter.rect_filled(node_rect, 4.0, fill_color);
            painter.rect_stroke(node_rect, 4.0, egui::Stroke::new(1.5, egui::Color32::WHITE), egui::StrokeKind::Inside);
            
            // Label
            let label = focus.display_name();
            let label = if label.len() > 15 { &label[..12] } else { label };
            painter.text(
                node_rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(10.0 * self.canvas_zoom),
                egui::Color32::WHITE,
            );
        }
        
        // Instructions
        ui.label("🖱️ Right-click drag to pan | Scroll to zoom | Click node to select");
    }
    
    /// Editor panel
    fn ui_editor_panel(&mut self, ui: &mut egui::Ui) {
        let creating_new = self.creating_new;
        ui.heading(if creating_new { "✨ New Focus" } else { "✏️ Edit Focus" });
        ui.separator();
        
        if self.editing_focus.is_none() {
            ui.label("No focus being edited");
            return;
        }
        
        let mut edited = self.editing_focus.clone().unwrap();
        let mut should_save = false;
        let mut should_cancel = false;
        
        egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
            Self::render_focus_editor(ui, &mut edited, &mut should_save, &mut should_cancel);
        });
        
        if should_save {
            self.editing_focus = Some(edited);
            self.save_edited_focus();
        } else if should_cancel {
            self.show_editor = false;
            self.editing_focus = None;
        } else {
            self.editing_focus = Some(edited);
        }
    }
    
    /// Render the focus editor UI
    fn render_focus_editor(
        ui: &mut egui::Ui,
        edited: &mut FocusNode,
        should_save: &mut bool,
        should_cancel: &mut bool,
    ) {
        ui.label("ID:");
        ui.text_edit_singleline(&mut edited.id);
        
        ui.label("Icon:");
        let mut icon_edit = edited.icon.clone().unwrap_or_default();
        ui.text_edit_singleline(&mut icon_edit);
        edited.icon = if icon_edit.is_empty() { None } else { Some(icon_edit) };
        
        ui.label("Position:");
        ui.horizontal(|ui| {
            ui.label("X:");
            ui.add(egui::DragValue::new(&mut edited.x).range(-50..=50));
            ui.label("Y:");
            ui.add(egui::DragValue::new(&mut edited.y).range(-10..=50));
        });
        
        ui.label("Relative Position ID:");
        let mut rel_str = edited.relative_position_id.clone().unwrap_or_default();
        ui.text_edit_singleline(&mut rel_str);
        edited.relative_position_id = if rel_str.is_empty() { None } else { Some(rel_str) };
        
        ui.label("Cost (days):");
        let mut cost_val = edited.cost.unwrap_or(5.0);
        ui.add(egui::DragValue::new(&mut cost_val).range(0.1..=100.0).speed(0.1));
        edited.cost = Some(cost_val);
        
        ui.separator();
        
        ui.label("Prerequisites (comma-separated IDs):");
        let mut prereq_str = edited.prerequisites.join(", ");
        ui.text_edit_singleline(&mut prereq_str);
        edited.prerequisites = prereq_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        
        ui.label("Mutually Exclusive (comma-separated IDs):");
        let mut me_str = edited.mutually_exclusive.join(", ");
        ui.text_edit_singleline(&mut me_str);
        edited.mutually_exclusive = me_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        
        ui.separator();
        
        ui.label("Search Filters:");
        let mut sf_str = edited.search_filters.join(", ");
        ui.text_edit_singleline(&mut sf_str);
        edited.search_filters = sf_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        
        ui.checkbox(&mut edited.bypass_if_unavailable, "Bypass if unavailable");
        
        ui.separator();
        
        ui.label("Completion Reward (Paradox script):");
        let mut reward_str = edited.completion_reward_raw.clone().unwrap_or_else(|| "{\n}".to_string());
        ui.add(egui::TextEdit::multiline(&mut reward_str).code_editor().desired_rows(8).desired_width(f32::INFINITY));
        edited.completion_reward_raw = Some(reward_str);
        
        ui.label("AI Will Do:");
        let mut ai_str = edited.ai_will_do_raw.clone().unwrap_or_else(|| "{\n\tbase = 10\n}".to_string());
        ui.add(egui::TextEdit::multiline(&mut ai_str).code_editor().desired_rows(4).desired_width(f32::INFINITY));
        edited.ai_will_do_raw = Some(ai_str);
        
        ui.separator();
        
        ui.horizontal(|ui| {
            if ui.button("💾 Save").clicked() {
                *should_save = true;
            }
            if ui.button("❌ Cancel").clicked() {
                *should_cancel = true;
            }
        });
    }
    
    /// Validation panel (floating, with crimson accents for errors)
    fn ui_validation_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Validación");
        ui.separator();
        
        if let Some(result) = &self.validation {
            ui.label(format!("{} errores, {} advertencias", result.errors.len(), result.warnings.len()));
            ui.separator();
            
            if !result.errors.is_empty() {
                ui.label(egui::RichText::new("Errores:").color(egui::Color32::from_rgb(220, 38, 38)));
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for err in &result.errors {
                        ui.colored_label(egui::Color32::from_rgb(220, 38, 38), format!("● {}", err));
                    }
                });
            }
            
            if !result.warnings.is_empty() {
                ui.label(egui::RichText::new("Advertencias:").color(egui::Color32::from_rgb(251, 191, 36)));
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for warn in &result.warnings {
                        ui.colored_label(egui::Color32::from_rgb(251, 191, 36), format!("● {}", warn));
                    }
                });
            }
            
            if result.is_ok() {
                ui.label(egui::RichText::new("✓ Sin problemas encontrados").color(egui::Color32::from_rgb(16, 185, 129)));
            }
        } else {
            ui.label("Ejecuta validación para ver resultados");
            if ui.button("Ejecutar Validación").clicked() {
                self.run_validation();
            }
        }
    }
    
}



