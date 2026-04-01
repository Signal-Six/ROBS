use robs_core::traits::*;
use robs_core::*;
use anyhow::Result;
use async_trait::async_trait;
use std::any::Any;

pub struct SilenceSource {
    id: SourceId,
    name: String,
    audio_info: AudioInfo,
    active: bool,
    samples_generated: u64,
}

impl SilenceSource {
    pub fn new(name: String) -> Self {
        Self {
            id: SourceId(ObjectId::new()),
            name,
            audio_info: AudioInfo::default(),
            active: false,
            samples_generated: 0,
        }
    }
}

#[async_trait]
impl Source for SilenceSource {
    fn id(&self) -> SourceId { self.id }
    fn name(&self) -> &str { &self.name }
    fn set_name(&mut self, name: String) { self.name = name; }
    fn get_video_info(&self) -> Option<VideoInfo> { None }
    fn get_audio_info(&self) -> Option<AudioInfo> { Some(self.audio_info.clone()) }
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
    
    fn properties_definition(&self) -> Vec<PropertyDef> { vec![] }
    fn get_property(&self, _name: &str) -> Option<PropertyValue> { None }
    fn set_property(&mut self, _name: &str, _value: PropertyValue) -> Result<()> { Ok(()) }
}

#[async_trait]
impl AudioSource for SilenceSource {
    async fn get_audio(&mut self, frames: u32) -> Result<Option<AudioFrame>> {
        if self.active {
            self.samples_generated += frames as u64;
            Ok(Some(AudioFrame::new(frames, &self.audio_info)))
        } else {
            Ok(None)
        }
    }
}