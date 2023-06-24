use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;

#[derive(Clone, Debug, Deserialize)]
#[serde(remote = "Self")]
pub struct TournamentDetailResponse {
    pub champions: Vec<Champion>,
    pub battles: Vec<Battle>,
}

impl<'de> Deserialize<'de> for TournamentDetailResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn derived_impl<'de, D>(deserializer: D) -> Result<TournamentDetailResponse, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            TournamentDetailResponse::deserialize(deserializer)
        }

        #[derive(Clone, Debug, Deserialize)]
        struct Helper {
            #[serde(rename = "match", deserialize_with = "derived_impl")]
            inner: TournamentDetailResponse,
        }

        Deserialize::deserialize(deserializer).map(|h: Helper| h.inner)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Champion {
    pub token_id: u64,
    pub first_wins: u64,
    pub second_wins: u64,
    pub total_fought: u64,
    pub stance: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(remote = "Self")]
pub struct Battle {
    pub round: u64,
    pub champions: Vec<ChampionAttack>,
}

impl<'de> Deserialize<'de> for Battle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn derived_impl<'de, D>(deserializer: D) -> Result<Battle, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Battle::deserialize(deserializer)
        }

        #[derive(Clone, Debug, Deserialize)]
        struct Helper {
            #[serde(deserialize_with = "derived_impl")]
            engagement: Battle,
        }

        Deserialize::deserialize(deserializer).map(|h: Helper| h.engagement)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChampionAttack {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: u64,
    pub attack: Vec<Attack>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Attack {
    pub special_attack: bool,
    pub special_defend: bool,
    pub missed_hit: bool,
    pub damage: u64,
    pub order: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DETAIL_1: &str = include_str!("tests/tournament_detail_1.json");
    const TEST_DETAIL_2: &str = include_str!("tests/tournament_detail_2.json");
    const TEST_DETAIL_3: &str = include_str!("tests/tournament_detail_3.json");
    const TEST_DETAIL_4: &str = include_str!("tests/tournament_detail_empty_battle.json");

    #[test]
    fn test_detail_1() {
        let _ = serde_json::from_str::<TournamentDetailResponse>(TEST_DETAIL_1).unwrap();
    }

    #[test]
    fn test_detail_2() {
        let _ = serde_json::from_str::<TournamentDetailResponse>(TEST_DETAIL_2).unwrap();
    }

    #[test]
    fn test_detail_3() {
        let _ = serde_json::from_str::<TournamentDetailResponse>(TEST_DETAIL_3).unwrap();
    }

    #[test]
    fn test_detail_empty_battle() {
        let x = serde_json::from_str::<TournamentDetailResponse>(TEST_DETAIL_4).unwrap();
        assert!(x.battles.is_empty());
    }
}
