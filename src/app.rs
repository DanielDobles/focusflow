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

    /// Diff content for Intel & Logs
    pub diff_content: String,
    
    /// Undo history
    pub undo_stack: Vec<String>,
    pub redo_stack: Vec<String>,
}

impl FocusFlowApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Millennium Command Style Initialization
        let mut style = (*cc.egui_ctx.style()).clone();
        
        // Obsidian & Charcoal Palette
        let obsidian = egui::Color32::from_rgb(18, 19, 23);
        let charcoal = egui::Color32::from_rgb(28, 30, 34);
        let tactical_cyan = egui::Color32::from_rgb(102, 252, 241);
        let deep_black = egui::Color32::from_rgb(11, 12, 16);
        
        style.spacing.item_spacing = egui::vec2(12.0, 12.0);
        style.visuals.window_fill = egui::Color32::from_rgba_unmultiplied(18, 19, 23, 240);
        style.visuals.panel_fill = deep_black;
        style.visuals.widgets.noninteractive.bg_fill = charcoal;
        style.visuals.extreme_bg_color = deep_black;
        
        // High-fidelity selection accents
        style.visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(102, 252, 241, 40);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, tactical_cyan);
        
        // No-line window frames (depth via tonal layering)
        style.visuals.window_stroke = egui::Stroke::NONE;
        style.visuals.window_shadow = egui::epaint::Shadow {
            offset: [(0.0) as i8, (12.0) as i8],
            blur: 40.0 as u8,
            spread: 2.0 as u8,
            color: egui::Color32::from_black_alpha(200),
        };
        style.visuals.popup_shadow = style.visuals.window_shadow;
        
        cc.egui_ctx.set_style(style);
        
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(egui::Color32::from_rgb(197, 198, 199)); // Operational Gray
        visuals.widgets.active.bg_fill = tactical_cyan;
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(69, 162, 158); // Emerald hover
        cc.egui_ctx.set_visuals(visuals);
        
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
            diff_content: String::new(),
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
                        self.diff_content.clear();
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

    /// Deploy: save file and push to git
    pub fn deploy_operations(&mut self) {
        self.save_file();

        // Try to git commit and push from the focusflow directory
        let deploy_result = std::process::Command::new("cmd.exe")
            .args(&["/c", "cd", "C:\\Users\\armon\\DEV\\HOI4_MD_FT\\focusflow", "&&", "git", "add", ".", "&&", "git", "diff", "--staged", "--quiet"])
            .output();

        match deploy_result {
            Ok(output) => {
                // If there are staged changes, commit and push
                if !output.status.success() {
                    let commit_result = std::process::Command::new("cmd.exe")
                        .args(&["/c", "cd", "C:\\Users\\armon\\DEV\\HOI4_MD_FT\\focusflow", "&&", "git", "commit", "-m", "FocusFlow: update focus tree", "&&", "git", "push"])
                        .output();

                    match commit_result {
                        Ok(output) if output.status.success() => {
                            self.status_message = "✅ Deployed: saved and pushed to git".to_string();
                        }
                        Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            self.status_message = format!("⚠️ Saved but git push failed: {}", stderr.lines().next().unwrap_or("unknown error"));
                        }
                        Err(e) => {
                            self.status_message = format!("⚠️ Saved but git error: {}", e);
                        }
                    }
                } else {
                    self.status_message = "✅ Saved: no git changes to push".to_string();
                }
            }
            Err(e) => {
                self.status_message = format!("⚠️ Saved but git error: {}", e);
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

    /// Generate diff between original and current tree
    pub fn generate_intel_logs(&mut self) {
        if let (Some(original), Some(current)) = (&self.original_tree, &self.tree) {
            self.diff_content = writer::generate_diff(original, current);
        } else if let Some(current) = &self.tree {
            self.diff_content = format!("📄 Tree: {}\n📊 Total focuses: {}\n📝 Modified: {}",
                current.id,
                current.focuses.len(),
                if current.modified { "Yes" } else { "No" }
            );
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
        self.handle_keyboard(ctx);

        let tactical_blue = egui::Color32::from_rgb(59, 130, 246);
        let warning_amber = egui::Color32::from_rgb(245, 158, 11);
        let error_crimson = egui::Color32::from_rgb(239, 68, 68);
        let surface_high = egui::Color32::from_rgb(26, 28, 34);
        let surface = egui::Color32::from_rgb(15, 17, 21);
        let on_surface = egui::Color32::from_rgb(229, 231, 235);
        let muted_gray = egui::Color32::from_rgb(107, 114, 128);
        let deep_black = egui::Color32::from_rgb(11, 12, 16);
        
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(surface)).show(ctx, |ui| {
            if self.view_mode == AppView::Canvas {
                self.ui_canvas_view(ui);
            } else {
                egui::Window::new("Data Assets")
                    .id(egui::Id::new("list_view_window"))
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .default_size(egui::vec2(1000.0, 700.0))
                    .frame(egui::Frame::window(&ctx.style()).fill(surface_high).stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(15))))
                    .title_bar(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.allocate_ui(egui::vec2(350.0, ui.available_height()), |ui| { self.ui_left_panel(ui); });
                            ui.add_space(8.0); // separator removed
                            ui.allocate_ui(ui.available_size(), |ui| { self.ui_list_view(ui); });
                        });
                    });
            }

            egui::Window::new("Command Protocols")
                .id(egui::Id::new("cmd_protocols"))
                .fixed_pos(egui::pos2(120.0, 100.0))
                .fixed_size(egui::vec2(280.0, 200.0))
                .resizable(false).title_bar(false)
                .frame(egui::Frame::window(&ctx.style()).fill(surface_high).stroke(egui::Stroke::new(1.5, tactical_blue)))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("TERMINAL").color(tactical_blue).size(10.0).monospace());
                        ui.label(egui::RichText::new("COMMAND PROTOCOLS").color(tactical_blue).size(12.0).strong());
                    });
                    ui.add_space(16.0);
                    
                    let btn_style = egui::vec2(ui.available_width(), 36.0);
                    
                    let b1 = egui::Button::new(egui::RichText::new("NEW MISSION").size(11.0).strong().color(on_surface)).fill(egui::Color32::from_rgb(31, 33, 40));
                    if ui.add_sized(btn_style, b1).clicked() { self.create_new_focus(); }
                    
                    ui.add_space(4.0);
                    
                    let b2 = egui::Button::new(egui::RichText::new("ARCHIVE MAP (RELOAD)").size(11.0).strong().color(on_surface)).fill(egui::Color32::from_rgb(31, 33, 40));
                    if ui.add_sized(btn_style, b2).clicked() {
                        if let Some(path) = self.file_path.clone() { self.load_file(&path); }
                    }
                    
                    ui.add_space(4.0);
                    
                    let b3 = egui::Button::new(egui::RichText::new("SYNC INTEL (SAVE)").size(11.0).strong().color(tactical_blue)).fill(egui::Color32::from_rgb(31, 33, 40));
                    if ui.add_sized(btn_style, b3).clicked() { self.save_file(); }
                });

            if self.show_editor {
                let mut editor_open = true;
                egui::Window::new("Node Jurisdiction")
                    .id(egui::Id::new("node_jurisdiction"))
                    .fixed_pos(egui::pos2(ctx.screen_rect().right() - 400.0, 100.0))
                    .fixed_size(egui::vec2(360.0, ctx.screen_rect().height() - 320.0))
                    .resizable(false).title_bar(false)
                    .frame(egui::Frame::window(&ctx.style()).fill(surface_high).stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(20))))
                    .open(&mut editor_open)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("NODE JURISDICTION").color(warning_amber).size(12.0).strong());
                                if let Some(f) = &self.editing_focus {
                                    ui.label(egui::RichText::new(format!("OBJECT_ID: {}", f.id)).color(muted_gray).size(10.0).monospace());
                                }
                            });
                        });
                        ui.add_space(16.0);
                        self.ui_editor_panel(ui);
                    });
                self.show_editor = editor_open;
            }

            // Operational Readiness
            egui::Window::new("Operational Readiness")
                .id(egui::Id::new("op_readiness"))
                .fixed_pos(egui::pos2(ctx.screen_rect().right() - 400.0, ctx.screen_rect().bottom() - 180.0))
                .fixed_size(egui::vec2(360.0, 150.0))
                .resizable(false).title_bar(false)
                .frame(egui::Frame::window(&ctx.style()).fill(surface_high).stroke(egui::Stroke::new(2.0, warning_amber)))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("OPERATIONAL READINESS").color(on_surface).size(12.0).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(if self.validation.as_ref().map_or(false, |v| v.is_ok()) { "94%" } else { "42%" }).color(warning_amber).size(24.0).strong());
                        });
                    });
                    ui.add_space(8.0);
                    let text = if let Some(val) = &self.validation {
                        format!("System parameters verified. {} conflicts detected.", val.total_issues())
                    } else {
                        "Run validation to verify system parameters.".to_string()
                    };
                    ui.label(egui::RichText::new(text).color(egui::Color32::from_white_alpha(150)).size(10.0));
                    ui.add_space(12.0);
                    if ui.add_sized(egui::vec2(ui.available_width(), 36.0), egui::Button::new(egui::RichText::new("COMMIT SELECTION").color(deep_black).strong()).fill(warning_amber)).clicked() {
                        self.save_file();
                    }
                });

            if self.show_validation {
                let mut v_open = true;
                egui::Window::new("Urgent Intel")
                    .id(egui::Id::new("urgent_intel"))
                    .fixed_pos(egui::pos2(ctx.screen_rect().right() - 780.0, ctx.screen_rect().bottom() - 280.0))
                    .fixed_size(egui::vec2(340.0, 250.0))
                    .resizable(false).title_bar(false)
                    .frame(egui::Frame::window(&ctx.style()).fill(egui::Color32::from_rgba_unmultiplied(147, 0, 10, 50)).stroke(egui::Stroke::new(1.0, error_crimson)))
                    .open(&mut v_open)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("WARNING").color(error_crimson).size(10.0).monospace());
                            ui.label(egui::RichText::new("URGENT INTEL").color(error_crimson).size(12.0).strong());
                        });
                        ui.add_space(12.0);
                        self.ui_validation_panel(ui);
                    });
                self.show_validation = v_open;
            }

            // INTEL & LOGS panel
            if self.show_diff {
                let mut intel_open = true;
                egui::Window::new("Intel & Logs")
                    .id(egui::Id::new("intel_logs"))
                    .fixed_pos(egui::pos2(ctx.screen_rect().right() - 780.0, ctx.screen_rect().bottom() - 380.0))
                    .fixed_size(egui::vec2(740.0, 350.0))
                    .resizable(true).title_bar(false)
                    .frame(egui::Frame::window(&ctx.style()).fill(surface_high).stroke(egui::Stroke::new(1.0, tactical_blue)))
                    .open(&mut intel_open)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("SYSTEM").color(tactical_blue).size(10.0).monospace());
                            ui.label(egui::RichText::new("INTEL & LOGS").color(tactical_blue).size(12.0).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("🔄 Refresh").clicked() {
                                    self.generate_intel_logs();
                                }
                            });
                        });
                        ui.add_space(8.0);
                        egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
                            egui::Frame::dark_canvas(&ui.style()).show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.diff_content.clone())
                                        .code_editor()
                                        .desired_rows(15)
                                        .desired_width(ui.available_width())
                                        .lock_focus(true)
                                );
                            });
                        });
                    });
                self.show_diff = intel_open;
            }
        });

        // Top Navigation Header
        egui::TopBottomPanel::top("top_nav_header")
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgba_unmultiplied(17, 19, 23, 220)).inner_margin(egui::Margin::symmetric(24, 16)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("FocusFlow").color(tactical_blue).size(20.0).strong());
                    
                    let sep_rect = egui::Rect::from_min_size(ui.cursor().min + egui::vec2(16.0, 4.0), egui::vec2(1.0, 20.0));
                    ui.painter().rect_filled(sep_rect, 0.0, egui::Color32::from_rgba_unmultiplied(134, 148, 138, 76));
                    ui.add_space(32.0);

                    if ui.selectable_label(self.view_mode == AppView::Canvas, egui::RichText::new("NODES").size(12.0).strong()).clicked() { self.view_mode = AppView::Canvas; }
                    if ui.selectable_label(self.view_mode == AppView::List, egui::RichText::new("ASSETS").size(12.0).strong()).clicked() { self.view_mode = AppView::List; }
                    if ui.selectable_label(self.show_diff, egui::RichText::new("INTEL & LOGS").size(12.0).strong()).clicked() {
                        self.show_diff = !self.show_diff;
                        if self.show_diff {
                            self.generate_intel_logs();
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let deploy_btn = ui.add(egui::Button::new(
                            egui::RichText::new("DEPLOY OPERATIONS").color(deep_black).size(12.0).strong()
                        ).fill(egui::Color32::from_rgb(16, 185, 129)));
                        if deploy_btn.clicked() { self.deploy_operations(); }

                        ui.add_space(16.0);

                        let read_btn = ui.add(egui::Button::new(
                            egui::RichText::new("OPERATIONAL READINESS").color(on_surface).size(12.0).strong()
                        ).fill(egui::Color32::from_rgb(40, 42, 46)).stroke(egui::Stroke::new(1.0, tactical_blue)));
                        if read_btn.clicked() { self.run_validation(); }
                    });
                });
            });

        // Left Navigation Sidebar
        egui::SidePanel::left("left_nav_sidebar")
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgba_unmultiplied(17, 19, 23, 240)).inner_margin(egui::Margin::symmetric(0, 40)))
            .exact_width(64.0).resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    if ui.add_sized(egui::vec2(48.0, 48.0), egui::SelectableLabel::new(self.view_mode == AppView::Canvas, "🌐
NAV")).on_hover_text("Terminal").clicked() { self.view_mode = AppView::Canvas; }
                    ui.add_space(16.0);
                    if ui.add_sized(egui::vec2(48.0, 48.0), egui::SelectableLabel::new(self.show_editor, "✏️
EDIT")).on_hover_text("Editor").clicked() { self.show_editor = !self.show_editor; }
                    ui.add_space(16.0);
                    if ui.add_sized(egui::vec2(48.0, 48.0), egui::SelectableLabel::new(self.show_validation, "⚠️
VAL")).on_hover_text("Validation").clicked() { self.show_validation = !self.show_validation; }
                });
            });
    }
}

impl FocusFlowApp {
    // Left panel: file load, search, focus list
}

impl FocusFlowApp {
    fn ui_left_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("FocusFlow");
        ui.add_space(8.0); // separator removed
        
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
        
        ui.add_space(8.0); // separator removed
        
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
        
        ui.add_space(8.0); // separator removed
        
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
        
        ui.add_space(8.0); // separator removed
        
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
                    // ui.heading removed
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
                    ui.add_space(8.0); // separator removed

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

                    ui.add_space(8.0); // separator removed

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

                    ui.add_space(8.0); // separator removed

                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("✏️ Edit Full (E)").clicked() {
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
                ui.vertical_centered(|ui| {
                    ui.label("← Select a focus from the Assets list");
                    ui.label("");
                    ui.label("Click a focus to view details");
                    ui.label("Double-click or click 'Edit Full' to edit");
                });
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
            ui.add_space(8.0); // separator removed
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
        
        ui.add_space(8.0); // separator removed
        
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
                    
                    // Tactical dotted line
                    let steps = 15;
                    for i in 0..steps {
                        if i % 2 == 0 {
                            let t1 = i as f32 / steps as f32;
                            let t2 = (i + 1) as f32 / steps as f32;
                            painter.line_segment(
                                [screen_pos.lerp(prereq_pos, t1), screen_pos.lerp(prereq_pos, t2)],
                                egui::Stroke::new(1.0, egui::Color32::from_rgb(59, 130, 246)), // Operational Blue
                            );
                        }
                    }
                    
                    // Arrow head
                    let dir = (screen_pos - prereq_pos).normalized();
                    let arrow_size = 6.0 * self.canvas_zoom;
                    let tip = screen_pos - dir * 25.0 * self.canvas_zoom;
                    let perp = egui::vec2(-dir.y, dir.x) * (arrow_size * 0.5);
                    painter.line_segment(
                        [tip, tip - dir * arrow_size + perp],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(59, 130, 246)),
                    );
                    painter.line_segment(
                        [tip, tip - dir * arrow_size - perp],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(59, 130, 246)),
                    );
                }
            }
            
            // Draw mutually exclusive connections
            for me_id in &focus.mutually_exclusive {
                if let Some(me) = tree.focuses.iter().find(|f| f.id == *me_id) {
                    let (mx, my) = me.pixel_position(&tree, grid_w, grid_h);
                    let me_pos = egui::pos2(mx + offset.x, my + offset.y);
                    
                    painter.line_segment(
                        [screen_pos, me_pos],
                        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(239, 68, 68, 100)), // Critical Red
                    );
                }
            }
        }
        
        // Draw focus nodes
        let node_w = 84.0 * self.canvas_zoom;
        let node_h = 42.0 * self.canvas_zoom;
        
        for focus in &tree.focuses {
            let (fx, fy) = focus.pixel_position(&tree, grid_w, grid_h);
            let pos = egui::pos2(fx + offset.x, fy + offset.y);
            let node_rect = egui::Rect::from_min_size(pos - egui::vec2(node_w/2.0, node_h/2.0), egui::vec2(node_w, node_h));
            
            let selected = self.selected_focus_idx.map_or(false, |idx| 
                tree.focuses.get(idx).map_or(false, |f| f.id == focus.id)
            );
            
            self.render_tactical_node(&painter, node_rect, focus, selected, self.canvas_zoom);
            
            // Handle clicking
            if response.clicked() && node_rect.contains(response.interact_pointer_pos().unwrap_or(egui::pos2(-1000.0, -1000.0))) {
                if let Some(idx) = tree.focuses.iter().position(|f| f.id == focus.id) {
                    self.selected_focus_idx = Some(idx);
                }
            }
        }
        
        // Instructions
        ui.label("🖱️ Right-click drag to pan | Scroll to zoom | Click node to select");
    }
    
    /// Editor panel
    fn ui_editor_panel(&mut self, ui: &mut egui::Ui) {
        let _creating_new = self.creating_new;
        // edit heading removed
        ui.add_space(8.0); // separator removed
        
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
        
        ui.add_space(8.0); // separator removed
        
        ui.label("Prerequisites (comma-separated IDs):");
        let mut prereq_str = edited.prerequisites.join(", ");
        ui.text_edit_singleline(&mut prereq_str);
        edited.prerequisites = prereq_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        
        ui.label("Mutually Exclusive (comma-separated IDs):");
        let mut me_str = edited.mutually_exclusive.join(", ");
        ui.text_edit_singleline(&mut me_str);
        edited.mutually_exclusive = me_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        
        ui.add_space(8.0); // separator removed
        
        ui.label("Search Filters:");
        let mut sf_str = edited.search_filters.join(", ");
        ui.text_edit_singleline(&mut sf_str);
        edited.search_filters = sf_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        
        ui.checkbox(&mut edited.bypass_if_unavailable, "Bypass if unavailable");
        
        ui.add_space(8.0); // separator removed
        
        ui.label("Completion Reward (Paradox script):");
        let mut reward_str = edited.completion_reward_raw.clone().unwrap_or_else(|| "{\n}".to_string());
        ui.add(egui::TextEdit::multiline(&mut reward_str).code_editor().desired_rows(8).desired_width(f32::INFINITY));
        edited.completion_reward_raw = Some(reward_str);
        
        ui.label("AI Will Do:");
        let mut ai_str = edited.ai_will_do_raw.clone().unwrap_or_else(|| "{\n\tbase = 10\n}".to_string());
        ui.add(egui::TextEdit::multiline(&mut ai_str).code_editor().desired_rows(4).desired_width(f32::INFINITY));
        edited.ai_will_do_raw = Some(ai_str);
        
        ui.add_space(8.0); // separator removed
        
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
        // validation heading removed
        ui.add_space(8.0); // separator removed
        
        if let Some(result) = &self.validation {
            ui.label(format!("{} errores, {} advertencias", result.errors.len(), result.warnings.len()));
            ui.add_space(8.0); // separator removed
            
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
        }
    }

    /// Render a high-fidelity tactical node (Palantir/Millennium style)
    fn render_tactical_node(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        focus: &FocusNode,
        selected: bool,
        zoom: f32,
    ) {
        let charcoal = egui::Color32::from_rgb(28, 30, 34);
        let tactical_blue = egui::Color32::from_rgb(59, 130, 246);
        let deep_black = egui::Color32::from_rgb(11, 12, 16);
        let border_color = if selected { tactical_blue } else { egui::Color32::from_rgb(60, 64, 72) };
        
        // 1. Draw persistent ambient glow for selected nodes
        if selected {
            for i in 1..=4 {
                let glow_rect = rect.expand(i as f32 * 2.0 * zoom);
                self.paint_octagon(painter, glow_rect, egui::Color32::from_rgba_unmultiplied(59, 130, 246, 20 / i), egui::Stroke::NONE);
            }
        }

        // 2. Main Node Body (Octagon)
        self.paint_octagon(painter, rect, charcoal, egui::Stroke::new(1.5 * zoom, border_color));

        // 3. Header Bar (Precision Metadata)
        let header_height = 8.0 * zoom;
        let header_rect = egui::Rect::from_min_max(
            rect.min,
            egui::pos2(rect.max.x, rect.min.y + header_height)
        );
        self.paint_octagon_top(painter, header_rect, deep_black, egui::Stroke::NONE);
        
        // ID Label in Monospace
        let id_text = if focus.id.len() > 10 { format!("{}...", &focus.id[..8]) } else { focus.id.clone() };
        painter.text(
            header_rect.center(),
            egui::Align2::CENTER_CENTER,
            id_text.to_uppercase(),
            egui::FontId::monospace(6.0 * zoom),
            egui::Color32::from_rgb(156, 163, 175),
        );

        // 4. Main Label
        let display_name = focus.display_name();
        let display_name = if display_name.len() > 14 { format!("{}...", &display_name[..11]) } else { display_name.to_string() };
        painter.text(
            rect.center() + egui::vec2(0.0, 4.0 * zoom),
            egui::Align2::CENTER_CENTER,
            display_name,
            egui::FontId::proportional(10.0 * zoom),
            egui::Color32::WHITE,
        );
    }

    /// Helper to paint an octagon (clipped corners)
    fn paint_octagon(&self, painter: &egui::Painter, rect: egui::Rect, fill: egui::Color32, stroke: egui::Stroke) {
        let bevel = rect.width().min(rect.height()) * 0.15;
        let points = vec![
            egui::pos2(rect.min.x + bevel, rect.min.y),
            egui::pos2(rect.max.x - bevel, rect.min.y),
            egui::pos2(rect.max.x, rect.min.y + bevel),
            egui::pos2(rect.max.x, rect.max.y - bevel),
            egui::pos2(rect.max.x - bevel, rect.max.y),
            egui::pos2(rect.min.x + bevel, rect.max.y),
            egui::pos2(rect.min.x, rect.max.y - bevel),
            egui::pos2(rect.min.x, rect.min.y + bevel),
        ];
        painter.add(egui::Shape::convex_polygon(points, fill, stroke));
    }

    /// Helper to paint only the top part of an octagon (for headers)
    fn paint_octagon_top(&self, painter: &egui::Painter, rect: egui::Rect, fill: egui::Color32, stroke: egui::Stroke) {
        let bevel = rect.width() * 0.15;
        let points = vec![
            egui::pos2(rect.min.x + bevel, rect.min.y),
            egui::pos2(rect.max.x - bevel, rect.min.y),
            egui::pos2(rect.max.x, rect.min.y + bevel),
            egui::pos2(rect.max.x, rect.max.y),
            egui::pos2(rect.min.x, rect.max.y),
            egui::pos2(rect.min.x, rect.min.y + bevel),
        ];
        painter.add(egui::Shape::convex_polygon(points, fill, stroke));
    }
}



