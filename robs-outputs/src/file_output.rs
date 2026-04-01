use robs_core::traits::*;
use robs_core::*;
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use parking_lot::RwLock;

pub struct FileOutput {
    id: OutputId,
    name: String,
    path: PathBuf,
    format: String,
    file: Option<RwLock<File>>,
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
            file: None,
            stats: RwLock::new(FileOutputStats::default()),
        }
    }
    
    pub fn set_path(&mut self, path: PathBuf) {
        self.path = path;
    }
    
    pub fn set_format(&mut self, format: String) {
        self.format = format;
    }
    
    fn write_header(&mut self) -> Result<()> {
        println!("[FileOutput] Writing {} header to {}", self.format, self.path.display());
        Ok(())
    }
    
    fn write_trailer(&mut self) -> Result<()> {
        println!("[FileOutput] Writing {} trailer", self.format);
        Ok(())
    }
}

#[async_trait]
impl Output for FileOutput {
    fn id(&self) -> OutputId { self.id }
    fn name(&self) -> &str { &self.name }
    fn protocol(&self) -> &str { "file" }
    
    fn is_connected(&self) -> bool {
        self.file.is_some()
    }
    
    fn is_reconnecting(&self) -> bool {
        false
    }
    
    async fn connect(&mut self) -> Result<()> {
        let file = File::create(&self.path)?;
        self.file = Some(RwLock::new(file));
        self.write_header()?;
        println!("[FileOutput] Recording to {}", self.path.display());
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        self.write_trailer()?;
        self.file = None;
        println!("[FileOutput] Recording saved to {}", self.path.display());
        Ok(())
    }
    
    async fn send_packet(&mut self, packet: EncodedPacket) -> Result<()> {
        if let Some(file) = &self.file {
            let mut f = file.write();
            
            let mut stats = self.stats.write();
            stats.bytes_written += packet.data.len() as u64;
            stats.frames_written += 1;
        }
        Ok(())
    }
    
    fn properties_definition(&self) -> Vec<PropertyDef> {
        vec![
            PropertyDef {
                name: "path".into(),
                display_name: "File Path".into(),
                type_: PropertyType::Path,
                default: PropertyValue::Path(String::new()),
                ..Default::default()
            },
            PropertyDef {
                name: "format".into(),
                display_name: "Container Format".into(),
                type_: PropertyType::Enum,
                default: PropertyValue::Enum("mp4".into()),
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