use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub general: GeneralSettings,
    pub video: VideoSettings,
    pub audio: AudioSettings,
    pub hotkeys: Vec<HotkeyBinding>,
    pub ui: UiSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
            video: VideoSettings::default(),
            audio: AudioSettings::default(),
            hotkeys: Vec::new(),
            ui: UiSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub language: String,
    pub theme: String,
    pub check_for_updates: bool,
    pub confirm_on_exit: bool,
    pub minimize_to_tray: bool,
    pub always_on_top: bool,
    pub recording_prefix: String,
    pub recording_suffix: String,
    pub replay_buffer_prefix: String,
    pub replay_buffer_suffix: String,
    pub filename_formatting: String,
    pub overwrite_confirm: bool,
    pub auto_replay_buffer: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: "en".into(),
            theme: "dark".into(),
            check_for_updates: true,
            confirm_on_exit: true,
            minimize_to_tray: false,
            always_on_top: false,
            recording_prefix: "".into(),
            recording_suffix: "".into(),
            replay_buffer_prefix: "".into(),
            replay_buffer_suffix: "".into(),
            filename_formatting: "%CCYY-%MM-%DD %hh-%mm-%ss".into(),
            overwrite_confirm: true,
            auto_replay_buffer: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSettings {
    pub adapter: u32,
    pub vsync: bool,
    pub fps: u32,
    pub base_resolution: (u32, u32),
    pub output_resolution: (u32, u32),
    pub downscale_filter: String,
    pub disable_audio_monitoring: bool,
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            adapter: 0,
            vsync: true,
            fps: 30,
            base_resolution: (1920, 1080),
            output_resolution: (1280, 720),
            downscale_filter: "bilinear".into(),
            disable_audio_monitoring: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub monitoring_device: String,
    pub monitoring_device_name: String,
    pub disable_audio_ducking: bool,
    pub suppress_warning: bool,
    pub sample_rate: u32,
    pub channel_setup: String,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            monitoring_device: "default".into(),
            monitoring_device_name: "Default".into(),
            disable_audio_ducking: false,
            suppress_warning: false,
            sample_rate: 48000,
            channel_setup: "Stereo".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    pub layout: String,
    pub preview_enabled: bool,
    pub preview_scaling: String,
    pub dock_layout: DockLayout,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            layout: "default".into(),
            preview_enabled: true,
            preview_scaling: "fit".into(),
            dock_layout: DockLayout::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockLayout {
    pub docks: Vec<DockNode>,
}

impl Default for DockLayout {
    fn default() -> Self {
        Self {
            docks: vec![
                DockNode::pane("Sources", 0.0, 0.0, 0.25, 0.4),
                DockNode::pane("Scenes", 0.0, 0.4, 0.25, 0.3),
                DockNode::pane("Controls", 0.75, 0.7, 0.25, 0.3),
                DockNode::pane("Chat", 0.75, 0.0, 0.25, 0.7),
                DockNode::tabbed(vec!["Audio Mixer", "Chat"], 0.75, 0.0, 0.25, 0.7),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockNode {
    pub kind: DockKind,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub tabs: Vec<String>,
    pub children: Vec<DockNode>,
}

impl DockNode {
    pub fn pane(name: &str, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            kind: DockKind::Pane,
            x,
            y,
            w,
            h,
            tabs: vec![name.to_string()],
            children: Vec::new(),
        }
    }

    pub fn tabbed(names: Vec<&str>, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            kind: DockKind::Tabs,
            x,
            y,
            w,
            h,
            tabs: names.into_iter().map(String::from).collect(),
            children: Vec::new(),
        }
    }

    pub fn horizontal(children: Vec<DockNode>) -> Self {
        Self {
            kind: DockKind::Horizontal,
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            tabs: Vec::new(),
            children,
        }
    }

    pub fn vertical(children: Vec<DockNode>) -> Self {
        Self {
            kind: DockKind::Vertical,
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            tabs: Vec::new(),
            children,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DockKind {
    Horizontal,
    Vertical,
    Pane,
    Tabs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyBinding {
    pub action: String,
    pub key: String,
    pub modifiers: Vec<String>,
}

impl HotkeyBinding {
    pub fn new(action: &str, key: &str, modifiers: Vec<&str>) -> Self {
        Self {
            action: action.to_string(),
            key: key.to_string(),
            modifiers: modifiers.into_iter().map(String::from).collect(),
        }
    }
}

impl AppSettings {
    pub fn load_or_default() -> Self {
        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
