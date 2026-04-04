import os

file_path = r"C:\Users\armon\DEV\HOI4_MD_FT\focusflow\src\app.rs"

with open(file_path, "r", encoding="utf-8") as f:
    content = f.read()

# 1. Update FocusFlowApp::new
old_new = """    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
        cc.egui_ctx.set_visuals(egui::Visuals::dark());"""

new_new = """    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.visuals.window_fill = egui::Color32::from_rgba_unmultiplied(26, 28, 32, 230);
        style.visuals.panel_fill = egui::Color32::from_rgb(17, 19, 23);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 32, 36); 
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(12, 14, 18); 
        style.visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(0, 227, 139, 60); 
        style.visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 227, 139)); 
        style.visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(134, 148, 138, 40));
        style.visuals.window_shadow = egui::epaint::Shadow {
            offset: [(0.0) as i8, (8.0) as i8],
            blur: 24.0 as u8,
            spread: 0.0 as u8,
            color: egui::Color32::from_black_alpha(150),
        };
        style.visuals.popup_shadow = style.visuals.window_shadow.clone();
        cc.egui_ctx.set_style(style);
        
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(egui::Color32::from_rgb(226, 226, 232));
        cc.egui_ctx.set_visuals(visuals);"""

content = content.replace(old_new, new_new)

# 2. Update the whole App update implementation
start_idx = content.find("impl eframe::App for FocusFlowApp {\n    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {")
end_idx = content.find("    /// Left panel")

if start_idx != -1 and end_idx != -1:
    pre_content = content[:start_idx]
    post_content = content[end_idx:]

    new_update = """impl eframe::App for FocusFlowApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard(ctx);

        let primary = egui::Color32::from_rgb(0, 227, 139);
        let secondary = egui::Color32::from_rgb(255, 207, 143);
        let error = egui::Color32::from_rgb(255, 180, 171);
        let surface_container_high = egui::Color32::from_rgba_unmultiplied(40, 42, 46, 240);
        let surface = egui::Color32::from_rgb(17, 19, 23);
        let on_surface = egui::Color32::from_rgb(226, 226, 232);
        
        egui::CentralPanel::default().frame(egui::Frame::none().fill(surface)).show(ctx, |ui| {
            if self.view_mode == AppView::Canvas {
                self.ui_canvas_view(ui);
            } else {
                egui::Window::new("Data Assets")
                    .id(egui::Id::new("list_view_window"))
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .default_size(egui::vec2(1000.0, 700.0))
                    .frame(egui::Frame::window(&ctx.style()).fill(surface_container_high).stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(20))))
                    .title_bar(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.allocate_ui(egui::vec2(350.0, ui.available_height()), |ui| { self.ui_left_panel(ui); });
                            ui.separator();
                            ui.allocate_ui(ui.available_size(), |ui| { self.ui_list_view(ui); });
                        });
                    });
            }

            egui::Window::new("Command Protocols")
                .id(egui::Id::new("cmd_protocols"))
                .fixed_pos(egui::pos2(120.0, 100.0))
                .fixed_size(egui::vec2(280.0, 200.0))
                .resizable(false).title_bar(false)
                .frame(egui::Frame::window(&ctx.style()).fill(surface_container_high).stroke(egui::Stroke::new(2.0, primary)))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("TERMINAL").color(primary).size(10.0));
                        ui.label(egui::RichText::new("COMMAND PROTOCOLS").color(primary).size(12.0).strong().tracking(1.5));
                    });
                    ui.add_space(16.0);
                    
                    let btn_style = egui::vec2(ui.available_width(), 36.0);
                    let mut draw_btn = |text: &str, accent: bool| {
                        let b = egui::Button::new(egui::RichText::new(text).size(11.0).strong().color(if accent { primary } else { on_surface }))
                            .fill(egui::Color32::from_rgb(30, 32, 36));
                        ui.add_sized(btn_style, b)
                    };
                    
                    if draw_btn("NEW MISSION", false).clicked() { self.create_new_focus(); }
                    ui.add_space(4.0);
                    if draw_btn("ARCHIVE MAP (RELOAD)", false).clicked() {
                        if let Some(path) = self.file_path.clone() { self.load_file(&path); }
                    }
                    ui.add_space(4.0);
                    if draw_btn("SYNC INTEL (SAVE)", true).clicked() { self.save_file(); }
                });

            if self.show_editor {
                let mut editor_open = true;
                egui::Window::new("Node Jurisdiction")
                    .id(egui::Id::new("node_jurisdiction"))
                    .fixed_pos(egui::pos2(ctx.screen_rect().right() - 400.0, 100.0))
                    .fixed_size(egui::vec2(360.0, ctx.screen_rect().height() - 320.0))
                    .resizable(false).title_bar(false)
                    .frame(egui::Frame::window(&ctx.style()).fill(surface_container_high).stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30))))
                    .open(&mut editor_open)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("NODE JURISDICTION").color(secondary).size(12.0).strong().tracking(1.5));
                                if let Some(f) = &self.editing_focus {
                                    ui.label(egui::RichText::new(format!("ID: {}", f.id)).color(egui::Color32::from_white_alpha(100)).size(10.0));
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
                .frame(egui::Frame::window(&ctx.style()).fill(surface_container_high).stroke(egui::Stroke::new(2.0, secondary)))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("OPERATIONAL READINESS").color(on_surface).size(12.0).strong().tracking(1.5));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(if self.validation.as_ref().map_or(false, |v| v.is_ok()) { "94%" } else { "42%" }).color(secondary).size(24.0).strong());
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
                    if ui.add_sized(egui::vec2(ui.available_width(), 36.0), egui::Button::new(egui::RichText::new("COMMIT SELECTION").color(egui::Color32::BLACK).strong()).fill(secondary)).clicked() {
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
                    .frame(egui::Frame::window(&ctx.style()).fill(egui::Color32::from_rgba_unmultiplied(147, 0, 10, 50)).stroke(egui::Stroke::new(1.0, error)))
                    .open(&mut v_open)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("WARNING").color(error).size(10.0));
                            ui.label(egui::RichText::new("URGENT INTEL").color(error).size(12.0).strong().tracking(1.5));
                        });
                        ui.add_space(12.0);
                        self.ui_validation_panel(ui);
                    });
                self.show_validation = v_open;
            }
        });

        // Top Navigation Header
        egui::TopBottomPanel::top("top_nav_header")
            .frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(17, 19, 23, 220)).inner_margin(egui::Margin::symmetric(24.0, 16.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("FocusFlow").color(primary).size(20.0).strong());
                    
                    let sep_rect = egui::Rect::from_min_size(ui.cursor().min + egui::vec2(16.0, 4.0), egui::vec2(1.0, 20.0));
                    ui.painter().rect_filled(sep_rect, 0.0, egui::Color32::from_rgba_unmultiplied(134, 148, 138, 76));
                    ui.add_space(32.0);

                    if ui.selectable_label(self.view_mode == AppView::Canvas, egui::RichText::new("NODES").size(12.0).strong()).clicked() { self.view_mode = AppView::Canvas; }
                    if ui.selectable_label(self.view_mode == AppView::List, egui::RichText::new("ASSETS").size(12.0).strong()).clicked() { self.view_mode = AppView::List; }
                    if ui.selectable_label(self.show_diff, egui::RichText::new("INTEL").size(12.0).strong()).clicked() { self.show_diff = !self.show_diff; }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let deploy_btn = ui.add(egui::Button::new(
                            egui::RichText::new("DEPLOY OPERATIONS").color(egui::Color32::from_rgb(0, 66, 37)).size(12.0).strong()
                        ).fill(egui::Color32::from_rgb(0, 186, 113)));
                        if deploy_btn.clicked() { self.save_file(); }

                        ui.add_space(16.0);

                        let read_btn = ui.add(egui::Button::new(
                            egui::RichText::new("OPERATIONAL READINESS").color(egui::Color32::from_rgb(226, 226, 232)).size(12.0).strong()
                        ).fill(egui::Color32::from_rgb(40, 42, 46)).stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(134, 148, 138, 50))));
                        if read_btn.clicked() { self.run_validation(); }
                    });
                });
            });

        // Left Navigation Sidebar
        egui::SidePanel::left("left_nav_sidebar")
            .frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(17, 19, 23, 240)).inner_margin(egui::Margin::symmetric(0.0, 40.0)))
            .exact_width(64.0).resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    if ui.add_sized(egui::vec2(48.0, 48.0), egui::SelectableLabel::new(self.view_mode == AppView::Canvas, "🌐\nNAV")).on_hover_text("Terminal").clicked() { self.view_mode = AppView::Canvas; }
                    ui.add_space(16.0);
                    if ui.add_sized(egui::vec2(48.0, 48.0), egui::SelectableLabel::new(self.show_editor, "✏️\nEDIT")).on_hover_text("Editor").clicked() { self.show_editor = !self.show_editor; }
                    ui.add_space(16.0);
                    if ui.add_sized(egui::vec2(48.0, 48.0), egui::SelectableLabel::new(self.show_validation, "⚠️\nVAL")).on_hover_text("Validation").clicked() { self.show_validation = !self.show_validation; }
                });
            });
    }

"""
    content = pre_content + new_update + post_content

# Remove generic headings from inner panels so layout looks like tactical overlays
content = content.replace('ui.heading("🪟 FocusFlow");', '// ui.heading removed')
content = content.replace('ui.heading(if creating_new { "✨ New Focus" } else { "✏️ Edit Focus" });', '// edit heading removed')
content = content.replace('ui.heading("Validación");', '// validation heading removed')
content = content.replace('ui.separator();', 'ui.add_space(8.0); // separator removed')

# 3. Canvas style updating (dotted lines, styling nodes)
# We find: `painter.line_segment([prereq_pos, screen_pos]`
old_line = """                    painter.line_segment(
                        [prereq_pos, screen_pos],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
                    );"""
new_line = """                    // Tactical dotted line
                    let steps = 15;
                    for i in 0..steps {
                        if i % 2 == 0 {
                            let t1 = i as f32 / steps as f32;
                            let t2 = (i + 1) as f32 / steps as f32;
                            painter.line_segment(
                                [screen_pos.lerp(prereq_pos, t1), screen_pos.lerp(prereq_pos, t2)],
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 227, 139)), // primary color dashed
                            );
                        }
                    }"""
content = content.replace(old_line, new_line)

old_line_2 = """                    painter.line_segment(
                        [tip, tip - dir * arrow_size + perp],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
                    );
                    painter.line_segment(
                        [tip, tip - dir * arrow_size - perp],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
                    );"""
new_line_2 = """                    painter.line_segment(
                        [tip, tip - dir * arrow_size + perp],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 227, 139)),
                    );
                    painter.line_segment(
                        [tip, tip - dir * arrow_size - perp],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 227, 139)),
                    );"""
content = content.replace(old_line_2, new_line_2)

old_me_line = """egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 80, 80)),"""
new_me_line = """egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 180, 171)), // error dashed"""
content = content.replace(old_me_line, new_me_line)

old_node = """            let fill_color = if selected {
                color.linear_multiply(1.5)
            } else {
                color.linear_multiply(0.6)
            };
            
            painter.rect_filled(node_rect, 4.0, fill_color);
            painter.rect_stroke(node_rect, 4.0, egui::Stroke::new(1.5, egui::Color32::WHITE), egui::StrokeKind::Inside);"""

new_node = """            let fill_color = egui::Color32::from_rgb(30, 32, 36);
            let stroke_color = if selected { egui::Color32::from_rgb(0, 227, 139) } else { egui::Color32::from_rgb(134, 148, 138) };
            
            if selected {
                painter.circle_filled(node_rect.center(), node_rect.width() * 0.6, egui::Color32::from_rgba_unmultiplied(0, 227, 139, 30));
            }
            
            painter.rect_filled(node_rect, 6.0, fill_color);
            painter.rect_stroke(node_rect, 6.0, egui::Stroke::new(1.5, stroke_color), egui::StrokeKind::Inside);"""
content = content.replace(old_node, new_node)

with open(file_path, "w", encoding="utf-8") as f:
    f.write(content)

print("Successfully patched app.rs with Tactical Editor UI")
