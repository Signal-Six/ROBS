use robs_core::traits::*;
use robs_core::*;
use anyhow::Result;
use async_trait::async_trait;
use std::any::Any;
use std::io::Write;
use std::process::ChildStdin;
use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::FfmpegEvent;

pub struct FfmpegH264Encoder {
    id: EncoderId,
    name: String,
    bitrate: u32,
    keyint: u32,
    rate_control: RateControlMode,
    preset: String,
    profile: String,
    caps: EncoderCapabilities,
    stdin: Option<ChildStdin>,
    child: Option<FfmpegChild>,
    frame_count: i64,
    initialized: bool,
    input_info: Option<VideoInfo>,
    width: u32,
    height: u32,
    fps_num: u32,
    fps_den: u32,
    pixel_format: &'static str,
}

impl FfmpegH264Encoder {
    pub fn new() -> Self {
        Self {
            id: EncoderId(ObjectId::new()),
            name: "FFmpeg x264".to_string(),
            bitrate: 6000,
            keyint: 2,
            rate_control: RateControlMode::CBR,
            preset: "faster".to_string(),
            profile: "high".to_string(),
            caps: EncoderCapabilities {
                max_width: 4096,
                max_height: 2160,
                max_fps: 120,
                supports_hardware: false,
                supported_pixel_formats: vec![
                    PixelFormat::NV12,
                    PixelFormat::I420,
                    PixelFormat::I422,
                    PixelFormat::I444,
                ],
            },
            stdin: None,
            child: None,
            frame_count: 0,
            initialized: false,
            input_info: None,
            width: 1920,
            height: 1080,
            fps_num: 30,
            fps_den: 1,
            pixel_format: "nv12",
        }
    }

    pub fn is_available() -> bool {
        std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn pixel_format_str(format: PixelFormat) -> &'static str {
        match format {
            PixelFormat::NV12 => "nv12",
            PixelFormat::I420 => "yuv420p",
            PixelFormat::I422 => "yuv422p",
            PixelFormat::I444 => "yuv444p",
            PixelFormat::RGBA => "rgba",
            PixelFormat::BGRA => "bgra",
            PixelFormat::Rgb24 => "rgb24",
            PixelFormat::Bgr24 => "bgr24",
            _ => "nv12",
        }
    }
}

impl Default for FfmpegH264Encoder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Encoder for FfmpegH264Encoder {
    fn id(&self) -> EncoderId { self.id }
    fn name(&self) -> &str { &self.name }
    fn codec_name(&self) -> &str { "h264" }
    fn media_type(&self) -> MediaType { MediaType::Video }

    fn get_input_info(&self) -> Option<MediaInfo> {
        self.input_info.as_ref().map(|v| MediaInfo::Video(VideoEncodeInfo {
            video: v.clone(),
            encoder_name: "libx264".to_string(),
            codec_params: serde_json::json!({}),
        }))
    }

    fn get_output_info(&self) -> Option<MediaInfo> { None }
    fn get_caps(&self) -> EncoderCapabilities { self.caps.clone() }

    fn get_presets(&self) -> Vec<EncoderPreset> {
        vec![
            EncoderPreset { id: 0, name: "ultrafast".into(), description: "Fastest, lowest quality".into(), quality_level: 1, speed_level: 9 },
            EncoderPreset { id: 1, name: "superfast".into(), description: "Very fast".into(), quality_level: 2, speed_level: 8 },
            EncoderPreset { id: 2, name: "veryfast".into(), description: "Fast".into(), quality_level: 3, speed_level: 7 },
            EncoderPreset { id: 3, name: "faster".into(), description: "Faster".into(), quality_level: 4, speed_level: 6 },
            EncoderPreset { id: 4, name: "fast".into(), description: "Quick".into(), quality_level: 5, speed_level: 5 },
            EncoderPreset { id: 5, name: "medium".into(), description: "Balanced".into(), quality_level: 6, speed_level: 5 },
            EncoderPreset { id: 6, name: "slow".into(), description: "Slower, better quality".into(), quality_level: 7, speed_level: 3 },
            EncoderPreset { id: 7, name: "slower".into(), description: "Much slower, high quality".into(), quality_level: 8, speed_level: 2 },
            EncoderPreset { id: 8, name: "veryslow".into(), description: "Slowest, best quality".into(), quality_level: 9, speed_level: 1 },
        ]
    }

    fn get_current_preset(&self) -> EncoderPreset {
        self.get_presets().into_iter()
            .find(|p| p.name == self.preset)
            .unwrap_or_else(|| EncoderPreset { id: 3, name: "faster".into(), description: "Faster".into(), quality_level: 4, speed_level: 6 })
    }

    fn set_preset(&mut self, preset: EncoderPreset) -> Result<()> {
        self.preset = preset.name;
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
                    ("CQP".into(), "CQP".into()),
                ],
                ..Default::default()
            },
            PropertyDef {
                name: "profile".into(),
                display_name: "H.264 Profile".into(),
                type_: PropertyType::Enum,
                default: PropertyValue::Enum("high".into()),
                enum_values: vec![
                    ("baseline".into(), "Baseline".into()),
                    ("main".into(), "Main".into()),
                    ("high".into(), "High".into()),
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
            "profile" => Some(PropertyValue::Enum(self.profile.clone())),
            _ => None,
        }
    }

    fn set_parameter(&mut self, name: &str, value: PropertyValue) -> Result<()> {
        match name {
            "bitrate" => {
                if let PropertyValue::Int(b) = value { self.bitrate = b as u32; }
            }
            "keyint" => {
                if let PropertyValue::Int(k) = value { self.keyint = k as u32; }
            }
            "rate_control" => {
                if let PropertyValue::Enum(rc) = value {
                    self.rate_control = match rc.as_str() {
                        "CBR" => RateControlMode::CBR,
                        "VBR" => RateControlMode::VBR,
                        "CRF" => RateControlMode::CRF,
                        "CQP" => RateControlMode::CQP,
                        _ => RateControlMode::CBR,
                    };
                }
            }
            "profile" => {
                if let PropertyValue::Enum(p) = value { self.profile = p; }
            }
            _ => {}
        }
        Ok(())
    }

    async fn initialize(&mut self, input: MediaInfo, _output: MediaInfo) -> Result<()> {
        if let MediaInfo::Video(v) = input {
            let info = &v.video;
            self.input_info = Some(info.clone());
            self.width = info.width;
            self.height = info.height;
            self.fps_num = info.fps_num;
            self.fps_den = info.fps_den;
            self.pixel_format = Self::pixel_format_str(info.format);

            let size_str = format!("{}x{}", self.width, self.height);
            let fps_str = format!("{}/{}", self.fps_num, self.fps_den);
            let br_str = format!("{}k", self.bitrate);
            let br2_str = format!("{}k", self.bitrate * 2);
            let br4_str = format!("{}k", self.bitrate * 4);
            let keyint = self.keyint * self.fps_num / self.fps_den;
            let keyint_str = keyint.to_string();
            let keyint_min_str = self.keyint.to_string();

            let mut cmd = FfmpegCommand::new();
            cmd.args(["-f", "rawvideo"]);
            cmd.args(["-pix_fmt", self.pixel_format]);
            cmd.args(["-s", &size_str]);
            cmd.args(["-r", &fps_str]);
            cmd.input("-");
            cmd.args(["-c:v", "libx264"]);
            cmd.args(["-preset", &self.preset]);
            cmd.args(["-profile:v", &self.profile]);
            cmd.args(["-tune", "zerolatency"]);

            match self.rate_control {
                RateControlMode::CBR => {
                    cmd.args(["-b:v", &br_str]);
                    cmd.args(["-maxrate", &br_str]);
                    cmd.args(["-bufsize", &br2_str]);
                    cmd.args(["-x264-params", "nal-hrd=cbr:force-cfr=1"]);
                }
                RateControlMode::VBR => {
                    cmd.args(["-b:v", &br_str]);
                    cmd.args(["-maxrate", &br2_str]);
                    cmd.args(["-bufsize", &br4_str]);
                }
                RateControlMode::CRF => {
                    cmd.args(["-crf", "23"]);
                }
                RateControlMode::CQP => {
                    cmd.args(["-qp", "23"]);
                }
            }

            cmd.args(["-g", &keyint_str]);
            cmd.args(["-keyint_min", &keyint_min_str]);
            cmd.args(["-f", "h264"]);
            cmd.output("-");

            let mut child = cmd.spawn()?;
            self.stdin = child.take_stdin();
            self.child = Some(child);
            self.initialized = true;

            println!("[FFmpeg x264] Initialized: {}x{} @ {}fps, {}kbps, preset={}",
                self.width, self.height, self.fps_num / self.fps_den, self.bitrate, self.preset);
        }
        Ok(())
    }

    async fn encode(&mut self, input: MediaData) -> Result<Option<EncodedPacket>> {
        if !self.initialized {
            return Ok(None);
        }

        if let MediaData::Video(frame) = input {
            if let Some(ref mut stdin) = self.stdin {
                stdin.write_all(&frame.data)?;
                stdin.flush()?;
            }

            if let Some(ref mut child) = self.child {
                if let Ok(mut iter) = child.iter() {
                    for event in iter.by_ref() {
                        if let FfmpegEvent::OutputChunk(data) = event {
                            let packet = EncodedPacket {
                                data,
                                pts: self.frame_count,
                                dts: self.frame_count,
                                duration: 0,
                                keyframe: self.frame_count == 0,
                                track: TrackId(0),
                            };
                            self.frame_count += 1;
                            return Ok(Some(packet));
                        }
                    }
                }
            }

            self.frame_count += 1;
            Ok(None)
        } else {
            Ok(None)
        }
    }

    async fn flush(&mut self) -> Result<Vec<EncodedPacket>> {
        let mut packets = Vec::new();
        self.stdin = None;
        if let Some(mut child) = self.child.take() {
            if let Ok(mut iter) = child.iter() {
                for event in iter.by_ref() {
                    if let FfmpegEvent::OutputChunk(data) = event {
                        packets.push(EncodedPacket {
                            data,
                            pts: self.frame_count,
                            dts: self.frame_count,
                            duration: 0,
                            keyframe: false,
                            track: TrackId(0),
                        });
                    }
                }
            }
        }
        self.initialized = false;
        Ok(packets)
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub fn create_ffmpeg_h264_encoder() -> Box<dyn Encoder> {
    Box::new(FfmpegH264Encoder::new())
}
