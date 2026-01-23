//! Built-in profile presets for nvproton
//!
//! Provides ready-to-use profiles for common scenarios:
//! - Steam Deck: Optimized for handheld gaming
//! - Competitive: Ultra-low latency for esports
//! - Balanced: Good mix of performance and quality
//! - Quality: Maximum visual quality

use anyhow::Result;
use serde_yaml::{Mapping, Value};

use crate::profile::{ProfileDocument, ProfileManager};

/// Preset type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetType {
    SteamDeck,
    Competitive,
    Balanced,
    Quality,
    Battery,
    // DLSS 4.5 presets
    DlssQuality,
    DlssPerformance,
    DlssFrameGen,
    DlssMfg4x,
    DlssDynamic,
    DlssMaxFps,
}

impl PresetType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SteamDeck => "steam-deck",
            Self::Competitive => "competitive",
            Self::Balanced => "balanced",
            Self::Quality => "quality",
            Self::Battery => "battery",
            Self::DlssQuality => "dlss-quality",
            Self::DlssPerformance => "dlss-performance",
            Self::DlssFrameGen => "dlss-framegen",
            Self::DlssMfg4x => "dlss-mfg-4x",
            Self::DlssDynamic => "dlss-dynamic",
            Self::DlssMaxFps => "dlss-max-fps",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::SteamDeck => "Optimized for Steam Deck handheld gaming",
            Self::Competitive => "Ultra-low latency for esports titles",
            Self::Balanced => "Good mix of performance and visual quality",
            Self::Quality => "Maximum visual quality, performance secondary",
            Self::Battery => "Power-efficient settings for extended battery life",
            Self::DlssQuality => "DLSS Quality mode - best image quality (RTX 20+)",
            Self::DlssPerformance => "DLSS Performance - 2x upscaling with frame gen (RTX 40+)",
            Self::DlssFrameGen => "DLSS 3 Frame Generation enabled (RTX 40+)",
            Self::DlssMfg4x => "DLSS 4 Multi Frame Gen 4x (RTX 50)",
            Self::DlssDynamic => "DLSS 4.5 Dynamic MFG - adapts to display (RTX 50)",
            Self::DlssMaxFps => "DLSS 4.5 Max FPS - 6x frame gen for 4K@240Hz (RTX 50)",
        }
    }

    pub fn all() -> &'static [PresetType] {
        &[
            Self::SteamDeck,
            Self::Competitive,
            Self::Balanced,
            Self::Quality,
            Self::Battery,
            Self::DlssQuality,
            Self::DlssPerformance,
            Self::DlssFrameGen,
            Self::DlssMfg4x,
            Self::DlssDynamic,
            Self::DlssMaxFps,
        ]
    }

    pub fn from_name(name: &str) -> Option<PresetType> {
        match name.to_lowercase().as_str() {
            "steam-deck" | "steamdeck" | "deck" => Some(Self::SteamDeck),
            "competitive" | "esports" | "low-latency" => Some(Self::Competitive),
            "balanced" | "default" => Some(Self::Balanced),
            "quality" | "high" | "ultra" => Some(Self::Quality),
            "battery" | "power-save" | "powersave" => Some(Self::Battery),
            "dlss-quality" | "dlss_quality" => Some(Self::DlssQuality),
            "dlss-performance" | "dlss_performance" => Some(Self::DlssPerformance),
            "dlss-framegen" | "dlss_framegen" | "dlss-fg" => Some(Self::DlssFrameGen),
            "dlss-mfg-4x" | "dlss_mfg_4x" | "mfg-4x" | "mfg4x" => Some(Self::DlssMfg4x),
            "dlss-dynamic" | "dlss_dynamic" | "dynamic" => Some(Self::DlssDynamic),
            "dlss-max-fps" | "dlss_max_fps" | "max-fps" => Some(Self::DlssMaxFps),
            _ => None,
        }
    }

    /// Check if preset requires specific GPU generation
    #[allow(dead_code)] // Library API for GPU compatibility checks
    pub fn required_gpu(&self) -> Option<&'static str> {
        match self {
            Self::DlssQuality | Self::DlssPerformance => Some("RTX 20+"),
            Self::DlssFrameGen => Some("RTX 40+"),
            Self::DlssMfg4x | Self::DlssDynamic | Self::DlssMaxFps => Some("RTX 50"),
            _ => None,
        }
    }
}

/// Generate a preset profile document
pub fn generate_preset(preset: PresetType) -> ProfileDocument {
    let mut settings = Mapping::new();

    match preset {
        PresetType::SteamDeck => {
            // Display settings optimized for Steam Deck
            let mut display = Mapping::new();
            display.insert(val("resolution"), val("1280x800"));
            display.insert(val("refresh_rate"), val("90"));
            display.insert(val("vrr"), val("true"));
            display.insert(val("hdr"), val("true"));
            settings.insert(val("display"), Value::Mapping(display));

            // FSR upscaling (AMD FidelityFX)
            let mut upscaling = Mapping::new();
            upscaling.insert(val("enabled"), val("true"));
            upscaling.insert(val("mode"), val("fsr"));
            upscaling.insert(val("sharpness"), val("5"));
            upscaling.insert(val("render_scale"), val("0.77")); // ~720p internal
            settings.insert(val("upscaling"), Value::Mapping(upscaling));

            // Frame limiting for battery
            let mut framerate = Mapping::new();
            framerate.insert(val("limit"), val("60"));
            framerate.insert(val("allow_tearing"), val("false"));
            settings.insert(val("framerate"), Value::Mapping(framerate));

            // Power management
            let mut power = Mapping::new();
            power.insert(val("tdp_limit"), val("15")); // 15W default
            power.insert(val("gpu_clock"), val("1100")); // MHz
            power.insert(val("cpu_boost"), val("true"));
            settings.insert(val("power"), Value::Mapping(power));

            // MangoHud for monitoring
            let mut mangohud = Mapping::new();
            mangohud.insert(val("enabled"), val("true"));
            mangohud.insert(val("position"), val("top-left"));
            mangohud.insert(val("compact"), val("true"));
            settings.insert(val("mangohud"), Value::Mapping(mangohud));

            // Gamescope integration
            let mut gamescope = Mapping::new();
            gamescope.insert(val("enabled"), val("true"));
            gamescope.insert(val("mode"), val("embedded"));
            gamescope.insert(val("fsr"), val("true"));
            settings.insert(val("gamescope"), Value::Mapping(gamescope));
        }

        PresetType::Competitive => {
            // Ultra-low latency display settings
            let mut display = Mapping::new();
            display.insert(val("vrr"), val("true"));
            display.insert(val("vsync"), val("false"));
            display.insert(val("hdr"), val("false")); // Disable for latency
            settings.insert(val("display"), Value::Mapping(display));

            // No upscaling - native resolution
            let mut upscaling = Mapping::new();
            upscaling.insert(val("enabled"), val("false"));
            settings.insert(val("upscaling"), Value::Mapping(upscaling));

            // Frame timing
            let mut framerate = Mapping::new();
            framerate.insert(val("limit"), val("0")); // Unlimited
            framerate.insert(val("allow_tearing"), val("true"));
            framerate.insert(val("reflex"), val("enabled+boost"));
            settings.insert(val("framerate"), Value::Mapping(framerate));

            // NVIDIA Reflex
            let mut nvidia = Mapping::new();
            nvidia.insert(val("reflex"), val("enabled+boost"));
            nvidia.insert(val("low_latency"), val("ultra"));
            nvidia.insert(val("prerendered_frames"), val("1"));
            settings.insert(val("nvidia"), Value::Mapping(nvidia));

            // Gamemode for CPU governor
            let mut gamemode = Mapping::new();
            gamemode.insert(val("enabled"), val("true"));
            gamemode.insert(val("cpu_governor"), val("performance"));
            gamemode.insert(val("renice"), val("-10"));
            settings.insert(val("gamemode"), Value::Mapping(gamemode));

            // No MangoHud (minimal overhead)
            let mut mangohud = Mapping::new();
            mangohud.insert(val("enabled"), val("false"));
            settings.insert(val("mangohud"), Value::Mapping(mangohud));
        }

        PresetType::Balanced => {
            // Balanced display settings
            let mut display = Mapping::new();
            display.insert(val("vrr"), val("true"));
            display.insert(val("vsync"), val("true"));
            display.insert(val("hdr"), val("auto"));
            settings.insert(val("display"), Value::Mapping(display));

            // Moderate frame limiting
            let mut framerate = Mapping::new();
            framerate.insert(val("limit"), val("0")); // Match display
            framerate.insert(val("allow_tearing"), val("false"));
            framerate.insert(val("reflex"), val("enabled"));
            settings.insert(val("framerate"), Value::Mapping(framerate));

            // NVIDIA settings
            let mut nvidia = Mapping::new();
            nvidia.insert(val("reflex"), val("enabled"));
            nvidia.insert(val("low_latency"), val("on"));
            nvidia.insert(val("prerendered_frames"), val("2"));
            settings.insert(val("nvidia"), Value::Mapping(nvidia));

            // Gamemode enabled
            let mut gamemode = Mapping::new();
            gamemode.insert(val("enabled"), val("true"));
            gamemode.insert(val("cpu_governor"), val("performance"));
            settings.insert(val("gamemode"), Value::Mapping(gamemode));

            // MangoHud with sensible defaults
            let mut mangohud = Mapping::new();
            mangohud.insert(val("enabled"), val("true"));
            mangohud.insert(val("position"), val("top-left"));
            mangohud.insert(val("fps"), val("true"));
            mangohud.insert(val("frametime"), val("true"));
            settings.insert(val("mangohud"), Value::Mapping(mangohud));
        }

        PresetType::Quality => {
            // Maximum quality display
            let mut display = Mapping::new();
            display.insert(val("vrr"), val("true"));
            display.insert(val("vsync"), val("true"));
            display.insert(val("hdr"), val("true"));
            settings.insert(val("display"), Value::Mapping(display));

            // DLSS quality mode
            let mut upscaling = Mapping::new();
            upscaling.insert(val("enabled"), val("true"));
            upscaling.insert(val("mode"), val("dlss"));
            upscaling.insert(val("quality"), val("quality")); // DLSS Quality
            upscaling.insert(val("ray_reconstruction"), val("true"));
            settings.insert(val("upscaling"), Value::Mapping(upscaling));

            // Frame limiting to maintain consistency
            let mut framerate = Mapping::new();
            framerate.insert(val("limit"), val("0")); // Match display
            framerate.insert(val("allow_tearing"), val("false"));
            framerate.insert(val("reflex"), val("enabled"));
            settings.insert(val("framerate"), Value::Mapping(framerate));

            // NVIDIA settings
            let mut nvidia = Mapping::new();
            nvidia.insert(val("reflex"), val("enabled"));
            nvidia.insert(val("low_latency"), val("on"));
            nvidia.insert(val("prerendered_frames"), val("3"));
            nvidia.insert(val("anisotropic_filtering"), val("16"));
            settings.insert(val("nvidia"), Value::Mapping(nvidia));

            // MangoHud detailed
            let mut mangohud = Mapping::new();
            mangohud.insert(val("enabled"), val("true"));
            mangohud.insert(val("position"), val("top-right"));
            mangohud.insert(val("full"), val("true"));
            settings.insert(val("mangohud"), Value::Mapping(mangohud));
        }

        PresetType::Battery => {
            // Power-efficient display
            let mut display = Mapping::new();
            display.insert(val("refresh_rate"), val("60"));
            display.insert(val("vrr"), val("false"));
            display.insert(val("vsync"), val("true"));
            display.insert(val("hdr"), val("false"));
            settings.insert(val("display"), Value::Mapping(display));

            // Aggressive upscaling
            let mut upscaling = Mapping::new();
            upscaling.insert(val("enabled"), val("true"));
            upscaling.insert(val("mode"), val("fsr"));
            upscaling.insert(val("quality"), val("performance"));
            upscaling.insert(val("render_scale"), val("0.5")); // 50% internal
            settings.insert(val("upscaling"), Value::Mapping(upscaling));

            // Strict frame limiting
            let mut framerate = Mapping::new();
            framerate.insert(val("limit"), val("30"));
            framerate.insert(val("allow_tearing"), val("false"));
            settings.insert(val("framerate"), Value::Mapping(framerate));

            // Power limits
            let mut power = Mapping::new();
            power.insert(val("tdp_limit"), val("8")); // 8W
            power.insert(val("gpu_clock"), val("800")); // MHz
            power.insert(val("cpu_boost"), val("false"));
            settings.insert(val("power"), Value::Mapping(power));

            // Gamemode in powersave
            let mut gamemode = Mapping::new();
            gamemode.insert(val("enabled"), val("true"));
            gamemode.insert(val("cpu_governor"), val("powersave"));
            settings.insert(val("gamemode"), Value::Mapping(gamemode));

            // Minimal MangoHud
            let mut mangohud = Mapping::new();
            mangohud.insert(val("enabled"), val("false"));
            settings.insert(val("mangohud"), Value::Mapping(mangohud));
        }

        // ===== DLSS 4.5 Presets =====

        PresetType::DlssQuality => {
            // DLSS Quality - best image quality for RTX 20+
            let mut dlss = Mapping::new();
            dlss.insert(val("enabled"), val("true"));
            dlss.insert(val("mode"), val("super_resolution"));
            dlss.insert(val("quality"), val("quality")); // 1.5x upscale
            dlss.insert(val("sharpness"), val("0.0"));
            dlss.insert(val("frame_generation"), val("disabled"));
            dlss.insert(val("ray_reconstruction"), val("false"));
            settings.insert(val("dlss"), Value::Mapping(dlss));

            // VRR and HDR enabled
            let mut display = Mapping::new();
            display.insert(val("vrr"), val("true"));
            display.insert(val("vsync"), val("true"));
            display.insert(val("hdr"), val("true"));
            settings.insert(val("display"), Value::Mapping(display));

            // Reflex for latency
            let mut nvidia = Mapping::new();
            nvidia.insert(val("reflex"), val("enabled"));
            settings.insert(val("nvidia"), Value::Mapping(nvidia));
        }

        PresetType::DlssPerformance => {
            // DLSS Performance with Frame Gen - RTX 40+
            let mut dlss = Mapping::new();
            dlss.insert(val("enabled"), val("true"));
            dlss.insert(val("mode"), val("super_resolution"));
            dlss.insert(val("quality"), val("performance")); // 2x upscale
            dlss.insert(val("sharpness"), val("0.2"));
            dlss.insert(val("frame_generation"), val("enabled"));
            dlss.insert(val("ray_reconstruction"), val("false"));
            settings.insert(val("dlss"), Value::Mapping(dlss));

            let mut display = Mapping::new();
            display.insert(val("vrr"), val("true"));
            display.insert(val("vsync"), val("false")); // Lower latency
            display.insert(val("hdr"), val("true"));
            settings.insert(val("display"), Value::Mapping(display));

            let mut nvidia = Mapping::new();
            nvidia.insert(val("reflex"), val("enabled+boost"));
            settings.insert(val("nvidia"), Value::Mapping(nvidia));
        }

        PresetType::DlssFrameGen => {
            // DLSS 3 Frame Generation focus - RTX 40+
            let mut dlss = Mapping::new();
            dlss.insert(val("enabled"), val("true"));
            dlss.insert(val("mode"), val("frame_generation"));
            dlss.insert(val("quality"), val("balanced")); // 1.7x upscale
            dlss.insert(val("sharpness"), val("0.1"));
            dlss.insert(val("frame_generation"), val("enabled"));
            dlss.insert(val("ray_reconstruction"), val("true"));
            settings.insert(val("dlss"), Value::Mapping(dlss));

            let mut display = Mapping::new();
            display.insert(val("vrr"), val("true"));
            display.insert(val("vsync"), val("true"));
            display.insert(val("hdr"), val("true"));
            settings.insert(val("display"), Value::Mapping(display));

            let mut nvidia = Mapping::new();
            nvidia.insert(val("reflex"), val("enabled+boost"));
            nvidia.insert(val("low_latency"), val("ultra"));
            settings.insert(val("nvidia"), Value::Mapping(nvidia));

            // MangoHud to monitor frame gen
            let mut mangohud = Mapping::new();
            mangohud.insert(val("enabled"), val("true"));
            mangohud.insert(val("fps"), val("true"));
            mangohud.insert(val("frame_timing"), val("true"));
            settings.insert(val("mangohud"), Value::Mapping(mangohud));
        }

        PresetType::DlssMfg4x => {
            // DLSS 4 Multi Frame Gen 4x - RTX 50 only
            let mut dlss = Mapping::new();
            dlss.insert(val("enabled"), val("true"));
            dlss.insert(val("mode"), val("multi_frame_gen"));
            dlss.insert(val("quality"), val("balanced"));
            dlss.insert(val("sharpness"), val("0.1"));
            dlss.insert(val("frame_generation"), val("multi_4x"));
            dlss.insert(val("ray_reconstruction"), val("true"));
            dlss.insert(val("model_type"), val("transformer_v2"));
            settings.insert(val("dlss"), Value::Mapping(dlss));

            let mut display = Mapping::new();
            display.insert(val("vrr"), val("true"));
            display.insert(val("vsync"), val("true"));
            display.insert(val("hdr"), val("true"));
            display.insert(val("target_refresh"), val("165"));
            settings.insert(val("display"), Value::Mapping(display));

            let mut nvidia = Mapping::new();
            nvidia.insert(val("reflex"), val("enabled+boost"));
            nvidia.insert(val("low_latency"), val("ultra"));
            settings.insert(val("nvidia"), Value::Mapping(nvidia));
        }

        PresetType::DlssDynamic => {
            // DLSS 4.5 Dynamic MFG - adapts to display refresh - RTX 50
            let mut dlss = Mapping::new();
            dlss.insert(val("enabled"), val("true"));
            dlss.insert(val("mode"), val("multi_frame_gen"));
            dlss.insert(val("quality"), val("balanced"));
            dlss.insert(val("sharpness"), val("0.1"));
            dlss.insert(val("frame_generation"), val("dynamic"));
            dlss.insert(val("ray_reconstruction"), val("true"));
            dlss.insert(val("model_type"), val("transformer_v2"));

            // Dynamic MFG settings
            let mut dynamic_mfg = Mapping::new();
            dynamic_mfg.insert(val("target_refresh_hz"), val("165"));
            dynamic_mfg.insert(val("min_multiplier"), val("1"));
            dynamic_mfg.insert(val("max_multiplier"), val("4"));
            dynamic_mfg.insert(val("max_latency_ms"), val("20"));
            dynamic_mfg.insert(val("auto_quality_scale"), val("true"));
            dlss.insert(val("dynamic_mfg"), Value::Mapping(dynamic_mfg));
            settings.insert(val("dlss"), Value::Mapping(dlss));

            let mut display = Mapping::new();
            display.insert(val("vrr"), val("true"));
            display.insert(val("vsync"), val("true"));
            display.insert(val("hdr"), val("true"));
            settings.insert(val("display"), Value::Mapping(display));

            let mut nvidia = Mapping::new();
            nvidia.insert(val("reflex"), val("enabled+boost"));
            nvidia.insert(val("low_latency"), val("ultra"));
            settings.insert(val("nvidia"), Value::Mapping(nvidia));
        }

        PresetType::DlssMaxFps => {
            // DLSS 4.5 Max FPS - 6x frame gen for 4K@240Hz - RTX 50
            let mut dlss = Mapping::new();
            dlss.insert(val("enabled"), val("true"));
            dlss.insert(val("mode"), val("multi_frame_gen"));
            dlss.insert(val("quality"), val("ultra_performance")); // 3x upscale
            dlss.insert(val("sharpness"), val("0.3"));
            dlss.insert(val("frame_generation"), val("dynamic_6x"));
            dlss.insert(val("ray_reconstruction"), val("false")); // Disabled for max FPS
            dlss.insert(val("model_type"), val("transformer_v2"));

            // Dynamic MFG maxed out
            let mut dynamic_mfg = Mapping::new();
            dynamic_mfg.insert(val("target_refresh_hz"), val("240"));
            dynamic_mfg.insert(val("min_multiplier"), val("2"));
            dynamic_mfg.insert(val("max_multiplier"), val("6"));
            dynamic_mfg.insert(val("max_latency_ms"), val("25"));
            dynamic_mfg.insert(val("auto_quality_scale"), val("true"));
            dlss.insert(val("dynamic_mfg"), Value::Mapping(dynamic_mfg));
            settings.insert(val("dlss"), Value::Mapping(dlss));

            let mut display = Mapping::new();
            display.insert(val("vrr"), val("true"));
            display.insert(val("vsync"), val("false")); // No vsync for max fps
            display.insert(val("hdr"), val("true"));
            display.insert(val("target_refresh"), val("240"));
            settings.insert(val("display"), Value::Mapping(display));

            let mut nvidia = Mapping::new();
            nvidia.insert(val("reflex"), val("enabled+boost"));
            nvidia.insert(val("low_latency"), val("ultra"));
            nvidia.insert(val("gpu_boost"), val("true")); // Max clocks
            settings.insert(val("nvidia"), Value::Mapping(nvidia));

            // Framerate target
            let mut framerate = Mapping::new();
            framerate.insert(val("limit"), val("0")); // Unlimited
            framerate.insert(val("allow_tearing"), val("true"));
            settings.insert(val("framerate"), Value::Mapping(framerate));
        }
    }

    ProfileDocument {
        name: preset.name().to_string(),
        extends: None,
        settings,
    }
}

/// Install all built-in presets to the profile directory
pub fn install_presets(manager: &ProfileManager, force: bool) -> Result<Vec<String>> {
    let mut installed = Vec::new();

    for preset in PresetType::all() {
        let name = preset.name();
        if !force && manager.exists(name) {
            continue;
        }

        let document = generate_preset(*preset);
        manager.save(&document)?;
        installed.push(name.to_string());
    }

    Ok(installed)
}

/// Check if running on Steam Deck
pub fn is_steam_deck() -> bool {
    // Check environment variable
    if std::env::var("SteamDeck").is_ok() {
        return true;
    }

    // Check DMI product name
    if let Ok(product) = std::fs::read_to_string("/sys/class/dmi/id/product_name") {
        let product = product.trim();
        if product.starts_with("Jupiter") || product.starts_with("Galileo") {
            return true;
        }
    }

    false
}

/// Get the recommended preset for the current system
pub fn recommended_preset() -> PresetType {
    if is_steam_deck() {
        PresetType::SteamDeck
    } else {
        PresetType::Balanced
    }
}

fn val(s: &str) -> Value {
    Value::String(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_names() {
        assert_eq!(PresetType::SteamDeck.name(), "steam-deck");
        assert_eq!(PresetType::Competitive.name(), "competitive");
    }

    #[test]
    fn test_from_name() {
        assert_eq!(PresetType::from_name("steam-deck"), Some(PresetType::SteamDeck));
        assert_eq!(PresetType::from_name("deck"), Some(PresetType::SteamDeck));
        assert_eq!(PresetType::from_name("competitive"), Some(PresetType::Competitive));
        assert_eq!(PresetType::from_name("unknown"), None);
    }

    #[test]
    fn test_generate_steam_deck_preset() {
        let doc = generate_preset(PresetType::SteamDeck);
        assert_eq!(doc.name, "steam-deck");
        assert!(doc.settings.contains_key(&val("display")));
        assert!(doc.settings.contains_key(&val("gamescope")));
    }
}
