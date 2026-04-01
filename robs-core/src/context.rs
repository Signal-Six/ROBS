use crate::event::{EventBus, EventRx, RobsEvent};
use crate::registry::{SourceRegistry, EncoderRegistry, OutputRegistry};
use crate::pipeline::Pipeline;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct AppContext {
    pub source_registry: Arc<RwLock<SourceRegistry>>,
    pub encoder_registry: Arc<RwLock<EncoderRegistry>>,
    pub output_registry: Arc<RwLock<OutputRegistry>>,
    pub event_bus: EventBus,
    pub pipeline: Arc<RwLock<Pipeline>>,
}

impl AppContext {
    pub fn new() -> (Self, EventRx) {
        let (event_bus, event_rx) = EventBus::new();
        let ctx = Self {
            source_registry: Arc::new(RwLock::new(SourceRegistry::new())),
            encoder_registry: Arc::new(RwLock::new(EncoderRegistry::new())),
            output_registry: Arc::new(RwLock::new(OutputRegistry::new())),
            event_bus,
            pipeline: Arc::new(RwLock::new(Pipeline::new())),
        };
        (ctx, event_rx)
    }
    
    pub fn send_event(&self, event: RobsEvent) {
        self.event_bus.send(event);
    }
    
    pub fn event_tx(&self) -> crate::event::EventTx {
        self.event_bus.tx()
    }
}

impl Default for AppContext {
    fn default() -> Self {
        let (ctx, _rx) = Self::new();
        ctx
    }
}

pub struct ScopedContext {
    ctx: Arc<AppContext>,
}

impl ScopedContext {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }
    
    pub fn context(&self) -> &AppContext {
        &self.ctx
    }
}