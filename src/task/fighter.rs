use anyhow::Result;
use api::fighter::FighterResponse;
use backoff::{Error, ExponentialBackoff};
use chrono::{DateTime, Utc};
use erc_nft_metadata::AttributeEntry;
use ethers_core::types::Address;
use futures::{stream, StreamExt};
use itertools::Itertools;
use sea_orm::sea_query::OnConflict;
use serde::Deserialize;
use serde_json::Value;
use std::str::FromStr;
use std::time::Duration;
use tracing::{info, instrument, warn};

use entity::entities::{fighter, fighter_parent, fighter_trait, prelude::*};
use sea_orm::ActiveValue::*;
use sea_orm::{prelude::*, QueryOrder};

const CONCURRENT_REQUESTS: usize = 128;
/// 2 hours
const SCRAPE_INTERVAL: u64 = 2 * 60 * 60 * 1000;
const SUMMONED_CHAMPIONS_CONTRACT: &str = "0x57f698d99d964aef66d974739b98ec694724b1b8";

#[derive(Debug)]
pub struct ChampionTask {
    client: reqwest::Client,
    conn: DatabaseConnection,
    alchemy_api_key: String,
}

impl ChampionTask {
    pub fn new(client: reqwest::Client, conn: DatabaseConnection, alchemy_api_key: String) -> Self {
        Self {
            client,
            conn,
            alchemy_api_key,
        }
    }

    #[instrument(skip_all)]
    pub async fn run(self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(SCRAPE_INTERVAL));
        // Note that the first tick completes immediately.
        loop {
            interval.tick().await;
            self.scan().await?;
            info!("champion scan complete");
        }
    }

    async fn scan(&self) -> Result<()> {
        let count = self.get_count().await?;
        info!(count = ?count, "highest token id");

        let mut champions = vec![];
        let mut traits = vec![];
        let mut parents = vec![];

        for (fighter, dt) in self.scrape_champions(count).await.into_iter() {
            if let Some(lineage_node) = fighter.lineage_node {
                parents.push(fighter_parent::ActiveModel {
                    fighter_id: Set(fighter.attributes.id as i64),
                    parent_id: Set(lineage_node.parents[0] as i64),
                });

                parents.push(fighter_parent::ActiveModel {
                    fighter_id: Set(fighter.attributes.id as i64),
                    parent_id: Set(lineage_node.parents[1] as i64),
                });
            }

            champions.push(fighter::ActiveModel {
                id: Set(fighter.attributes.id as i64),
                wisdom_point: Set(fighter.statistic.wisdom.point as i32),
                strength_from: Set(fighter.statistic.wisdom.strength.from as i32),
                strength_to: Set(fighter.statistic.wisdom.strength.to as i32),
                attack_from: Set(fighter.statistic.wisdom.attack.from as i32),
                attack_to: Set(fighter.statistic.wisdom.attack.to as i32),
                defence_from: Set(fighter.statistic.wisdom.defence.from as i32),
                defence_to: Set(fighter.statistic.wisdom.defence.to as i32),
                omega_from: Set(fighter.statistic.wisdom.omega.from as i32),
                omega_to: Set(fighter.statistic.wisdom.omega.to as i32),
                last_updated: Set(dt.naive_utc()),
                mum: Set(fighter.lineage_node.map(|l| l.original_mum as i64)),
            });

            traits.extend(
                fighter
                    .attributes
                    .attributes
                    .attributes
                    .into_iter()
                    .map(|x| match x {
                        AttributeEntry::String { trait_type, value } => {
                            fighter_trait::ActiveModel {
                                fighter_id: Set(fighter.attributes.id as i64),
                                trait_type: Set(trait_type),
                                value: Set(value),
                            }
                        }
                        AttributeEntry::Number {
                            trait_type,
                            value,
                            display_type: _,
                        } => fighter_trait::ActiveModel {
                            fighter_id: Set(fighter.attributes.id as i64),
                            trait_type: Set(trait_type),
                            value: Set(value.to_string()),
                        },
                    }),
            )
        }

        let champions = champions
            .into_iter()
            .chunks(100)
            .into_iter()
            .map(|ck| ck.collect::<Vec<_>>())
            .collect::<Vec<_>>();

        for chunk in champions {
            let _ = Fighter::insert_many(chunk)
                .on_conflict(
                    OnConflict::column(fighter::Column::Id)
                        .update_columns([
                            fighter::Column::AttackFrom,
                            fighter::Column::AttackTo,
                            fighter::Column::DefenceFrom,
                            fighter::Column::DefenceTo,
                            fighter::Column::OmegaFrom,
                            fighter::Column::OmegaTo,
                            fighter::Column::StrengthFrom,
                            fighter::Column::StrengthTo,
                            fighter::Column::WisdomPoint,
                        ])
                        .to_owned(),
                )
                .exec(&self.conn)
                .await?;
        }

        let traits = traits
            .into_iter()
            .chunks(100)
            .into_iter()
            .map(|ck| ck.collect::<Vec<_>>())
            .collect::<Vec<_>>();

        for chunk in traits {
            let _ = FighterTrait::insert_many(chunk)
                .on_conflict(
                    OnConflict::columns([
                        fighter_trait::Column::FighterId,
                        fighter_trait::Column::TraitType,
                    ])
                    .update_columns([
                        fighter_trait::Column::FighterId,
                        fighter_trait::Column::TraitType,
                        fighter_trait::Column::Value,
                    ])
                    .to_owned(),
                )
                .exec(&self.conn)
                .await?;
        }

        let parents = parents
            .into_iter()
            .chunks(100)
            .into_iter()
            .map(|ck| ck.collect::<Vec<_>>())
            .collect::<Vec<_>>();

        for chunk in parents {
            let _ = FighterParent::insert_many(chunk)
                .on_conflict(
                    OnConflict::columns([
                        fighter_parent::Column::FighterId,
                        fighter_parent::Column::ParentId,
                    ])
                    .update_columns([
                        fighter_parent::Column::FighterId,
                        fighter_parent::Column::ParentId,
                    ])
                    .to_owned(),
                )
                .exec(&self.conn)
                .await?;
        }

        Ok(())
    }

    async fn get_count(&self) -> Result<u64> {
        let contract_address =
            Address::from_str(SUMMONED_CHAMPIONS_CONTRACT).expect("invalid contract address");

        // Fetch last known highest counter
        let mut last_highest = Fighter::find()
            .order_by_desc(fighter::Column::Id)
            .one(&self.conn)
            .await?
            .map(|m| m.id as u64)
            // Checkpoint value as of 2023-01-15
            .unwrap_or(29000);

        // Use pagination to search
        loop {
            let res = get_nfts_for_collection(
                &self.client,
                &self.alchemy_api_key,
                contract_address,
                last_highest,
            )
            .await?;

            match res.next_token {
                Some(next) => {
                    let next = next.trim_start_matches("0x");
                    last_highest = u64::from_str_radix(next, 16)?;
                    continue;
                }
                None => {
                    let n = res.nfts.len() - 1;
                    last_highest += n as u64;
                    break;
                }
            }
        }

        Ok(last_highest)
        // Ok(1000)
    }

    async fn scrape_champions(&self, up_to: u64) -> Vec<(FighterResponse, DateTime<Utc>)> {
        stream::iter(0..up_to)
            .map(|i| async move {
                let resp = self.get_champion(i).await?;
                info!(n = ?i, "completed");
                Ok::<(FighterResponse, DateTime<Utc>), reqwest::Error>((resp, Utc::now()))
            })
            .buffer_unordered(CONCURRENT_REQUESTS)
            .collect::<Vec<Result<_, _>>>()
            .await
            .into_iter()
            .filter_map(|x| x.ok())
            .collect()
    }

    async fn get_champion(&self, id: u64) -> Result<FighterResponse, reqwest::Error> {
        backoff::future::retry(ExponentialBackoff::default(), || async {
            info!(id = ?id, "sending request");
            self.client
                .get(format!(
                    "https://federation22.theredvillage.com/api/v2/champions/id/{}",
                    id
                ))
                .send()
                .await
                .and_then(|resp| resp.error_for_status())
                .map(|resp| resp.json::<FighterResponse>())
                .map_err(|e| {
                    warn!(e = ?e);
                    e
                })?
                .await
                // No point in trying again if metadata is invalid
                .map_err(|e| {
                    warn!(id = ?id, "invalid metadata (perhaps token doesn't exist)");
                    Error::Permanent(e)
                })
        })
        .await
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetNftsForCollection {
    nfts: Vec<Value>,
    // Pagination
    next_token: Option<String>,
}

async fn get_nfts_for_collection(
    client: &reqwest::Client,
    key: &str,
    contract_address: Address,
    start_token: u64,
) -> Result<GetNftsForCollection, reqwest::Error> {
    backoff::future::retry(ExponentialBackoff::default(), || async {
        client
            .get(format!(
                "https://polygon-mainnet.g.alchemy.com/nft/v2/{}/getNFTsForCollection",
                key
            ))
            .query(&[
                ("contractAddress", format!("{:?}", contract_address)),
                ("withMetadata", "false".to_owned()),
                ("startToken", start_token.to_string()),
            ])
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .map(|resp| resp.json::<GetNftsForCollection>())?
            .await
            .map_err(Error::Permanent)
    })
    .await
}
