use eframe::egui;

pub struct StreamControls {
    pub streaming: bool,
    pub recording: bool,
    pub replay_buffer: bool,
    pub virtual_cam: bool,
}

impl Default for StreamControls {
    fn default() -> Self {
        Self {
            streaming: false,
            recording: false,
            replay_buffer: false,
            virtual_cam: false,
        }
    }
}

pub fn render_controls(ui: &mut egui::Ui, controls: &mut StreamControls) {
    ui.vertical(|ui| {
        let stream_color = if controls.streaming {
            egui::Color32::from_rgb(244, 67, 54)
        } else {
            egui::Color32::from_rgb(67, 160, 71)
        };
        
        let stream_text = if controls.streaming { "Stop Streaming" } else { "Start Streaming" };
        if ui.add(egui::Button::new(egui::RichText::new(stream_text).color(stream_color)).fill(stream_color.gamma_multiply(0.3))).clicked() {
            controls.streaming = !controls.streaming;
        }
        
        let rec_color = if controls.recording {
            egui::Color32::from_rgb(244, 67, 54)
        } else {
            egui::Color32::from_rgb(255, 152, 0)
        };
        
        let rec_text = if controls.recording { "Stop Recording" } else { "Start Recording" };
        if ui.add(egui::Button::new(egui::RichText::new(rec_text).color(rec_color)).fill(rec_color.gamma_multiply(0.3))).clicked() {
            controls.recording = !controls.recording;
        }
        
        if controls.replay_buffer {
            if ui.button("Save Replay").clicked() {}
        }
        
        ui.separator();
        
        ui.horizontal(|ui| {
            if ui.selectable_label(false, "Studio Mode").clicked() {}
        });
        
        ui.horizontal(|ui| {
            if ui.selectable_label(false, "Settings").clicked() {}
        });
        
        ui.horizontal(|ui| {
            if ui.selectable_label(false, "Exit").clicked() {}
        });
    });
}

pub fn streaming_status(ui: &mut egui::Ui, is_streaming: bool, time_code: Option<&str>, bitrate: Option<u32>) {
    ui.horizontal(|ui| {
        if is_streaming {
            ui.label(egui::RichText::new("●LIVE").color(egui::Color32::RED));
            if let Some(tc) = time_code {
                ui.label(tc);
            }
            if let Some(br) = bitrate {
                ui.label(format!("{} kbps", br));
            }
        } else {
            ui.label(egui::RichText::new("Offline").color(egui::Color32::GRAY));
        }
    });
}