//! Game capture / encode / stream integration.
//!
//! Wraps [ghoststream](https://github.com/ghostkellz/ghoststream) (NVENC capture
//! and encode) behind a small, feature-gated API. When the `streaming` feature
//! is disabled the entry points are stubs that report unavailability, so call
//! sites compile and behave identically either way.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[cfg(feature = "streaming")]
mod ghoststream;

/// Where a capture session should send its output.
///
/// The inner values are only consumed when the `streaming` feature is enabled;
/// without it the stub `Session::start` rejects every destination.
#[cfg_attr(not(feature = "streaming"), allow(dead_code))]
pub enum Destination {
    /// Record to a local file (container inferred from the path extension).
    File(PathBuf),
    /// Stream to an RTMP endpoint (Twitch, YouTube, ...).
    Rtmp(String),
    /// Stream to an SRT endpoint (low latency).
    Srt(String),
}

/// An active capture/encode session. Dropping it stops the pipeline.
pub struct Session {
    // Held only for its `Drop`, which tears down the pipeline.
    #[cfg(feature = "streaming")]
    #[allow(dead_code)]
    inner: ghoststream::Session,
}

impl Session {
    /// Start a capture session writing to `dest`.
    ///
    /// Returns an error if the `streaming` feature is not compiled in.
    pub fn start(dest: Destination) -> anyhow::Result<Self> {
        #[cfg(feature = "streaming")]
        {
            Ok(Self {
                inner: ghoststream::Session::start(dest)?,
            })
        }
        #[cfg(not(feature = "streaming"))]
        {
            let _ = dest;
            anyhow::bail!(
                "nvproton was built without the `streaming` feature (rebuild with --features streaming)"
            )
        }
    }

    /// Stop the capture session and flush any pending output.
    pub fn stop(self) {
        // The inner backend's `Drop` tears down the pipeline.
    }
}

/// NVENC / encoder capability report for status output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncoderStatus {
    /// Whether the `streaming` feature is compiled in.
    pub available: bool,
    /// Whether NVENC hardware encoding is usable.
    pub nvenc: bool,
    /// GPU name reported by the encoder backend, if any.
    pub gpu_name: Option<String>,
    /// NVENC-supported codecs (e.g. "H264", "Hevc", "Av1").
    pub codecs: Vec<String>,
}

/// Query encoder capabilities without starting a pipeline.
pub fn status() -> EncoderStatus {
    #[cfg(feature = "streaming")]
    {
        ghoststream::status()
    }
    #[cfg(not(feature = "streaming"))]
    {
        EncoderStatus {
            available: false,
            nvenc: false,
            gpu_name: None,
            codecs: Vec::new(),
        }
    }
}
