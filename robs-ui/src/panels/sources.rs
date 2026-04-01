use eframe::egui;

pub fn sources_panel(ui: &mut egui::Ui, sources: &[(&str, bool)], on_add: impl FnOnce(), on_remove: impl FnOnce(usize), on_move_up: impl FnOnce(usize), on_move_down: impl FnOnce(usize), on_toggle: impl FnOnce(usize, bool)) {
    ui.horizontal(|ui| {
        if ui.add(egui::Button::new("➕ Add")).clicked() {
            on_add();
        }
        if ui.add(egui::Button::new("↑")).clicked() {}
        if ui.add(egui::Button::new("↓")).clicked() {}
        if ui.add(egui::Button::new("➖")).clicked() {}
    });
    
    ui.separator();
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (i, (name, visible)) in sources.iter().enumerate() {
            let mut visible = *visible;
            ui.horizontal(|ui| {
                ui.checkbox(&mut visible, "");
                ui.label(*name);
            });
            
            if visible != sources[i].1 {
                on_toggle(i, visible);
            }
        }
    });
}

pub fn add_source_menu(ui: &mut egui::Ui) -> Option<String> {
    let mut result = None;
    
    ui.menu_button("➕ Add", |ui| {
        if ui.button("Image").clicked() { result = Some("image".into()); ui.close_menu(); }
        if ui.button("Color Source").clicked() { result = Some("color".into()); ui.close_menu(); }
        if ui.button("Browser").clicked() { result = Some("browser".into()); ui.close_menu(); }
        if ui.button("Text (GDI+)").clicked() { result = Some("text".into()); ui.close_menu(); }
        if ui.button("Media Source").clicked() { result = Some("media".into()); ui.close_menu(); }
        ui.separator();
        if ui.button("Window Capture").clicked() { result = Some("window_capture".into()); ui.close_menu(); }
        if ui.button("Game Capture").clicked() { result = Some("game_capture".into()); ui.close_menu(); }
        if ui.button("Monitor Capture").clicked() { result = Some("monitor_capture".into()); ui.close_menu(); }
    });
    
    result
}