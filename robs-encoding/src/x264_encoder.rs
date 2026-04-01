use robs_core::traits::*;
use robs_core::*;
use anyhow::Result;
use async_trait::async_trait;
use std::any::Any;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct X264Encoder {
    id: EncoderId,
    name: String,
    preset: u32,
    bitrate: u32,
    keyint: u32,
    rate_control: RateControlMode,
    caps: EncoderCapabilities,
    initialized: bool,
    input_info: Option<VideoInfo>,
}

impl X264Encoder {
    pub fn new() -> Self {
        Self {
            id: EncoderId(ObjectId::new()),
            name: "x264".to_string(),
            preset: 3,
            bitrate: 6000,
            keyint: 2,
            rate_control: RateControlMode::CBR,
            caps: EncoderCapabilities {
                max_width: 4096,
                max_height: 2160,
                max_fps: 120,
                supports_hardware: false,
                supported_pixel_formats: vec![
                    PixelFormat::NV12,
                    PixelFormat::I420,
                    PixelFormat::I444,
                ],
            },
            initialized: false,
            input_info: None,
        }
    }
    
    fn preset_name(&self) -> &str {
        match self.preset {
            0 => "ultrafast",
            1 => "superfast",
            2 => "veryfast",
            3 => "faster",
            4 => "fast",
            5 => "medium",
            6 => "slow",
            7 => "slower",
            8 => "veryslow",
            _ => "medium",
        }
    }
}

impl Default for X264Encoder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Encoder for X264Encoder {
    fn id(&self) -> EncoderId { self.id }
    fn name(&self) -> &str { &self.name }
    fn codec_name(&self) -> &str { "h264" }
    fn media_type(&self) -> MediaType { MediaType::Video }
    
    fn get_input_info(&self) -> Option<MediaInfo> {
        self.input_info.as_ref().map(|v| MediaInfo::Video(VideoEncodeInfo {
            video: v.clone(),
            encoder_name: "x264".to_string(),
            codec_params: serde_json::json!({}),
        }))
    }
    
    fn get_output_info(&self) -> Option<MediaInfo> {
        None
    }
    
    fn get_caps(&self) -> EncoderCapabilities { self.caps.clone() }
    
    fn get_presets(&self) -> Vec<EncoderPreset> {
        vec![
            EncoderPreset { id: 0, name: "ultrafast".into(), description: "Fastest encoding, lower quality".into(), quality_level: 1, speed_level: 9 },
            EncoderPreset { id: 1, name: "superfast".into(), description: "Very fast encoding".into(), quality_level: 2, speed_level: 8 },
            EncoderPreset { id: 2, name: "veryfast".into(), description: "Fast encoding".into(), quality_level: 3, speed_level: 7 },
            EncoderPreset { id: 3, name: "faster".into(), description: "Faster encoding".into(), quality_level: 4, speed_level: 6 },
            EncoderPreset { id: 4, name: "fast".into(), description: "Quick encoding".into(), quality_level: 5, speed_level: 5 },
            EncoderPreset { id: 5, name: "medium".into(), description: "Balancedencoding and quality".into(), quality_level: 6, speed_level: 5 },
            EncoderPreset { id: 6, name: "slow".into(), description: "Slower encoding, better quality".into(), quality_level: 7, speed_level: 3 },
            EncoderPreset { id: 7, name: "slower".into(), description: "Much slower encoding, high quality".into(), quality_level: 8, speed_level: 2 },
            EncoderPreset { id: 8, name: "veryslow".into(), description: "Slowest encoding, best quality".into(), quality_level: 9, speed_level: 1 },
        ]
    }
    
    fn get_current_preset(&self) -> EncoderPreset {
        self.get_presets().into_iter()
            .find(|p| p.id == self.preset)
            .unwrap_or(EncoderPreset { id: 5, name: "medium".into(), description: "Balanced".into(), quality_level: 6, speed_level: 5 })
    }
    
    fn set_preset(&mut self, preset: EncoderPreset) -> Result<()> {
        self.preset = preset.id;
        Ok(())
    }
    
    fn parameters_definition(&self) -> Vec<PropertyDef> {
        vec![
            PropertyDef {
                name: "bitrate".into(),
                display_name: "Bitrate (kbps)".into(),
                type_: PropertyType::Int,
                default: PropertyValue::Int(6000),
                min: Some(100.0),
                max: Some(50000.0),
                ..Default::default()
            },
            PropertyDef {
                name: "keyint".into(),
                display_name: "Keyframe Interval (seconds)".into(),
                type_: PropertyType::Int,
                default: PropertyValue::Int(2),
                min: Some(1.0),
                max: Some(20.0),
                ..Default::default()
            },
            PropertyDef {
                name: "rate_control".into(),
                display_name: "Rate Control".into(),
                type_: PropertyType::Enum,
                default: PropertyValue::Enum("CBR".into()),
                enum_values: vec![
                    ("CBR".into(), "CBR".into()),
                    ("VBR".into(), "VBR".into()),
                    ("CRF".into(), "CRF".into()),
                ],
                ..Default::default()
            },
        ]
    }
    
    fn get_parameter(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "bitrate" => Some(PropertyValue::Int(self.bitrate as i64)),
            "keyint" => Some(PropertyValue::Int(self.keyint as i64)),
            "rate_control" => Some(PropertyValue::Enum(format!("{:?}", self.rate_control))),
            _ => None,
        }
    }
    
    fn set_parameter(&mut self, name: &str, value: PropertyValue) -> Result<()> {
        match name {
            "bitrate" => {
                if let PropertyValue::Int(b) = value {
                    self.bitrate = b as u32;
                }
            }
            "keyint" => {
                if let PropertyValue::Int(k) = value {
                    self.keyint = k as u32;
                }
            }
            "rate_control" => {
                if let PropertyValue::Enum(rc) = value {
                    self.rate_control = match rc.as_str() {
                        "CBR" => RateControlMode::CBR,
                        "VBR" => RateControlMode::VBR,
                        "CRF" => RateControlMode::CRF,
                        _ => RateControlMode::CBR,
                    };
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    async fn initialize(&mut self, input: MediaInfo, _output: MediaInfo) -> Result<()> {
        if let MediaInfo::Video(v) = input {
            self.input_info = Some(v.video);
            
            let encoded_header = vec![0u8; 32];
            
            println!("[x264] Initializing encoder:");
            println!("  Resolution: {}x{}", self.input_info.as_ref().unwrap().width, self.input_info.as_ref().unwrap().height);
            println!("  Preset: {}", self.preset_name());
            println!("  Bitrate: {}kbps", self.bitrate);
            println!("  Rate Control: {:?}", self.rate_control);
            
            self.initialized = true;
        }
        Ok(())
    }
    
    async fn encode(&mut self, input: MediaData) -> Result<Option<EncodedPacket>> {
        if !self.initialized {
            return Ok(None);
        }
        
        if let MediaData::Video(frame) = input {
            let packet = EncodedPacket {
                data: vec![0u8; (self.bitrate / 8 / 30) as usize],
                pts: frame.pts,
                dts: frame.pts,
                duration: frame.duration,
                keyframe: frame.pts == 0,
                track: TrackId(0),
            };
            
            Ok(Some(packet))
        } else {
            Ok(None)
        }
    }
    
    async fn flush(&mut self) -> Result<Vec<EncodedPacket>> {
        self.initialized = false;
        Ok(vec![])
    }
    
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub fn create_x264_encoder() -> Box<dyn Encoder> {
    Box::new(X264Encoder::new())
}