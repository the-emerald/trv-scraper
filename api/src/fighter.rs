use erc_nft_metadata::Metadata;
use ethers_core::types::Address;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FighterResponse {
    pub attributes: Attributes,
    pub statistic: Statistics,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Attributes {
    pub id: u64,
    pub champion_type: Option<String>,
    #[serde(flatten)]
    pub attributes: Metadata,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Statistics {
    pub wisdom: Wisdom,
    pub elo: Option<u64>,
    pub owner_address: Address,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Wisdom {
    pub point: u64,
    pub strength: Stat,
    pub attack: Stat,
    pub defence: Stat,
    pub omega: Stat,
}

#[derive(Debug, Clone)]
pub struct Stat {
    pub current_range: u64,
    pub from: u64,
    pub to: u64,
}

impl<'de> Deserialize<'de> for Stat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Clone, Debug, Deserialize)]
        struct Helper {
            current_range: u64,
            range: [u64; 2],
        }

        let helper: Helper = Deserialize::deserialize(deserializer)?;
        Ok(Self {
            current_range: helper.current_range,
            from: helper.range[0],
            to: helper.range[1],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FIGHTER: &str = include_str!("tests/fighter.json");
    const TEST_FIGHTER_29001: &str = include_str!("tests/fighter_29001.json");
    const TEST_FIGHTER_28787: &str = include_str!("tests/fighter_28787.json");

    #[test]
    fn test_fighter() {
        let _ = serde_json::from_str::<FighterResponse>(TEST_FIGHTER).unwrap();
    }

    #[test]
    fn test_fighter_29001() {
        let _ = serde_json::from_str::<FighterResponse>(TEST_FIGHTER_29001).unwrap();
    }

    #[test]
    fn test_fighter_28787() {
        let _ = serde_json::from_str::<FighterResponse>(TEST_FIGHTER_28787).unwrap();
    }
}
