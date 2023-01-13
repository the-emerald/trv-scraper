use erc_nft_metadata::Metadata;
use ethers_core::types::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FighterResponse {
    pub attributes: Attributes,
    pub statistic: Statistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attributes {
    pub id: u64,
    pub champion_type: String,
    #[serde(flatten)]
    pub attributes: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub wisdom: Wisdom,
    pub elo: u64,
    pub owner_address: Address,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wisdom {
    pub point: u64,
    pub strength: Stat,
    pub attack: Stat,
    pub defence: Stat,
    pub omega: Stat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stat {
    pub current_range: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FIGHTER: &str = include_str!("tests/fighter.json");

    #[test]
    fn test_fighter() {
        let _ = serde_json::from_str::<FighterResponse>(TEST_FIGHTER).unwrap();
    }
}
