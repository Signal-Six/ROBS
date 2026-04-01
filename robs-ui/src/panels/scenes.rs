use eframe::egui;

pub fn scenes_panel(ui: &mut egui::Ui, scenes: &[String], selected: usize, on_select: impl FnOnce(usize), on_add: impl FnOnce(), on_remove: impl FnOnce(usize), on_duplicate: impl FnOnce(usize)) {
    ui.horizontal(|ui| {
        if ui.add(egui::Button::new("➕ Add")).clicked() {
            on_add();
        }
        if ui.add(egui::Button::new("➖")).clicked() {
            on_remove(selected);
        }
        if ui.add(egui::Button::new("⧉")).clicked() {
            on_duplicate(selected);
        }
    });
    
    ui.separator();
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (i, scene) in scenes.iter().enumerate() {
            let is_selected = i == selected;
            if ui.selectable_label(is_selected, scene).clicked() {
                on_select(i);
            }
        }
    });
}

pub fn scene_context_menu(ui: &mut egui::Ui, on_rename: impl FnOnce(), on_duplicate: impl FnOnce(), on_remove: impl FnOnce(), on_move_up: impl FnOnce(), on_move_down: impl FnOnce()) {
    ui.menu_button("Scene", |ui| {
        if ui.button("Rename").clicked() { on_rename(); ui.close_menu(); }
        if ui.button("Duplicate").clicked() { on_duplicate(); ui.close_menu(); }
        if ui.button("Remove").clicked() { on_remove(); ui.close_menu(); }
        ui.separator();
        if ui.button("Move Up").clicked() { on_move_up(); ui.close_menu(); }
        if ui.button("Move Down").clicked() { on_move_down(); ui.close_menu(); }
    });
}