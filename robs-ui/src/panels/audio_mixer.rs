use eframe::egui;

#[derive(Debug, Clone)]
pub struct AudioChannel {
    pub name: String,
    pub volume: f32,
    pub muted: bool,
    pub db: f32,
    pub mono_mode: bool,
    pub monitoring: bool,
}

impl Default for AudioChannel {
    fn default() -> Self {
        Self {
            name: String::new(),
            volume: 1.0,
            muted: false,
            db: 0.0,
            mono_mode: false,
            monitoring: false,
        }
    }
}

impl AudioChannel {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

pub fn audio_mixer_panel(ui: &mut egui::Ui, channels: &mut [AudioChannel]) {
    for channel in channels.iter_mut() {
        ui.horizontal(|ui| {
            ui.set_min_width(200.0);
            
            ui.push_id(&channel.name, |ui| {
                ui.checkbox(&mut channel.muted, "Mute");
            });
        });
        
        ui.add(
            egui::Slider::new(&mut channel.volume, 0.0..=1.0).show_value(false)
        );
        
        let db = if channel.volume > 0.0 {
            20.0 * channel.volume.log10()
        } else {
            -f32::INFINITY
        };
        
        let db_text = if db.is_finite() {
            format!("{:.1} dB", db)
        } else {
            "-∞ dB".to_string()
        };
        
        ui.label(db_text);
        
        ui.separator();
    }
}

pub fn volume_meter(ui: &mut egui::Ui, level: f32, peak: f32) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter();
    
    painter.rect_filled(rect, 0.0, egui::Color32::from_gray(40));
    
    let level_width = rect.width()* level.min(1.0);
    let level_rect = egui::Rect::from_min_size(
        rect.min,
        egui::vec2(level_width, rect.height()),
    );
    
    let color = if level > 0.9 {
        egui::Color32::RED
    } else if level > 0.7 {
        egui::Color32::YELLOW
    } else {
        egui::Color32::GREEN
    };
    
    painter.rect_filled(level_rect, 0.0, color.gamma_multiply(0.7));
    
    let peak_x = rect.min.x + rect.width() * peak.min(1.0);
    painter.line_segment(
        [egui::pos2(peak_x, rect.min.y), egui::pos2(peak_x, rect.max.y)],
        egui::Stroke::new(2.0, egui::Color32::WHITE),
    );
}