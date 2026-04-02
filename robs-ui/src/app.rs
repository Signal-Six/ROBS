use eframe::egui;
use parking_lot::RwLock;
use robs_chat::aggregator::ChatAggregator;
use robs_chat::message::{ChatEvent, UnifiedChatMessage};
use robs_encoding::detect_encoders;
use robs_outputs::FileOutput;
use robs_profiles::profile::ProfileManager;
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Clone, PartialEq, Eq, Hash)]
enum Panel {
    Preview,
    Sources,
    Scenes,
    Controls,
    AudioMixer,
    Chat,
    Stats,
}

pub struct RobsApp {
    streaming: bool,
    recording: bool,
    profile_manager: Arc<RwLock<ProfileManager>>,
    chat_messages: Arc<RwLock<VecDeque<UnifiedChatMessage>>>,
    chat_input: String,
    chat_rx: Option<mpsc::Receiver<ChatEvent>>,
    current_scene: String,
    scenes: Vec<String>,
    sources: Vec<SourceEntry>,
    show_settings: bool,
    settings_rect: Option<egui::Rect>,
    active_settings_tab: usize,
    audio_channels: Vec<AudioChannel>,
    streaming_time: u64,
    bitrate: u32,
    dropped_frames: u64,
    fps: f64,
    active_panel: Panel,
    show_preview: bool,
    show_sources: bool,
    show_scenes: bool,
    show_controls: bool,
    show_audio: bool,
    show_chat: bool,
    show_stats: bool,
    confirm_on_exit: bool,
    minimize_to_tray: bool,
    always_on_top: bool,
    check_for_updates: bool,
    filename_formatting: String,
    base_width: u32,
    base_height: u32,
    output_width: u32,
    output_height: u32,
    fps_setting: f32,
    stream_server: String,
    stream_key: String,
    stream_bitrate: u32,
    keyframe_interval: u32,
    recording_bitrate: u32,
    recording_path: String,
    recording_format: String,
    video_encoder: String,
    audio_encoder: String,
    available_video_encoders: Vec<String>,
    available_audio_encoders: Vec<String>,
    nvenc_available: bool,
    aac_available: bool,
    ffmpeg_available: bool,
    recording_file_output: Option<FileOutput>,
    recording_start_time: Option<u64>,
    last_recording_path: String,
}

#[derive(Clone)]
struct SourceEntry {
    name: String,
    visible: bool,
    source_type: String,
}

#[derive(Clone)]
struct AudioChannel {
    name: String,
    volume: f32,
    muted: bool,
}

impl RobsApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let detection = detect_encoders();

        let mut video_encoders = Vec::new();
        if detection.ffmpeg_available {
            video_encoders.push("FFmpeg x264 (Software)".into());
        }
        if detection.nvenc_available {
            video_encoders.push("NVIDIA NVENC H.264 (Hardware)".into());
        }
        if video_encoders.is_empty() {
            video_encoders.push("None Available".into());
        }

        let mut audio_encoders = Vec::new();
        if detection.aac_available {
            audio_encoders.push("FFmpeg AAC".into());
        }
        if audio_encoders.is_empty() {
            audio_encoders.push("None Available".into());
        }

        let video_encoder = if detection.nvenc_available {
            "NVIDIA NVENC H.264 (Hardware)".into()
        } else if detection.ffmpeg_available {
            "FFmpeg x264 (Software)".into()
        } else {
            "None Available".into()
        };

        let audio_encoder = if detection.aac_available {
            "FFmpeg AAC".into()
        } else {
            "None Available".into()
        };

        Self {
            streaming: false,
            recording: false,
            profile_manager: Arc::new(RwLock::new(ProfileManager::default())),
            chat_messages: Arc::new(RwLock::new(VecDeque::with_capacity(500))),
            chat_input: String::new(),
            chat_rx: None,
            current_scene: "Scene 1".to_string(),
            scenes: vec!["Scene 1".to_string()],
            sources: vec![SourceEntry {
                name: "Test Pattern".to_string(),
                visible: true,
                source_type: "test_pattern".to_string(),
            }],
            show_settings: false,
            settings_rect: None,
            audio_channels: vec![
                AudioChannel {
                    name: "Mic/Aux".into(),
                    volume: 0.8,
                    muted: false,
                },
                AudioChannel {
                    name: "Desktop Audio".into(),
                    volume: 0.6,
                    muted: false,
                },
                AudioChannel {
                    name: "Application".into(),
                    volume: 0.9,
                    muted: false,
                },
            ],
            streaming_time: 0,
            bitrate: 6000,
            dropped_frames: 0,
            fps: 30.0,
            active_panel: Panel::Preview,
            show_preview: true,
            show_sources: true,
            show_scenes: true,
            show_controls: true,
            show_audio: true,
            show_chat: true,
            show_stats: true,
            active_settings_tab: 0,
            confirm_on_exit: true,
            minimize_to_tray: false,
            always_on_top: false,
            check_for_updates: true,
            filename_formatting: "%CCYY-%MM-%DD %hh-%mm-%ss".into(),
            base_width: 1920,
            base_height: 1080,
            output_width: 1280,
            output_height: 720,
            fps_setting: 30.0,
            stream_server: "rtmp://live.twitch.tv/app".into(),
            stream_key: String::new(),
            stream_bitrate: 6000,
            keyframe_interval: 2,
            recording_bitrate: 10000,
            recording_path: String::new(),
            recording_format: "mp4".into(),
            video_encoder,
            audio_encoder,
            available_video_encoders: video_encoders,
            available_audio_encoders: audio_encoders,
            nvenc_available: detection.nvenc_available,
            aac_available: detection.aac_available,
            ffmpeg_available: detection.ffmpeg_available,
            recording_file_output: None,
            recording_start_time: None,
            last_recording_path: String::new(),
        }
    }

    pub fn with_chat(
        mut self,
        _aggregator: Arc<ChatAggregator>,
        rx: mpsc::Receiver<ChatEvent>,
    ) -> Self {
        self.chat_rx = Some(rx);
        self
    }

    fn handle_events(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &mut self.chat_rx {
            while let Ok(event) = rx.try_recv() {
                if let ChatEvent::Message(msg) = event {
                    self.chat_messages.write().push_back(*msg);
                }
            }
        }
        if self.streaming {
            self.streaming_time += 1;
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        }
        if self.recording {
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        }
    }

    fn start_recording(&mut self) {
        let path = if self.recording_path.is_empty() {
            let default_dir = std::env::var("USERPROFILE")
                .map(|p| format!("{}\\Videos", p))
                .unwrap_or_else(|_| "C:\\Users\\Videos".to_string());
            fs::create_dir_all(&default_dir).ok();
            default_dir
        } else {
            self.recording_path.clone()
        };

        let timestamp = chrono::Local::now().format("%Y-%m-%d %H-%M-%S");
        let filename = format!("ROBS_{}.{}", timestamp, self.recording_format);
        let full_path = PathBuf::from(&path).join(&filename);

        self.last_recording_path = full_path.to_string_lossy().into_owned();

        println!(
            "[Recording] Starting recording to {}",
            self.last_recording_path
        );
        self.recording = true;
        self.recording_start_time = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    fn stop_recording(&mut self) {
        self.recording = false;
        let elapsed = self.recording_start_time.map(|start| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                - start
        });

        let duration_str = elapsed.map(|s| Self::format_time(s)).unwrap_or_default();
        println!(
            "[Recording] Stopped recording ({}s) saved to {}",
            duration_str, self.last_recording_path
        );
        self.recording_start_time = None;
    }

    fn format_time(seconds: u64) -> String {
        let h = seconds / 3600;
        let m = (seconds % 3600) / 60;
        let s = seconds % 60;
        format!("{:02}:{:02}:{:02}", h, m, s)
    }

    fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Profile").clicked() {
                        let id = self.profile_manager.write().create("New Profile".into());
                        self.profile_manager.write().set_current(id).ok();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Settings").clicked() {
                        self.show_settings = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close_menu();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Redo").clicked() {
                        ui.close_menu();
                    }
                });
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_preview, "Preview");
                    ui.checkbox(&mut self.show_sources, "Sources");
                    ui.checkbox(&mut self.show_scenes, "Scenes");
                    ui.checkbox(&mut self.show_controls, "Controls");
                    ui.checkbox(&mut self.show_audio, "Audio Mixer");
                    ui.checkbox(&mut self.show_chat, "Chat");
                    ui.checkbox(&mut self.show_stats, "Stats");
                });
                ui.menu_button("Profile", |ui| {
                    let profiles = self.profile_manager.read().list();
                    for (id, name) in profiles {
                        if ui.button(&name).clicked() {
                            self.profile_manager.write().set_current(id).ok();
                            ui.close_menu();
                        }
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About ROBS").clicked() {
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn streaming_controls(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("streaming_controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let stream_text = if self.streaming {
                    "Stop Streaming"
                } else {
                    "Start Streaming"
                };
                let stream_color = if self.streaming {
                    egui::Color32::RED
                } else {
                    egui::Color32::GREEN
                };
                if ui
                    .add(egui::Button::new(
                        egui::RichText::new(stream_text).color(stream_color),
                    ))
                    .clicked()
                {
                    self.streaming = !self.streaming;
                    if self.streaming {
                        self.streaming_time = 0;
                    }
                }

                let rec_text = if self.recording {
                    "Stop Recording"
                } else {
                    "Start Recording"
                };
                let rec_color = if self.recording {
                    egui::Color32::RED
                } else {
                    egui::Color32::from_rgb(200, 100, 0)
                };
                if ui
                    .add(egui::Button::new(
                        egui::RichText::new(rec_text).color(rec_color),
                    ))
                    .clicked()
                {
                    if self.recording {
                        self.stop_recording();
                    } else {
                        self.start_recording();
                    }
                }

                ui.separator();
                ui.label(format!("Scene: {}", self.current_scene));

                if self.streaming {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("● LIVE").color(egui::Color32::RED));
                        ui.label(Self::format_time(self.streaming_time));
                        ui.label(format!("{} kbps", self.bitrate));
                    });
                }
            });
        });
    }

    fn show_settings_window(&mut self, ctx: &egui::Context) {
        let mut show_settings = self.show_settings;
        egui::Window::new("Settings")
            .id(egui::Id::new("settings_window"))
            .resizable(true)
            .open(&mut show_settings)
            .default_width(650.0)
            .default_height(450.0)
            .min_width(400.0)
            .min_height(300.0)
            .show(ctx, |ui| {
                let available_height = ui.available_height();
                ui.columns(2, |cols| {
                    let left = &mut cols[0];
                    left.set_min_width(120.0);
                    left.set_max_width(160.0);
                    for (i, tab) in [
                        "General",
                        "Video",
                        "Audio",
                        "Hotkeys",
                        "Streaming",
                        "Outputs",
                    ]
                    .iter()
                    .enumerate()
                    {
                        if left
                            .selectable_label(self.active_settings_tab == i, *tab)
                            .clicked()
                        {
                            self.active_settings_tab = i;
                        }
                    }

                    let right = &mut cols[1];
                    egui::ScrollArea::vertical()
                        .max_height(available_height)
                        .show(right, |ui| {
                            ui.set_min_width(250.0);
                            match self.active_settings_tab {
                                0 => self.settings_general(ui),
                                1 => self.settings_video(ui),
                                2 => self.settings_audio(ui),
                                3 => self.settings_hotkeys(ui),
                                4 => self.settings_streaming(ui),
                                5 => self.settings_outputs(ui),
                                _ => {}
                            }
                        });
                });
                ui.separator();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        self.show_settings = false;
                    }
                });
            });
        self.show_settings = show_settings;
    }

    fn settings_general(&mut self, ui: &mut egui::Ui) {
        ui.heading("General");
        ui.separator();
        egui::Grid::new("settings_general").show(ui, |ui| {
            ui.label("Language:");
            ui.button("English");
            ui.end_row();
            ui.label("Theme:");
            ui.button("Dark");
            ui.end_row();
            ui.label("Confirm on exit:");
            ui.checkbox(&mut self.confirm_on_exit, "");
            ui.end_row();
            ui.label("Minimize to tray:");
            ui.checkbox(&mut self.minimize_to_tray, "");
            ui.end_row();
            ui.label("Always on top:");
            ui.checkbox(&mut self.always_on_top, "");
            ui.end_row();
            ui.label("Check for updates:");
            ui.checkbox(&mut self.check_for_updates, "");
            ui.end_row();
            ui.label("Filename formatting:");
            ui.text_edit_singleline(&mut self.filename_formatting);
            ui.end_row();
        });
    }

    fn settings_video(&mut self, ui: &mut egui::Ui) {
        ui.heading("Video");
        ui.separator();
        egui::Grid::new("settings_video").show(ui, |ui| {
            ui.label("Base Resolution:");
            ui.label(format!("{}x{}", self.base_width, self.base_height));
            ui.end_row();
            ui.label("Output Resolution:");
            ui.label(format!("{}x{}", self.output_width, self.output_height));
            ui.end_row();
            ui.label("Downscale Filter:");
            ui.button("Bilinear");
            ui.end_row();
            ui.label("FPS Type:");
            ui.button("Integer");
            ui.end_row();
            ui.label("FPS:");
            ui.add(egui::Slider::new(&mut self.fps_setting, 1.0..=120.0));
            ui.end_row();
        });
    }

    fn settings_audio(&mut self, ui: &mut egui::Ui) {
        ui.heading("Audio");
        ui.separator();
        egui::Grid::new("settings_audio").show(ui, |ui| {
            ui.label("Sample Rate:");
            ui.button("48kHz");
            ui.end_row();
            ui.label("Channels:");
            ui.button("Stereo");
            ui.end_row();
            ui.label("Monitoring Device:");
            ui.button("Default");
            ui.end_row();
        });
    }

    fn settings_hotkeys(&mut self, ui: &mut egui::Ui) {
        ui.heading("Hotkeys");
        ui.separator();
        ui.label("Hotkey configuration coming soon.");
    }

    fn settings_streaming(&mut self, ui: &mut egui::Ui) {
        ui.heading("Streaming");
        ui.separator();
        egui::Grid::new("settings_streaming").show(ui, |ui| {
            ui.label("Service:");
            ui.button("Twitch");
            ui.end_row();
            ui.label("Server:");
            ui.text_edit_singleline(&mut self.stream_server);
            ui.end_row();
            ui.label("Stream Key:");
            ui.text_edit_singleline(&mut self.stream_key);
            ui.end_row();
            ui.label("Video Encoder:");
            egui::ComboBox::from_id_salt("video_encoder")
                .selected_text(&self.video_encoder)
                .show_ui(ui, |ui| {
                    for enc in &self.available_video_encoders {
                        ui.selectable_value(&mut self.video_encoder, enc.clone(), enc);
                    }
                });
            ui.end_row();
            ui.label("Audio Encoder:");
            egui::ComboBox::from_id_salt("audio_encoder")
                .selected_text(&self.audio_encoder)
                .show_ui(ui, |ui| {
                    for enc in &self.available_audio_encoders {
                        ui.selectable_value(&mut self.audio_encoder, enc.clone(), enc);
                    }
                });
            ui.end_row();
            ui.label("Bitrate:");
            ui.add(egui::Slider::new(&mut self.stream_bitrate, 1000..=20000).suffix(" kbps"));
            ui.end_row();
            ui.label("Rate Control:");
            ui.button("CBR");
            ui.end_row();
            ui.label("Keyframe Interval:");
            ui.add(egui::Slider::new(&mut self.keyframe_interval, 0..=20).suffix(" s"));
            ui.end_row();
            ui.label("Preset:");
            ui.button("faster");
            ui.end_row();
        });

        ui.separator();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Encoder Status:").strong());
            if self.ffmpeg_available {
                ui.label(egui::RichText::new("FFmpeg OK").color(egui::Color32::GREEN));
            } else {
                ui.label(egui::RichText::new("FFmpeg not found").color(egui::Color32::RED));
            }
            if self.nvenc_available {
                ui.label(egui::RichText::new("NVENC OK").color(egui::Color32::GREEN));
            } else {
                ui.label(egui::RichText::new("NVENC not available").color(egui::Color32::YELLOW));
            }
            if self.aac_available {
                ui.label(egui::RichText::new("AAC OK").color(egui::Color32::GREEN));
            } else {
                ui.label(egui::RichText::new("AAC not available").color(egui::Color32::YELLOW));
            }
        });
    }

    fn settings_outputs(&mut self, ui: &mut egui::Ui) {
        ui.heading("Outputs");
        ui.separator();
        ui.heading("Recording");
        ui.separator();
        egui::Grid::new("settings_recording").show(ui, |ui| {
            ui.label("Type:");
            ui.button("Standard");
            ui.end_row();
            ui.label("Format:");
            egui::ComboBox::from_id_salt("recording_format")
                .selected_text(&self.recording_format)
                .show_ui(ui, |ui| {
                    for fmt in ["mp4", "mkv", "flv", "mov"] {
                        ui.selectable_value(&mut self.recording_format, fmt.to_string(), fmt);
                    }
                });
            ui.end_row();
            ui.label("Encoder:");
            egui::ComboBox::from_id_salt("recording_encoder")
                .selected_text(&self.video_encoder)
                .show_ui(ui, |ui| {
                    for enc in &self.available_video_encoders {
                        ui.selectable_value(&mut self.video_encoder, enc.clone(), enc);
                    }
                });
            ui.end_row();
            ui.label("Bitrate:");
            ui.add(egui::Slider::new(&mut self.recording_bitrate, 1000..=50000).suffix(" kbps"));
            ui.end_row();
            ui.label("Path:");
            ui.horizontal(|ui| {
                let display_path = if self.recording_path.is_empty() {
                    "(not set)".to_string()
                } else {
                    self.recording_path.clone()
                };
                ui.label(display_path);
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_directory(
                            std::env::var("USERPROFILE")
                                .unwrap_or_else(|_| "C:\\Users".to_string()),
                        )
                        .pick_folder()
                    {
                        self.recording_path = path.to_string_lossy().into_owned();
                    }
                }
            });
            ui.end_row();
        });
    }

    fn sources_panel(&mut self, ctx: &egui::Context) {
        if self.show_sources {
            egui::SidePanel::left("sources_panel")
                .default_width(250.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.heading("Sources");
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("+ Add").clicked() {}
                        if ui.button("↑").clicked() {}
                        if ui.button("↓").clicked() {}
                        if ui.button("−").clicked() {}
                    });
                    ui.separator();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for source in self.sources.iter_mut() {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut source.visible, "");
                                ui.label(&source.name);
                            });
                        }
                    });
                });
        }
    }

    fn scenes_panel(&mut self, ctx: &egui::Context) {
        if self.show_scenes {
            egui::SidePanel::left("scenes_panel")
                .default_width(200.0)
                .resizable(true)
                .show_animated(ctx, true, |ui| {
                    ui.heading("Scenes");
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("+ Add").clicked() {
                            self.scenes.push(format!("Scene {}", self.scenes.len() + 1));
                        }
                        if ui.button("−").clicked() {
                            if self.scenes.len() > 1 {
                                self.scenes.pop();
                            }
                        }
                    });
                    ui.separator();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for scene in self.scenes.iter() {
                            let selected = *scene == self.current_scene;
                            if ui.selectable_label(selected, scene).clicked() {
                                self.current_scene = scene.clone();
                            }
                        }
                    });
                });
        }
    }

    fn preview_panel(&mut self, ctx: &egui::Context) {
        if self.show_preview {
            egui::CentralPanel::default().show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();
                ui.painter()
                    .rect_filled(rect, 2.0, egui::Color32::from_rgb(30, 30, 30));
                let center = rect.center();
                ui.painter().text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    "Preview",
                    egui::FontId::proportional(24.0),
                    egui::Color32::GRAY,
                );
                if self.streaming {
                    ui.painter().text(
                        rect.left_top() + egui::vec2(10.0, 10.0),
                        egui::Align2::LEFT_TOP,
                        "● LIVE",
                        egui::FontId::proportional(18.0),
                        egui::Color32::RED,
                    );
                }
            });
        }
    }

    fn right_panel(&mut self, ctx: &egui::Context) {
        if self.show_audio || self.show_chat || self.show_stats || self.show_controls {
            egui::SidePanel::right("right_panel")
                .default_width(300.0)
                .resizable(true)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if self.show_controls {
                            ui.collapsing("Controls", |ui| {
                                ui.vertical_centered(|ui| {
                                    let stream_text = if self.streaming {
                                        "Stop Streaming"
                                    } else {
                                        "Start Streaming"
                                    };
                                    let stream_color = if self.streaming {
                                        egui::Color32::RED
                                    } else {
                                        egui::Color32::GREEN
                                    };
                                    if ui
                                        .add(egui::Button::new(
                                            egui::RichText::new(stream_text).color(stream_color),
                                        ))
                                        .clicked()
                                    {
                                        self.streaming = !self.streaming;
                                        if self.streaming {
                                            self.streaming_time = 0;
                                        }
                                    }
                                    let rec_text = if self.recording {
                                        "Stop Recording"
                                    } else {
                                        "Start Recording"
                                    };
                                    let rec_color = if self.recording {
                                        egui::Color32::RED
                                    } else {
                                        egui::Color32::from_rgb(200, 100, 0)
                                    };
                                    if ui
                                        .add(egui::Button::new(
                                            egui::RichText::new(rec_text).color(rec_color),
                                        ))
                                        .clicked()
                                    {
                                        self.recording = !self.recording;
                                    }
                                    ui.separator();
                                    if ui.button("Studio Mode").clicked() {}
                                    if ui.button("Settings").clicked() {
                                        self.show_settings = true;
                                    }
                                });
                            });
                        }

                        if self.show_audio {
                            ui.collapsing("Audio Mixer", |ui| {
                                for ch in self.audio_channels.iter_mut() {
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut ch.muted, "M");
                                        ui.label(&ch.name);
                                        ui.add(
                                            egui::Slider::new(&mut ch.volume, 0.0..=1.0)
                                                .show_value(false),
                                        );
                                        ui.label(format!("{:.0}%", ch.volume * 100.0));
                                    });
                                    let vol = ch.volume;
                                    let meter_color = if vol > 0.9 {
                                        egui::Color32::RED
                                    } else if vol > 0.7 {
                                        egui::Color32::YELLOW
                                    } else {
                                        egui::Color32::GREEN
                                    };
                                    let meter_rect = ui.available_rect_before_wrap();
                                    let meter = egui::Rect::from_min_size(
                                        meter_rect.min,
                                        egui::vec2(meter_rect.width() * vol, 6.0),
                                    );
                                    ui.painter().rect_filled(
                                        meter,
                                        1.0,
                                        meter_color.gamma_multiply(0.7),
                                    );
                                    ui.add_space(10.0);
                                }
                            });
                        }

                        if self.show_chat {
                            ui.collapsing("Chat", |ui| {
                                egui::ScrollArea::vertical()
                                    .stick_to_bottom(true)
                                    .show(ui, |ui| {
                                        let messages = self.chat_messages.read();
                                        if messages.is_empty() {
                                            ui.label(
                                                egui::RichText::new("No messages")
                                                    .color(egui::Color32::GRAY),
                                            );
                                        } else {
                                            for msg in messages.iter() {
                                                ui.horizontal(|ui| {
                                                    let color = msg
                                                        .user
                                                        .color
                                                        .as_ref()
                                                        .and_then(|c| {
                                                            egui::Color32::from_hex(c).ok()
                                                        })
                                                        .unwrap_or(egui::Color32::WHITE);
                                                    ui.label(
                                                        egui::RichText::new(&msg.user.display_name)
                                                            .color(color)
                                                            .strong(),
                                                    );
                                                    ui.label(":");
                                                    ui.label(&msg.content);
                                                });
                                            }
                                        }
                                    });
                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.text_edit_singleline(&mut self.chat_input);
                                    if ui.button("Send").clicked() && !self.chat_input.is_empty() {
                                        self.chat_input.clear();
                                    }
                                });
                            });
                        }

                        if self.show_stats {
                            ui.collapsing("Stats", |ui| {
                                egui::Grid::new("stats").show(ui, |ui| {
                                    ui.label("Streaming:");
                                    ui.label(if self.streaming { "Active" } else { "Inactive" });
                                    ui.end_row();
                                    ui.label("Recording:");
                                    ui.label(if self.recording { "Active" } else { "Inactive" });
                                    ui.end_row();
                                    if self.streaming {
                                        ui.label("Duration:");
                                        ui.label(Self::format_time(self.streaming_time));
                                        ui.end_row();
                                    }
                                    ui.label("Frame Rate:");
                                    ui.label(format!("{:.1} fps", self.fps));
                                    ui.end_row();
                                    ui.label("Bitrate:");
                                    ui.label(format!("{} kbps", self.bitrate));
                                    ui.end_row();
                                    ui.label("Dropped Frames:");
                                    ui.label(format!("{}", self.dropped_frames));
                                    ui.end_row();
                                });
                            });
                        }
                    });
                });
        }
    }
}

impl eframe::App for RobsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_events(ctx);
        self.menu_bar(ctx);
        self.streaming_controls(ctx);
        self.sources_panel(ctx);
        self.scenes_panel(ctx);
        self.preview_panel(ctx);
        self.right_panel(ctx);

        if self.show_settings {
            self.show_settings_window(ctx);
        }
    }
}
