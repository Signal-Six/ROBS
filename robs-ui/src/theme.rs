pub struct DarkTheme;

impl DarkTheme {
    pub fn background() -> egui::Color32 {
        egui::Color32::from_rgb(30, 30, 30)
    }

    pub fn panel_bg() -> egui::Color32 {
        egui::Color32::from_rgb(43, 43, 43)
    }

    pub fn button_bg() -> egui::Color32 {
        egui::Color32::from_rgb(60, 60, 60)
    }

    pub fn button_hover() -> egui::Color32 {
        egui::Color32::from_rgb(80, 80, 80)
    }

    pub fn text() -> egui::Color32 {
        egui::Color32::from_rgb(200, 200, 200)
    }

    pub fn text_dim() -> egui::Color32 {
        egui::Color32::from_rgb(140, 140, 140)
    }

    pub fn accent() -> egui::Color32 {
        egui::Color32::from_rgb(94, 74, 192)
    }

    pub fn live_red() -> egui::Color32 {
        egui::Color32::from_rgb(227, 64, 64)
    }

    pub fn success_green() -> egui::Color32 {
        egui::Color32::from_rgb(64, 180, 64)
    }

    pub fn warning_orange() -> egui::Color32 {
        egui::Color32::from_rgb(220, 140, 20)
    }

    pub fn apply_to(_ctx: &egui::Context) {}
}
