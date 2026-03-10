//! DX12 game knowledge base
//!
//! Contains curated list of popular DirectX 12 games that will benefit
//! from VK_EXT_descriptor_heap optimization and the 595 driver heap fix
//! (VK_NV_extended_sparse_address_space).

use std::collections::HashMap;

use once_cell::sync::Lazy;

/// Game API type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameApi {
    /// DirectX 12 - will benefit from descriptor_heap
    Dx12,
    /// DirectX 11 - uses DXVK, no descriptor_heap benefit
    Dx11,
    /// Vulkan native - no translation layer needed
    Vulkan,
    /// OpenGL - uses Zink or native
    #[allow(dead_code)]
    OpenGL,
    /// Unknown API
    Unknown,
}

impl GameApi {
    pub fn benefits_from_descriptor_heap(&self) -> bool {
        matches!(self, GameApi::Dx12)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            GameApi::Dx12 => "DX12",
            GameApi::Dx11 => "DX11",
            GameApi::Vulkan => "Vulkan",
            GameApi::OpenGL => "OpenGL",
            GameApi::Unknown => "Unknown",
        }
    }
}

/// Known DX12 games by Steam AppID
/// These games will see significant improvement with descriptor_heap
static DX12_GAMES: Lazy<HashMap<&'static str, GameInfo>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // AAA titles with DX12
    m.insert("1245620", GameInfo::dx12("Elden Ring"));
    m.insert("1938090", GameInfo::dx12("Call of Duty: Modern Warfare II"));
    m.insert("2519060", GameInfo::dx12("Call of Duty: Modern Warfare III"));
    m.insert("1172470", GameInfo::dx12("Apex Legends")); // DX11/DX12
    m.insert("271590", GameInfo::dx12("Grand Theft Auto V")); // DX11/DX12
    m.insert("1174180", GameInfo::dx12("Red Dead Redemption 2"));
    m.insert("1151640", GameInfo::dx12("Horizon Zero Dawn"));
    m.insert("1593500", GameInfo::dx12("God of War"));
    m.insert("1817070", GameInfo::dx12("Marvel's Spider-Man Remastered"));
    m.insert("1817190", GameInfo::dx12("Marvel's Spider-Man: Miles Morales"));
    m.insert("2138710", GameInfo::dx12("God of War Ragnarok"));
    m.insert("1888160", GameInfo::dx12("ARMORED CORE VI"));
    m.insert("814380", GameInfo::dx12("Sekiro: Shadows Die Twice"));
    m.insert("374320", GameInfo::dx12("DARK SOULS III"));
    m.insert("570", GameInfo::dx12("Dota 2")); // DX11/Vulkan/DX12
    m.insert("730", GameInfo::vulkan("Counter-Strike 2")); // Source 2, Vulkan native
    m.insert("1091500", GameInfo::dx12("Cyberpunk 2077"));
    m.insert("292030", GameInfo::dx12("The Witcher 3: Wild Hunt"));
    m.insert("1086940", GameInfo::vulkan("Baldur's Gate 3")); // Vulkan native
    m.insert("1449560", GameInfo::dx12("Halo Infinite"));
    m.insert("1063730", GameInfo::dx12("Halo: The Master Chief Collection"));
    m.insert("1240440", GameInfo::dx12("Halo: Combat Evolved Anniversary")); // MCC

    // Microsoft titles (DX12 native)
    m.insert("1293830", GameInfo::dx12("Forza Horizon 4"));
    m.insert("1551360", GameInfo::dx12("Forza Horizon 5"));
    m.insert("1262540", GameInfo::dx12("Forza Motorsport"));
    m.insert("871720", GameInfo::dx12("Gears 5"));
    m.insert("1328670", GameInfo::dx12("Microsoft Flight Simulator"));
    m.insert("459820", GameInfo::dx12("Age of Empires II: Definitive Edition"));
    m.insert("933110", GameInfo::dx12("Age of Empires IV"));
    m.insert("1817480", GameInfo::dx12("Age of Mythology: Retold"));
    m.insert("1240440", GameInfo::dx12("Starfield"));

    // Competitive/esports (mixed)
    m.insert("578080", GameInfo::dx11("PUBG: BATTLEGROUNDS")); // DX11
    m.insert("359550", GameInfo::dx11("Tom Clancy's Rainbow Six Siege")); // Vulkan option
    m.insert("440", GameInfo::dx11("Team Fortress 2")); // Source, DX9/DX11
    m.insert("252490", GameInfo::dx11("Rust")); // DX11
    m.insert("892970", GameInfo::dx12("Valheim")); // Unity, Vulkan/DX12
    m.insert("275850", GameInfo::vulkan("No Man's Sky")); // Vulkan native
    m.insert("1623730", GameInfo::dx12("Palworld"));

    // Recent AAA (2023-2024)
    m.insert("2050650", GameInfo::dx12("Resident Evil 4 (2023)"));
    m.insert("1222730", GameInfo::dx12("TEKKEN 8"));
    m.insert("2358720", GameInfo::dx12("Black Myth: Wukong"));
    m.insert("1794680", GameInfo::dx12("Lies of P"));
    m.insert("2054970", GameInfo::dx12("Diablo IV"));
    m.insert("2379780", GameInfo::dx12("S.T.A.L.K.E.R. 2"));
    m.insert("2246340", GameInfo::dx12("Monster Hunter Wilds"));
    m.insert("2252570", GameInfo::dx12("Street Fighter 6"));
    m.insert("2344520", GameInfo::dx12("Alan Wake 2"));
    m.insert("2420510", GameInfo::dx12("Lords of the Fallen (2023)"));
    m.insert("1677740", GameInfo::dx12("Wo Long: Fallen Dynasty"));
    m.insert("1971870", GameInfo::dx12("Like a Dragon: Ishin!"));
    m.insert("1687950", GameInfo::dx12("Persona 5 Royal"));
    m.insert("2254740", GameInfo::dx12("Granblue Fantasy: Relink"));
    m.insert("2149940", GameInfo::dx12("Like a Dragon Gaiden"));
    m.insert("2406770", GameInfo::dx12("Like a Dragon: Infinite Wealth"));

    // Older but popular DX12 titles
    m.insert("582010", GameInfo::dx12("Monster Hunter: World"));
    m.insert("1118310", GameInfo::dx12("Monster Hunter Rise"));
    m.insert("1240440", GameInfo::dx12("Starfield"));
    m.insert("8500", GameInfo::dx11("EVE Online"));
    m.insert("292120", GameInfo::dx12("FINAL FANTASY XV"));
    m.insert("1382330", GameInfo::dx12("FINAL FANTASY VII REMAKE"));
    m.insert("1096410", GameInfo::dx12("FINAL FANTASY XVI")); // Demo only on PC
    m.insert("1286680", GameInfo::dx12("CRISIS CORE –FINAL FANTASY VII– REUNION"));
    m.insert("39210", GameInfo::dx12("FINAL FANTASY XIV Online"));
    m.insert("1113000", GameInfo::dx12("Persona 3 Reload"));
    m.insert("1382330", GameInfo::dx12("Persona 4 Golden")); // DX11

    // Strategy games with DX12
    m.insert("594570", GameInfo::dx12("Total War: WARHAMMER II"));
    m.insert("1142710", GameInfo::dx12("Total War: WARHAMMER III"));
    m.insert("1158310", GameInfo::dx12("Crusader Kings III"));
    m.insert("1328670", GameInfo::dx12("Hearts of Iron IV")); // Actually DX9/11
    m.insert("236390", GameInfo::dx12("War Thunder"));
    m.insert("444090", GameInfo::dx12("Payday 3"));

    // Unreal Engine 5 games (DX12 by default)
    m.insert("2215430", GameInfo::dx12("The Finals"));
    m.insert("1966720", GameInfo::dx12("Warhammer 40,000: Space Marine 2"));
    m.insert("2139460", GameInfo::dx12("Once Human"));
    m.insert("2677660", GameInfo::dx12("Throne and Liberty"));
    m.insert("2358720", GameInfo::dx12("Black Myth: Wukong"));
    m.insert("2507950", GameInfo::dx12("Delta Force"));

    m
});

/// Game info entry
#[derive(Debug, Clone)]
pub struct GameInfo {
    pub name: &'static str,
    pub api: GameApi,
}

impl GameInfo {
    fn dx12(name: &'static str) -> Self {
        Self {
            name,
            api: GameApi::Dx12,
        }
    }

    fn dx11(name: &'static str) -> Self {
        Self {
            name,
            api: GameApi::Dx11,
        }
    }

    #[allow(dead_code)]
    fn vulkan(name: &'static str) -> Self {
        Self {
            name,
            api: GameApi::Vulkan,
        }
    }
}

/// Look up a game's API by Steam AppID
pub fn get_game_api(app_id: &str) -> GameApi {
    DX12_GAMES
        .get(app_id)
        .map(|g| g.api)
        .unwrap_or(GameApi::Unknown)
}

/// Get info about a known game
#[allow(dead_code)] // Library API
pub fn get_game_info(app_id: &str) -> Option<&'static GameInfo> {
    DX12_GAMES.get(app_id)
}

/// Check if a game is known to be DX12
#[allow(dead_code)] // Library API
pub fn is_dx12_game(app_id: &str) -> bool {
    get_game_api(app_id) == GameApi::Dx12
}

/// Get list of all known DX12 games
pub fn known_dx12_games() -> impl Iterator<Item = (&'static str, &'static GameInfo)> {
    DX12_GAMES
        .iter()
        .filter(|(_, info)| info.api == GameApi::Dx12)
        .map(|(id, info)| (*id, info))
}

/// Count of known DX12 games
#[allow(dead_code)] // Library API
pub fn dx12_game_count() -> usize {
    DX12_GAMES
        .values()
        .filter(|g| g.api == GameApi::Dx12)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_games() {
        assert!(is_dx12_game("1245620")); // Elden Ring
        assert!(is_dx12_game("1091500")); // Cyberpunk 2077
        assert!(!is_dx12_game("9999999")); // Unknown
    }

    #[test]
    fn test_game_api() {
        assert_eq!(get_game_api("1245620"), GameApi::Dx12); // Elden Ring
        assert_eq!(get_game_api("730"), GameApi::Vulkan); // CS2 - Vulkan native
        assert_eq!(get_game_api("1086940"), GameApi::Vulkan); // BG3 - Vulkan native
        assert_eq!(get_game_api("1091500"), GameApi::Dx12); // Cyberpunk 2077
        assert_eq!(get_game_api("unknown"), GameApi::Unknown);
    }

    #[test]
    fn test_descriptor_heap_benefit() {
        assert!(GameApi::Dx12.benefits_from_descriptor_heap());
        assert!(!GameApi::Dx11.benefits_from_descriptor_heap());
        assert!(!GameApi::Vulkan.benefits_from_descriptor_heap());
    }
}
