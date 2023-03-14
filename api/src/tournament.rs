use chrono::{DateTime, NaiveDateTime, Utc};
use entity::entities::sea_orm_active_enums::TournamentStatus;
use ethers_core::types::{Address, U256};
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
        configs: Configs,
        key: String,
        level: Level,
        modified: DateTime<Utc>,
        restrictions: Value,
        solo_optionals: Value,
        start_time: NaiveDateTime,
        status: Status,
        solo_warriors: Vec<SoloWarrior>,
    },
    Blooding {
        tournament_id: i64,
        class: Value,
        configs: Configs,
        key: String,
        legacy: bool,
        level: Level,
        modified: DateTime<Utc>,
        name: String,
        restrictions: Value,
        start_time: NaiveDateTime,
        status: Status,
        tournament_type: String,
        warriors: Vec<Warrior>,
    },
    Bloodbath {
        tournament_id: i64,
        class: Value,
        configs: Configs,
        key: String,
        legacy: bool,
        level: Level,
        modified: DateTime<Utc>,
        name: String,
        restrictions: Value,
        start_time: NaiveDateTime,
        status: Status,
        tournament_type: String,
        warriors: Vec<Warrior>,
    },
    BloodElo {
        tournament_id: i64,
        class: Value,
        configs: Configs,
        key: String,
        legacy: bool,
        level: Level,
        modified: DateTime<Utc>,
        name: String,
        restrictions: Value,
        start_time: NaiveDateTime,
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

    pub fn status(&self) -> Status {
        match self {
            Tournament::OneVOne {
                tournament_id: _,
                configs: _,
                key: _,
                level: _,
                modified: _,
                restrictions: _,
                solo_optionals: _,
                start_time: _,
                status,
                solo_warriors: _,
            } => *status,
            Tournament::Blooding {
                tournament_id: _,
                class: _,
                configs: _,
                key: _,
                legacy: _,
                level: _,
                modified: _,
                name: _,
                restrictions: _,
                start_time: _,
                status,
                tournament_type: _,
                warriors: _,
            } => *status,
            Tournament::Bloodbath {
                tournament_id: _,
                class: _,
                configs: _,
                key: _,
                legacy: _,
                level: _,
                modified: _,
                name: _,
                restrictions: _,
                start_time: _,
                status,
                tournament_type: _,
                warriors: _,
            } => *status,
            Tournament::BloodElo {
                tournament_id: _,
                class: _,
                configs: _,
                key: _,
                legacy: _,
                level: _,
                modified: _,
                name: _,
                restrictions: _,
                start_time: _,
                status,
                tournament_type: _,
                warriors: _,
            } => *status,
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
            configs: Configs,
            key: String,
            legacy: Option<bool>,
            level: Level,
            modified: DateTime<Utc>,
            name: Option<String>,
            restrictions: Value,
            #[serde(with = "start_time_dt_format")]
            start_time: NaiveDateTime,
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

pub mod start_time_dt_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M";

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Warrior {
    pub account: Address,
    pub id: u64,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SoloWarrior {
    pub id: u64,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Status {
    #[serde(rename = "COMPLETE_SUCCEED")]
    Completed,
    #[serde(rename = "CANCEL_SUCCEED")]
    Cancelled,
}

impl From<Status> for TournamentStatus {
    fn from(value: Status) -> Self {
        match value {
            Status::Completed => Self::Completed,
            Status::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub nav_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configs {
    pub currency: Address,
    pub fee_percentage: u64,
    #[serde(with = "crate::util::u256_fromstr_radix_10")]
    pub buy_in: U256,
    #[serde(with = "crate::util::u256_fromstr_radix_10")]
    pub top_up: U256,
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
