use robs_core::traits::*;
use robs_core::*;
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::Write;
use parking_lot::RwLock;

pub struct FileOutput {
    id: OutputId,
    name: String,
    path: PathBuf,
    format: String,
    video_encoder: String,
    audio_encoder: String,
    active: bool,
    ffmpeg_process: Option<std::process::Child>,
    stats: RwLock<FileOutputStats>,
}

#[derive(Debug, Clone, Default)]
pub struct FileOutputStats {
    pub bytes_written: u64,
    pub frames_written: u64,
    pub duration_ms: u64,
}

impl FileOutput {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            id: OutputId(ObjectId::new()),
            name,
            path,
            format: "mp4".to_string(),
            video_encoder: "libx264".to_string(),
            audio_encoder: "aac".to_string(),
            active: false,
            ffmpeg_process: None,
            stats: RwLock::new(FileOutputStats::default()),
        }
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.path = path;
    }

    pub fn set_format(&mut self, format: String) {
        self.format = format;
    }

    pub fn set_video_encoder(&mut self, encoder: String) {
        self.video_encoder = encoder;
    }

    pub fn set_audio_encoder(&mut self, encoder: String) {
        self.audio_encoder = encoder;
    }

    fn start_ffmpeg(&mut self, width: u32, height: u32, fps: u32) -> Result<()> {
        let output_path = format!("{}.{}", self.path.display(), self.format);

        let mut cmd = Command::new("ffmpeg");

        // Video input (raw BGRA from stdin)
        cmd.args([
            "-f", "rawvideo",
            "-pix_fmt", "bgra",
            "-s", &format!("{}x{}", width, height),
            "-r", &fps.to_string(),
            "-i", "-",
        ]);

        // Audio input (raw s16le from stdin)
        cmd.args([
            "-f", "s16le",
            "-ar", "48000", 
            "-ac", "2",
            "-i", "-",
        ]);

        // Video encoder
        match self.video_encoder.as_str() {
            "h264_nvenc" => {
                cmd.args(["-c:v", "h264_nvenc", "-preset", "p4", "-cq", "23"]);
            }
            _ => {
                cmd.args(["-c:v", "libx264", "-preset", "fast", "-crf", "23"]);
            }
        }

        // Audio encoder
        cmd.args(["-c:a", "aac", "-b:a", "192k"]);

        // Output
        cmd.args(["-y", &output_path]);

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        let child = cmd.spawn()?;
        self.ffmpeg_process = Some(child);

        println!("[FileOutput] Recording started: {}", output_path);
        Ok(())
    }

    fn stop_ffmpeg(&mut self) {
        if let Some(mut child) = self.ffmpeg_process.take() {
            drop(child.stdin.take());
            let _ = child.wait();
        }
        let stats = self.stats.read();
        println!("[FileOutput] Recording saved: {} bytes, {} frames", 
            stats.bytes_written, stats.frames_written);
    }
}

#[async_trait]
impl Output for FileOutput {
    fn id(&self) -> OutputId { self.id }
    fn name(&self) -> &str { &self.name }
    fn protocol(&self) -> &str { "file" }

    fn is_connected(&self) -> bool {
        self.active
    }

    fn is_reconnecting(&self) -> bool {
        false
    }

    async fn connect(&mut self) -> Result<()> {
        self.active = true;
        self.start_ffmpeg(1920, 1080, 30)?;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.active = false;
        self.stop_ffmpeg();
        Ok(())
    }

    async fn send_packet(&mut self, packet: EncodedPacket) -> Result<()> {
        if !self.active {
            return Ok(());
        }

        if let Some(ref mut child) = self.ffmpeg_process {
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(&packet.data)?;
                stdin.flush()?;

                let mut stats = self.stats.write();
                stats.bytes_written += packet.data.len() as u64;
                stats.frames_written += 1;
            }
        }
        Ok(())
    }

    fn properties_definition(&self) -> Vec<PropertyDef> {
        vec![
            PropertyDef {
                name: "path".into(),
                display_name: "File Path".into(),
                type_: PropertyType::Path,
                default: PropertyValue::Path(self.path.to_string_lossy().into()),
                ..Default::default()
            },
            PropertyDef {
                name: "format".into(),
                display_name: "Container Format".into(),
                type_: PropertyType::Enum,
                default: PropertyValue::Enum(self.format.clone()),
                enum_values: vec![
                    ("mp4".into(), "MP4".into()),
                    ("mkv".into(), "MKV".into()),
                    ("flv".into(), "FLV".into()),
                    ("mov".into(), "MOV".into()),
                ],
                ..Default::default()
            },
        ]
    }

    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "path" => Some(PropertyValue::Path(self.path.to_string_lossy().into())),
            "format" => Some(PropertyValue::Enum(self.format.clone())),
            _ => None,
        }
    }

    fn set_property(&mut self, name: &str, value: PropertyValue) -> Result<()> {
        match name {
            "path" => {
                if let PropertyValue::Path(p) = value {
                    self.path = PathBuf::from(p);
                }
            }
            "format" => {
                if let PropertyValue::Enum(f) = value {
                    self.format = f;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
