use robs_core::traits::*;
use robs_core::*;
use anyhow::Result;
use async_trait::async_trait;
use std::any::Any;
use parking_lot::RwLock;

pub struct WindowsCaptureSource {
    id: SourceId,
    name: String,
    window_name: Option<String>,
    window_class: Option<String>,
    active: bool,
    video_info: VideoInfo,
}

impl WindowsCaptureSource {
    pub fn new(name: String) -> Self {
        Self {
            id: SourceId(ObjectId::new()),
            name,
            window_name: None,
            window_class: None,
            active: false,
            video_info: VideoInfo {
                width: 1920,
                height: 1080,
                fps_num: 60,
                fps_den: 1,
                format: PixelFormat::BGRA,
                range: VideoRange::Full,
                color_space: ColorSpace::SRGB,
            },
        }
    }
    
    pub fn set_window(&mut self, window_name: Option<String>, window_class: Option<String>) {
        self.window_name = window_name;
        self.window_class = window_class;
    }
    
    fn capture_frame(&self) -> Option<VideoFrame> {
        let mut frame = VideoFrame::new(self.video_info.width, self.video_info.height, self.video_info.format);
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;
        frame.pts = now;
        frame.duration = (1_000_000.0 / self.video_info.fps()) as i64;
        
        Some(frame)
    }
}

#[async_trait]
impl Source for WindowsCaptureSource {
    fn id(&self) -> SourceId { self.id }
    fn name(&self) -> &str { &self.name }
    fn set_name(&mut self, name: String) { self.name = name; }
    fn get_video_info(&self) -> Option<VideoInfo> { Some(self.video_info) }
    fn get_audio_info(&self) -> Option<AudioInfo> { None }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    
    async fn activate(&mut self) -> Result<()> {
        self.active = true;
        println!("[WindowCapture] Activated: {:?}", self.window_name);
        Ok(())
    }
    
    async fn deactivate(&mut self) -> Result<()> {
        self.active = false;
        println!("[WindowCapture] Deactivated");
        Ok(())
    }
    
    fn is_active(&self) -> bool { self.active }
    
    fn properties_definition(&self) -> Vec<PropertyDef> {
        vec![
            PropertyDef {
                name: "window".into(),
                display_name: "Window".into(),
                description: "Window to capture".into(),
                type_: PropertyType::Enum,
                default: PropertyValue::Enum(String::new()),
                enum_values: vec![], // Would be populated with window list
                ..Default::default()
            },
        ]
    }
    
    fn get_property(&self,name:&str) -> Option<PropertyValue> {
        match name {
            "window" => self.window_name.clone().map(PropertyValue::Enum),
            _ => None,
        }
    }
    
    fn set_property(&mut self, name: &str, value: PropertyValue) -> Result<()> {
        match name {
            "window" => {
                if let PropertyValue::Enum(w) = value {
                    self.window_name = Some(w);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait]
impl VideoSource for WindowsCaptureSource {
    async fn get_frame(&mut self) -> Result<Option<VideoFrame>> {
        if self.active {
            Ok(self.capture_frame())
        } else {
            Ok(None)
        }
    }
}

pub struct MonitorCaptureSource {
    id: SourceId,
    name: String,
    monitor_index: u32,
    active: bool,
    video_info: VideoInfo,
}

impl MonitorCaptureSource {
    pub fn new(name: String) -> Self {
        Self {
            id: SourceId(ObjectId::new()),
            name,
            monitor_index: 0,
            active: false,
            video_info: VideoInfo {
                width: 1920,
                height: 1080,
                fps_num: 60,
                fps_den: 1,
                format: PixelFormat::BGRA,
                range: VideoRange::Full,
                color_space: ColorSpace::SRGB,
            },
        }
    }
}

#[async_trait]
impl Source for MonitorCaptureSource {
    fn id(&self) -> SourceId { self.id }
    fn name(&self) -> &str { &self.name }
    fn set_name(&mut self, name: String) { self.name = name; }
    fn get_video_info(&self) -> Option<VideoInfo> { Some(self.video_info) }
    fn get_audio_info(&self) -> Option<AudioInfo> { None }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    
    async fn activate(&mut self) -> Result<()> {
        self.active = true;
        println!("[MonitorCapture] Activated: Monitor {}", self.monitor_index);
        Ok(())
    }
    
    async fn deactivate(&mut self) -> Result<()> {
        self.active = false;
        Ok(())
    }
    
    fn is_active(&self) -> bool { self.active }
    
    fn properties_definition(&self) -> Vec<PropertyDef> {
        vec![
            PropertyDef {
                name: "monitor".into(),
                display_name: "Monitor".into(),
                type_: PropertyType::Int,
                default: PropertyValue::Int(0),
                min: Some(0.0),
                max: Some(10.0),
                ..Default::default()
            },
        ]
    }
    
    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "monitor" => Some(PropertyValue::Int(self.monitor_index as i64)),
            _ => None,
        }
    }
    
    fn set_property(&mut self, name: &str, value: PropertyValue) -> Result<()> {
        match name {
            "monitor" => {
                if let PropertyValue::Int(m) = value {
                    self.monitor_index = m as u32;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait]
impl VideoSource for MonitorCaptureSource {
    async fn get_frame(&mut self) -> Result<Option<VideoFrame>> {
        if self.active {
            let mut frame = VideoFrame::new(self.video_info.width, self.video_info.height, self.video_info.format);
            frame.pts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as i64;
            Ok(Some(frame))
        } else {
            Ok(None)
        }
    }
}

pub struct GameCaptureSource {
    id: SourceId,
    name: String,
    process_name: Option<String>,
    active: bool,
    video_info: VideoInfo,
    use_hook: bool,
}

impl GameCaptureSource {
    pub fn new(name: String) -> Self {
        Self {
            id: SourceId(ObjectId::new()),
            name,
            process_name: None,
            active: false,
            video_info: VideoInfo {
                width: 1920,
                height: 1080,
                fps_num: 60,
                fps_den: 1,
                format: PixelFormat::NV12,
                range: VideoRange::Full,
                color_space: ColorSpace::SRGB,
            },
            use_hook: true,
        }
    }
}

#[async_trait]
impl Source for GameCaptureSource {
    fn id(&self) -> SourceId { self.id }
    fn name(&self) -> &str { &self.name }
    fn set_name(&mut self, name: String) { self.name = name; }
    fn get_video_info(&self) -> Option<VideoInfo> { Some(self.video_info) }
    fn get_audio_info(&self) -> Option<AudioInfo> { None }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    
    async fn activate(&mut self) -> Result<()> {
        self.active = true;
        println!("[GameCapture] Activated for: {:?}", self.process_name);
        Ok(())
    }
    
    async fn deactivate(&mut self) -> Result<()> {
        self.active = false;
        Ok(())
    }
    
    fn is_active(&self) -> bool { self.active }
    
    fn properties_definition(&self) -> Vec<PropertyDef> {
        vec![
            PropertyDef {
                name: "process".into(),
                display_name: "Process Name".into(),
                type_: PropertyType::String,
                default: PropertyValue::String(String::new()),
                ..Default::default()
            },
        ]
    }
    
    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "process" => self.process_name.clone().map(PropertyValue::String),
            _ => None,
        }
    }
    
    fn set_property(&mut self, name: &str, value: PropertyValue) -> Result<()> {
        match name {
            "process" => {
                if let PropertyValue::String(p) = value {
                    self.process_name = Some(p);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait]
impl VideoSource for GameCaptureSource {
    async fn get_frame(&mut self) -> Result<Option<VideoFrame>> {
        if self.active {
            let mut frame = VideoFrame::new(self.video_info.width, self.video_info.height, self.video_info.format);
            frame.pts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as i64;
            Ok(Some(frame))
        } else {
            Ok(None)
        }
    }
}