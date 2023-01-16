use ethers_core::types::Address;
use serde::{de::Error, Deserialize, Serialize};
use serde_json::Value;

use crate::Pagination;

#[derive(Clone, Deserialize)]
pub struct TournamentResponse {
    #[serde(flatten)]
    pub pagination: Pagination,

    pub items: Vec<Tournament>,
}

#[derive(Debug, Clone)]
pub enum Tournament {
    OneVOne {
        tournament_id: i64,
        configs: Value,
        key: String,
        level: Level,
        modified: String,
        restrictions: Value,
        solo_optionals: Value,
        start_time: String,
        status: Status,
        solo_warriors: Vec<SoloWarrior>,
    },
    Blooding {
        tournament_id: i64,
        class: Value,
        configs: Value,
        key: String,
        legacy: bool,
        level: Level,
        modified: String,
        name: String,
        restrictions: Value,
        start_time: String,
        status: Status,
        tournament_type: String,
        warriors: Vec<Warrior>,
    },
    Bloodbath {
        tournament_id: i64,
        class: Value,
        configs: Value,
        key: String,
        legacy: bool,
        level: Level,
        modified: String,
        name: String,
        restrictions: Value,
        start_time: String,
        status: Status,
        tournament_type: String,
        warriors: Vec<Warrior>,
    },
    BloodElo {
        tournament_id: i64,
        class: Value,
        configs: Value,
        key: String,
        legacy: bool,
        level: Level,
        modified: String,
        name: String,
        restrictions: Value,
        start_time: String,
        status: Status,
        tournament_type: String,
        warriors: Vec<Warrior>,
    },
}

impl Tournament {
    pub fn service_id(&self) -> u64 {
        match self {
            Tournament::OneVOne { .. } => 0,
            Tournament::Blooding { .. } => 1,
            Tournament::Bloodbath { .. } => 2,
            Tournament::BloodElo { .. } => 3,
        }
    }
}

impl<'de> Deserialize<'de> for Tournament {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Has all the fields of Tournament. Non-common fields are Optional.
        #[derive(Clone, Debug, Deserialize)]
        struct KitchenSink {
            service_id: u64,
            tournament_id: i64,
            class: Option<Value>,
            configs: Value,
            key: String,
            legacy: Option<bool>,
            level: Level,
            modified: String,
            name: Option<String>,
            restrictions: Value,
            start_time: String,
            status: Status,
            tournament_type: Option<String>,
            warriors: Vec<Warrior>,
            solo_warriors: Vec<SoloWarrior>,
            solo_optionals: Option<Value>,
        }
        let sink: KitchenSink = Deserialize::deserialize(deserializer)?;

        match sink.service_id {
            0 => Ok(Tournament::OneVOne {
                tournament_id: sink.tournament_id,
                configs: sink.configs,
                key: sink.key,
                level: sink.level,
                modified: sink.modified,
                restrictions: sink.restrictions,
                solo_optionals: sink
                    .solo_optionals
                    .ok_or_else(|| D::Error::custom("expected solo_optional"))?,
                start_time: sink.start_time,
                status: sink.status,
                solo_warriors: sink.solo_warriors,
            }),
            1 => Ok(Tournament::Blooding {
                tournament_id: sink.tournament_id,
                configs: sink.configs,
                key: sink.key,
                level: sink.level,
                modified: sink.modified,
                restrictions: sink.restrictions,
                start_time: sink.start_time,
                status: sink.status,
                warriors: sink.warriors,
                class: sink
                    .class
                    .ok_or_else(|| D::Error::custom("expected class"))?,
                legacy: sink.legacy.unwrap_or_default(),
                name: sink.name.ok_or_else(|| D::Error::custom("expected name"))?,
                tournament_type: sink
                    .tournament_type
                    .ok_or_else(|| D::Error::custom("expected tournament type"))?,
            }),
            2 => Ok(Tournament::Bloodbath {
                tournament_id: sink.tournament_id,
                configs: sink.configs,
                key: sink.key,
                level: sink.level,
                modified: sink.modified,
                restrictions: sink.restrictions,
                start_time: sink.start_time,
                status: sink.status,
                warriors: sink.warriors,
                class: sink
                    .class
                    .ok_or_else(|| D::Error::custom("expected class"))?,
                legacy: sink.legacy.unwrap_or_default(),
                name: sink.name.ok_or_else(|| D::Error::custom("expected name"))?,
                tournament_type: sink
                    .tournament_type
                    .ok_or_else(|| D::Error::custom("expected tournament type"))?,
            }),
            3 => Ok(Tournament::BloodElo {
                tournament_id: sink.tournament_id,
                configs: sink.configs,
                key: sink.key,
                level: sink.level,
                modified: sink.modified,
                restrictions: sink.restrictions,
                start_time: sink.start_time,
                status: sink.status,
                warriors: sink.warriors,
                class: sink
                    .class
                    .ok_or_else(|| D::Error::custom("expected class"))?,
                legacy: sink.legacy.unwrap_or_default(),
                name: sink.name.ok_or_else(|| D::Error::custom("expected name"))?,
                tournament_type: sink
                    .tournament_type
                    .ok_or_else(|| D::Error::custom("expected tournament type"))?,
            }),
            _ => Err(D::Error::custom(format!(
                "{} not a valid service id",
                sink.service_id
            ))),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Warrior {
    account: Address,
    id: u64,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SoloWarrior {
    id: u64,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "COMPLETE_SUCCEED")]
    Completed,
    #[serde(rename = "CANCEL_SUCCEED")]
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub nav_key: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_1V1: &str = include_str!("tests/1v1.json");
    const TEST_BLOODING: &str = include_str!("tests/blooding.json");
    const TEST_BLOODBATH: &str = include_str!("tests/bloodbath.json");
    const TEST_BLOODELO: &str = include_str!("tests/bloodelo.json");

    const TEST_1V1_LARGE: &str = include_str!("tests/1v1_large.json");
    const TEST_BLOODING_LARGE: &str = include_str!("tests/blooding_large.json");
    const TEST_BLOODBATH_LARGE: &str = include_str!("tests/bloodbath_large.json");
    const TEST_BLOODELO_LARGE: &str = include_str!("tests/bloodelo_large.json");

    #[test]
    fn test_1v1() {
        let _ = serde_json::from_str::<TournamentResponse>(TEST_1V1).unwrap();
    }

    #[test]
    fn test_blooding() {
        let _ = serde_json::from_str::<TournamentResponse>(TEST_BLOODING).unwrap();
    }

    #[test]
    fn test_bloodbath() {
        let _ = serde_json::from_str::<TournamentResponse>(TEST_BLOODBATH).unwrap();
    }

    #[test]
    fn test_bloodelo() {
        let _ = serde_json::from_str::<TournamentResponse>(TEST_BLOODELO).unwrap();
    }

    #[test]
    fn test_1v1_large() {
        let _ = serde_json::from_str::<TournamentResponse>(TEST_1V1_LARGE).unwrap();
    }

    #[test]
    fn test_blooding_large() {
        let _ = serde_json::from_str::<TournamentResponse>(TEST_BLOODING_LARGE).unwrap();
    }

    #[test]
    fn test_bloodbath_large() {
        let _ = serde_json::from_str::<TournamentResponse>(TEST_BLOODBATH_LARGE).unwrap();
    }

    #[test]
    fn test_bloodelo_large() {
        let _ = serde_json::from_str::<TournamentResponse>(TEST_BLOODELO_LARGE).unwrap();
    }
}
