//! ghostwave-core backed implementation of the noise-suppression session.
//!
//! A [`GhostWaveProcessor`] performs the (push-based) DSP while a
//! [`PipeWireIntegration`] owns the real-time thread and publishes the
//! "GhostWave Clean" virtual source. Audio is routed through the processor from
//! the PipeWire callback.

use std::sync::{Arc, Mutex};

use anyhow::Result;
use ghostwave_core::{Config, GhostWaveProcessor, PipeWireConfig, PipeWireIntegration};

pub struct Session {
    integration: PipeWireIntegration,
    mode: String,
    // Kept alive for the lifetime of the PipeWire callback below.
    #[allow(dead_code)]
    processor: Arc<Mutex<GhostWaveProcessor>>,
}

impl Session {
    pub fn start(strength: Option<f32>) -> Result<Self> {
        let mut config = Config::default();
        config.noise_suppression.enabled = true;
        if let Some(s) = strength {
            config.noise_suppression.strength = s.clamp(0.0, 1.0);
        }

        let processor = GhostWaveProcessor::new(config)?;
        let mode = processor.get_processing_mode();
        let processor = Arc::new(Mutex::new(processor));

        let mut integration = PipeWireIntegration::new(PipeWireConfig::default());
        let proc = Arc::clone(&processor);
        integration.start(move |input: &[f32], output: &mut [f32]| match proc.lock() {
            Ok(p) => {
                if p.process(input, output).is_err() {
                    output.copy_from_slice(input);
                }
            }
            Err(_) => output.copy_from_slice(input),
        })?;

        Ok(Self {
            integration,
            mode,
            processor,
        })
    }

    pub fn processing_mode(&self) -> String {
        self.mode.clone()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.integration.stop();
    }
}

/// Probe ghostwave-core capabilities without starting a PipeWire stream.
pub fn status() -> super::AudioStatus {
    match GhostWaveProcessor::new(Config::default()) {
        Ok(p) => super::AudioStatus {
            available: true,
            processing_mode: Some(p.get_processing_mode()),
            rtx_acceleration: p.has_rtx_acceleration(),
        },
        Err(_) => super::AudioStatus {
            available: true,
            processing_mode: None,
            rtx_acceleration: false,
        },
    }
}
