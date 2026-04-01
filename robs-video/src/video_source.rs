use robs_core::traits::*;
use robs_core::*;
use anyhow::Result;
use async_trait::async_trait;
use std::any::Any;

pub struct TestPatternSource {
    id: SourceId,
    name: String,
    video_info: VideoInfo,
    active: bool,
    frame_count: u64,
}

impl TestPatternSource {
    pub fn new(name: String) -> Self {
        Self {
            id: SourceId(ObjectId::new()),
            name,
            video_info: VideoInfo::default(),
            active: false,
            frame_count: 0,
        }
    }
    
    fn generate_test_frame(&mut self) -> VideoFrame {
        let mut frame = VideoFrame::new(
            self.video_info.width,
            self.video_info.height,
            self.video_info.format,
        );
        
        for y in 0..self.video_info.height as usize {
            for x in 0..self.video_info.width as usize {
                let offset = (y * frame.linesize[0] + x * 4) as usize;
                if offset + 3 < frame.data.len() {
                    let t = self.frame_count as f32 / 60.0;
                    frame.data[offset] = ((x as f32 * 0.2 + t * 50.0) as u8).wrapping_add(100);
                    frame.data[offset + 1] = ((y as f32 * 0.2 + t * 30.0) as u8).wrapping_add(80);
                    frame.data[offset + 2] = ((x as f32 * 0.1 + y as f32 * 0.1 + t * 20.0) as u8).wrapping_add(120);
                    frame.data[offset + 3] = 255;
                }
            }
        }
        
        self.frame_count += 1;
        frame
    }
}

#[async_trait]
impl Source for TestPatternSource {
    fn id(&self) -> SourceId { self.id }
    fn name(&self) -> &str { &self.name }
    fn set_name(&mut self, name: String) { self.name = name; }
    fn get_video_info(&self) -> Option<VideoInfo> { Some(self.video_info) }
    fn get_audio_info(&self) -> Option<AudioInfo> { None }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    
    async fn activate(&mut self) -> Result<()> {
        self.active = true;
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
                name: "width".to_string(),
                display_name: "Width".to_string(),
                type_: PropertyType::Int,
                default: PropertyValue::Int(1920),
                min: Some(1.0),
                max: Some(4096.0),
                ..Default::default()
            },
            PropertyDef {
                name: "height".to_string(),
                display_name: "Height".to_string(),
                type_: PropertyType::Int,
                default: PropertyValue::Int(1080),
                min: Some(1.0),
                max: Some(2160.0),
                ..Default::default()
            },
        ]
    }
    
    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "width" => Some(PropertyValue::Int(self.video_info.width as i64)),
            "height" => Some(PropertyValue::Int(self.video_info.height as i64)),
            _ => None,
        }
    }
    
    fn set_property(&mut self, name: &str, value: PropertyValue) -> Result<()> {
        match name {
            "width" => {
                if let PropertyValue::Int(w) = value {
                    self.video_info.width = w as u32;
                }
            }
            "height" => {
                if let PropertyValue::Int(h) = value {
                    self.video_info.height = h as u32;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait]
impl VideoSource for TestPatternSource {
    async fn get_frame(&mut self) -> Result<Option<VideoFrame>> {
        if self.active {
            Ok(Some(self.generate_test_frame()))
        } else {
            Ok(None)
        }
    }
}

pub fn create_test_pattern_source() -> Box<dyn VideoSource> {
    Box::new(TestPatternSource::new("Test Pattern".to_string()))
}