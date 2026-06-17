//! ghoststream-backed implementation of the capture/encode session.
//!
//! ghoststream is async; we own a small Tokio runtime and `block_on` the
//! pipeline's start/stop so the surrounding launcher stays synchronous.

use std::path::Path;

use anyhow::{Context, Result};
use ghoststream::{Container, Output, Pipeline, PipelineBuilder, Preset};
use tokio::runtime::Runtime;

use super::Destination;

pub struct Session {
    rt: Runtime,
    pipeline: Pipeline,
}

impl Session {
    pub fn start(dest: Destination) -> Result<Self> {
        let (preset, output) = match dest {
            Destination::File(path) => {
                let container = container_for(&path);
                (Preset::Recording, Output::file(path, container))
            }
            Destination::Rtmp(url) => (Preset::Stream1080p60, Output::rtmp(url)),
            Destination::Srt(url) => (Preset::LowLatency, Output::srt(url, 120)),
        };

        let rt = Runtime::new().context("failed to create capture runtime")?;
        let pipeline = PipelineBuilder::new()
            .preset(preset)
            .output(output)
            .build()
            .context("failed to build ghoststream pipeline")?;
        rt.block_on(pipeline.start())
            .context("failed to start capture pipeline")?;

        Ok(Self { rt, pipeline })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.rt.block_on(self.pipeline.stop());
    }
}

/// Map a file extension to a ghoststream container format.
fn container_for(path: &Path) -> Container {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("mp4") => Container::Mp4,
        Some("webm") => Container::WebM,
        Some("ts") => Container::Ts,
        _ => Container::Matroska,
    }
}

pub fn status() -> super::EncoderStatus {
    let info = ghoststream::get_encoder_info();
    super::EncoderStatus {
        available: true,
        nvenc: info.nvenc_available,
        gpu_name: info.gpu_name,
        codecs: info.nvenc_codecs.iter().map(|c| format!("{c:?}")).collect(),
    }
}
