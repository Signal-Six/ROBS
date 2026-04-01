use eframe::egui;
use robs_chat::message::{UnifiedChatMessage, ChatPlatform};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct ChatPanel {
    messages: Arc<RwLock<Vec<UnifiedChatMessage>>>,
    input: String,
    scroll_to_bottom: bool,
    platform_filter: Option<ChatPlatform>,
}

impl ChatPanel {
    pub fn new(messages: Arc<RwLock<Vec<UnifiedChatMessage>>>) -> Self {
        Self {
            messages,
            input: String::new(),
            scroll_to_bottom: true,
            platform_filter: None,
        }
    }
    
    pub fn set_platform_filter(&mut self,platform: Option<ChatPlatform>) {
        self.platform_filter = platform;
    }
}

pub fn chat_panel(ui: &mut egui::Ui, panel: &mut ChatPanel) {
    let messages = panel.messages.read();
    
    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
        let filtered: Vec<_> = messages.iter()
            .filter(|m| {
                panel.platform_filter.map_or(true, |p| m.platform == p)
            })
            .collect();
        
        if filtered.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("No chat messages").color(egui::Color32::GRAY));
            });
        } else {
            for msg in filtered {
                ui.horizontal(|ui| {
                    let platform_color = egui::Color32::from_hex(msg.platform.color_hex()).unwrap_or(egui::Color32::GRAY);
                    
                    ui.label(
                        egui::RichText::new(format!("[{}]", msg.platform.display_name()))
                            .color(platform_color)
                            .size(10.0)
                    );
                    
                    let user_color = msg.user.color.as_ref()
                        .and_then(|c| egui::Color32::from_hex(c).ok())
                        .unwrap_or(egui::Color32::WHITE);
                    
                    ui.label(
                        egui::RichText::new(&msg.user.display_name)
                            .color(user_color)
                            .strong()
                    );
                    
                    ui.label(&msg.content);
                });
            }
        }
    });
    
    ui.separator();
    
    ui.horizontal(|ui| {
        ui.menu_button("🌐", |ui| {
            if ui.button("All Platforms").clicked() {
                panel.platform_filter = None;
                ui.close_menu();
            }
            if ui.button("Twitch").clicked() {
                panel.platform_filter = Some(ChatPlatform::Twitch);
                ui.close_menu();
            }
            if ui.button("YouTube").clicked() {
                panel.platform_filter = Some(ChatPlatform::YouTube);
                ui.close_menu();
            }
        });
        
        ui.add(
            egui::TextEdit::singleline(&mut panel.input)
                .hint_text("Send a message...")
        );
        
        if ui.add(egui::Button::new("Send")).clicked() && !panel.input.is_empty() {
            panel.input.clear();
        }
    });
}

pub fn format_timestamp(msg: &UnifiedChatMessage) -> String {
    let now = chrono::Utc::now();
    let diff = now - msg.timestamp;
    
    if diff.num_seconds() < 60 {
        format!("{}s", diff.num_seconds())
    } else if diff.num_minutes() < 60 {
        format!("{}m", diff.num_minutes())
    } else {
        format!("{}h", diff.num_hours())
    }
}