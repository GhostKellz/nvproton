//! Microphone noise suppression integration.
//!
//! Wraps [ghostwave-core](https://github.com/ghostkellz/ghostwave)'s PipeWire
//! virtual-source ("GhostWave Clean") behind a small, feature-gated API so call
//! sites stay identical whether or not the `noise-suppression` feature is built.
//! When the feature is disabled every entry point is a cheap no-op/stub.

use serde::{Deserialize, Serialize};

#[cfg(feature = "noise-suppression")]
mod ghostwave;

/// Static capability report for status output. Does not start any stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStatus {
    /// Whether the `noise-suppression` feature is compiled in.
    pub available: bool,
    /// Processing mode label (e.g. "RTX GPU", "CPU"), if the backend reports one.
    pub processing_mode: Option<String>,
    /// Whether RTX/GPU acceleration is active for noise suppression.
    pub rtx_acceleration: bool,
}

/// Query noise-suppression capabilities without starting a virtual source.
pub fn status() -> AudioStatus {
    #[cfg(feature = "noise-suppression")]
    {
        ghostwave::status()
    }
    #[cfg(not(feature = "noise-suppression"))]
    {
        AudioStatus {
            available: false,
            processing_mode: None,
            rtx_acceleration: false,
        }
    }
}

/// An active noise-suppression session.
///
/// While this value is alive the "GhostWave Clean" virtual microphone source is
/// exposed to applications. Dropping it tears the virtual source down.
pub struct NoiseSuppression {
    #[cfg(feature = "noise-suppression")]
    inner: ghostwave::Session,
}

impl NoiseSuppression {
    /// Start the virtual microphone source.
    ///
    /// `strength` is the optional 0.0–1.0 noise-reduction strength; `None` uses
    /// the backend default. Returns an error if the feature is not compiled in.
    pub fn start(strength: Option<f32>) -> anyhow::Result<Self> {
        #[cfg(feature = "noise-suppression")]
        {
            Ok(Self {
                inner: ghostwave::Session::start(strength)?,
            })
        }
        #[cfg(not(feature = "noise-suppression"))]
        {
            let _ = strength;
            anyhow::bail!(
                "nvproton was built without the `noise-suppression` feature (rebuild with --features noise-suppression)"
            )
        }
    }

    /// Human-readable processing mode label for status reporting (e.g. "RTX GPU", "CPU").
    pub fn processing_mode(&self) -> String {
        #[cfg(feature = "noise-suppression")]
        {
            self.inner.processing_mode()
        }
        #[cfg(not(feature = "noise-suppression"))]
        {
            "unavailable".to_string()
        }
    }
}
