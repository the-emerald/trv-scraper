use anyhow::Result;
use api::{
    tournament::{RawTournamentResponse, Status, Tournament, TournamentResponse},
    tournament_detail::TournamentDetailResponse,
};
use backoff::{Error, ExponentialBackoff};
use chrono::Utc;
use entity::entities::{
    meta_failed_tournament_request, meta_last_page, tournament, tournament_detail_attack,
    tournament_detail_champion, tournament_fighter,
};
use ethers_core::abi::AbiEncode;
use futures::{future, stream, StreamExt};
use itertools::Itertools;
use sea_orm::{
    sea_query::OnConflict, ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait,
    DatabaseBackend, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter, Set,
    Statement, TransactionTrait,
};
use tracing::{debug, info, instrument, warn};

use crate::CONCURRENT_REQUESTS;

#[derive(Debug)]
pub struct TournamentTask {
    client: reqwest::Client,
    conn: DatabaseConnection,
    page_size: u64,
}

impl TournamentTask {
    pub fn new(client: reqwest::Client, conn: DatabaseConnection, page_size: u64) -> Self {
        Self {
            client,
            conn,
            page_size,
        }
    }

    #[instrument(skip_all)]
    pub async fn scan(&self) -> Result<()> {
        info!("beginning tournament scan");

        let _ = self.retry_failed().await;

        let mut next_page_index = self.get_starting_page().await?;

        loop {
            info!(size = ?self.page_size, page = ?next_page_index, "scanning");

            let batch = match self
                .get_tournament_batch(self.page_size, next_page_index)
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    warn!(e = ?e);

                    if let Err(e) = self
                        .insert_failed_page(self.page_size, next_page_index)
                        .await
                    {
                        warn!(size = ?self.page_size, index = ?next_page_index, e = ?e, "could not register failed page. this page will not be tried again!");
                    }

                    next_page_index += 1;
                    continue;
                }
            };

            let batch = TournamentResponse {
                pagination: batch.pagination,
                items: batch.items.into_iter().enumerate().filter_map(|(idx, item)| match serde_json::from_value(item) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        warn!(size = ?self.page_size, page = ?next_page_index, index = ?idx, e = ?e, "could not deserialize, skipping. this tournament will not be tried again!");
                        None
                    }
                })
                .collect()
            };

            if let Err(e) = self.insert_tournament_batch(batch.items.clone()).await {
                warn!(page = ?batch.pagination, e = ?e, "page failed to insert");

                // If the page insert fails we will try again sometime later
                if let Err(e) = self
                    .insert_failed_page(self.page_size, next_page_index)
                    .await
                {
                    warn!(size = ?self.page_size, index = ?next_page_index, e = ?e, "could not register failed page. this page will not be tried again!");
                }
            }

            let details = stream::iter(batch.items)
                .filter(|b| future::ready(b.status() != Status::Cancelled))
                .map(|b| async move {
                    let id = b.id();
                    let service_id = b.service_id();

                    self.get_tournament_detail(id, service_id)
                        .await
                        .map(|res| (id, service_id, res))
                })
                .buffer_unordered(CONCURRENT_REQUESTS)
                .collect::<Vec<Result<_, _>>>()
                .await
                .into_iter()
                .filter_map(|x| x.ok())
                .collect::<Vec<_>>();

            for (id, service_id, detail) in details {
                if let Err(e) = self.insert_tournament_detail(id, service_id, detail).await {
                    warn!(e = ?e, id = id, service_id = service_id, "could not insert tournament detail")
                }
            }

            if !batch.pagination.has_next_page {
                break;
            } else {
                next_page_index += 1;
            }
        }

        self.conn
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                "TRUNCATE meta_last_page;".to_owned(),
            ))
            .await?;

        meta_last_page::ActiveModel {
            page_size: Set(self.page_size as i64),
            page_index: Set(next_page_index as i64),
        }
        .insert(&self.conn)
        .await?;

        Ok(())
    }

    /// Get the starting page number.
    async fn get_starting_page(&self) -> Result<u64> {
        let entry = meta_last_page::Entity::find()
            .one(&self.conn)
            .await?
            .map(|v| v.page_index * v.page_size)
            .unwrap_or_default();
        Ok(entry as u64 / self.page_size)
    }

    /// Insert a failed page into the database to be tried later.
    async fn insert_failed_page(&self, page_size: u64, page_index: u64) -> Result<(), DbErr> {
        use meta_failed_tournament_request::*;

        Entity::insert(meta_failed_tournament_request::ActiveModel {
            page_size: Set(page_size as i64),
            page_index: Set(page_index as i64),
        })
        .on_conflict(
            OnConflict::columns([Column::PageSize, Column::PageIndex])
                .do_nothing()
                .to_owned(),
        )
        .exec(&self.conn)
        .await?;

        Ok(())
    }

    /// Insert a response batch into the database.
    async fn insert_tournament_batch(&self, tournaments: Vec<Tournament>) -> Result<()> {
        let mut tournament_rows = vec![];
        let mut tournament_warrior_rows = vec![];
        let time = Utc::now();
        // let mut tournament_solo_warrior_rows = vec![];

        for tournament in tournaments {
            if tournament.status() == Status::Cancelled {
                continue;
            }

            let service_id = tournament.service_id();
            match tournament {
                Tournament::OneVOne {
                    tournament_id,
                    configs,
                    key,
                    level,
                    modified,
                    restrictions,
                    solo_optionals,
                    start_time,
                    status,
                    solo_warriors,
                } => {
                    tournament_warrior_rows.extend(solo_warriors.into_iter().map(|sw| {
                        tournament_fighter::ActiveModel {
                            tournament_id: Set(tournament_id),
                            tournament_service_id: Set(service_id as i32),
                            fighter_id: Set(sw.id as i64),
                            account: Set(None),
                        }
                    }));

                    tournament_rows.push(tournament::ActiveModel {
                        id: Set(tournament_id),
                        service_id: Set(service_id as i32),
                        currency: Set(configs.currency.as_bytes().to_vec()),
                        fee_percentage: Set(configs.fee_percentage as i32),
                        buy_in: Set(configs.buy_in.encode()),
                        top_up: Set(configs.top_up.encode()),
                        key: Set(key),
                        legacy: Set(None),
                        level: Set(level.nav_key),
                        modified: Set(modified.naive_utc()),
                        name: Set(None),
                        restrictions: Set(restrictions),
                        solo_optionals: Set(Some(solo_optionals)),
                        start_time: Set(start_time),
                        status: Set(status.into()),
                        meta_last_updated: Set(time.naive_utc()),
                    });
                }
                Tournament::Blooding {
                    tournament_id,
                    class: _,
                    configs,
                    key,
                    legacy,
                    level,
                    modified,
                    name,
                    restrictions,
                    start_time,
                    status,
                    tournament_type: _,
                    warriors,
                } => {
                    tournament_warrior_rows.extend(warriors.into_iter().map(|sw| {
                        tournament_fighter::ActiveModel {
                            tournament_id: Set(tournament_id),
                            tournament_service_id: Set(service_id as i32),
                            fighter_id: Set(sw.id as i64),
                            account: Set(Some(sw.account.as_bytes().to_vec())),
                        }
                    }));

                    tournament_rows.push(tournament::ActiveModel {
                        id: Set(tournament_id),
                        service_id: Set(service_id as i32),
                        currency: Set(configs.currency.as_bytes().to_vec()),
                        fee_percentage: Set(configs.fee_percentage as i32),
                        buy_in: Set(configs.buy_in.encode()),
                        top_up: Set(configs.top_up.encode()),
                        key: Set(key),
                        legacy: Set(Some(legacy)),
                        level: Set(level.nav_key),
                        modified: Set(modified.naive_utc()),
                        name: Set(Some(name)),
                        restrictions: Set(restrictions),
                        solo_optionals: Set(None),
                        start_time: Set(start_time),
                        status: Set(status.into()),
                        meta_last_updated: Set(time.naive_utc()),
                    });
                }
                Tournament::Bloodbath {
                    tournament_id,
                    class: _,
                    configs,
                    key,
                    legacy,
                    level,
                    modified,
                    name,
                    restrictions,
                    start_time,
                    status,
                    tournament_type: _,
                    warriors,
                } => {
                    tournament_warrior_rows.extend(warriors.into_iter().map(|sw| {
                        tournament_fighter::ActiveModel {
                            tournament_id: Set(tournament_id),
                            tournament_service_id: Set(service_id as i32),
                            fighter_id: Set(sw.id as i64),
                            account: Set(Some(sw.account.as_bytes().to_vec())),
                        }
                    }));

                    tournament_rows.push(tournament::ActiveModel {
                        id: Set(tournament_id),
                        service_id: Set(service_id as i32),
                        currency: Set(configs.currency.as_bytes().to_vec()),
                        fee_percentage: Set(configs.fee_percentage as i32),
                        buy_in: Set(configs.buy_in.encode()),
                        top_up: Set(configs.top_up.encode()),
                        key: Set(key),
                        legacy: Set(Some(legacy)),
                        level: Set(level.nav_key),
                        modified: Set(modified.naive_utc()),
                        name: Set(Some(name)),
                        restrictions: Set(restrictions),
                        solo_optionals: Set(None),
                        start_time: Set(start_time),
                        status: Set(status.into()),
                        meta_last_updated: Set(time.naive_utc()),
                    });
                }
                Tournament::BloodElo {
                    tournament_id,
                    class: _,
                    configs,
                    key,
                    legacy,
                    level,
                    modified,
                    name,
                    restrictions,
                    start_time,
                    status,
                    tournament_type: _,
                    warriors,
                } => {
                    tournament_warrior_rows.extend(warriors.into_iter().map(|sw| {
                        tournament_fighter::ActiveModel {
                            tournament_id: Set(tournament_id),
                            tournament_service_id: Set(service_id as i32),
                            fighter_id: Set(sw.id as i64),
                            account: Set(Some(sw.account.as_bytes().to_vec())),
                        }
                    }));

                    tournament_rows.push(tournament::ActiveModel {
                        id: Set(tournament_id),
                        service_id: Set(service_id as i32),
                        currency: Set(configs.currency.as_bytes().to_vec()),
                        fee_percentage: Set(configs.fee_percentage as i32),
                        buy_in: Set(configs.buy_in.encode()),
                        top_up: Set(configs.top_up.encode()),
                        key: Set(key),
                        legacy: Set(Some(legacy)),
                        level: Set(level.nav_key),
                        modified: Set(modified.naive_utc()),
                        name: Set(Some(name)),
                        restrictions: Set(restrictions),
                        solo_optionals: Set(None),
                        start_time: Set(start_time),
                        status: Set(status.into()),
                        meta_last_updated: Set(time.naive_utc()),
                    });
                }
                Tournament::DoubleUp {
                    tournament_id,
                    class: _,
                    configs,
                    key,
                    level,
                    modified,
                    name,
                    restrictions,
                    start_time,
                    status,
                    tournament_type: _,
                    warriors: _,
                } => {
                    tournament_rows.push(tournament::ActiveModel {
                        id: Set(tournament_id),
                        service_id: Set(service_id as i32),
                        currency: Set(configs.currency.as_bytes().to_vec()),
                        fee_percentage: Set(configs.fee_percentage as i32),
                        buy_in: Set(configs.buy_in.encode()),
                        top_up: Set(configs.top_up.encode()),
                        key: Set(key),
                        legacy: Set(None),
                        level: Set(level.nav_key),
                        modified: Set(modified.naive_utc()),
                        name: Set(Some(name)),
                        restrictions: Set(restrictions),
                        solo_optionals: Set(None),
                        start_time: Set(start_time),
                        status: Set(status.into()),
                        meta_last_updated: Set(time.naive_utc()),
                    });
                }
                Tournament::DoubleUpReverse {
                    tournament_id,
                    class: _,
                    configs,
                    key,
                    level,
                    modified,
                    name,
                    restrictions,
                    start_time,
                    status,
                    tournament_type: _,
                    warriors: _,
                } => {
                    tournament_rows.push(tournament::ActiveModel {
                        id: Set(tournament_id),
                        service_id: Set(service_id as i32),
                        currency: Set(configs.currency.as_bytes().to_vec()),
                        fee_percentage: Set(configs.fee_percentage as i32),
                        buy_in: Set(configs.buy_in.encode()),
                        top_up: Set(configs.top_up.encode()),
                        key: Set(key),
                        legacy: Set(None),
                        level: Set(level.nav_key),
                        modified: Set(modified.naive_utc()),
                        name: Set(Some(name)),
                        restrictions: Set(restrictions),
                        solo_optionals: Set(None),
                        start_time: Set(start_time),
                        status: Set(status.into()),
                        meta_last_updated: Set(time.naive_utc()),
                    });
                }
                Tournament::Traditional {
                    tournament_id,
                    class: _,
                    configs,
                    key,
                    level,
                    modified,
                    name,
                    restrictions,
                    start_time,
                    status,
                    tournament_type: _,
                    warriors: _,
                } => {
                    tournament_rows.push(tournament::ActiveModel {
                        id: Set(tournament_id),
                        service_id: Set(service_id as i32),
                        currency: Set(configs.currency.as_bytes().to_vec()),
                        fee_percentage: Set(configs.fee_percentage as i32),
                        buy_in: Set(configs.buy_in.encode()),
                        top_up: Set(configs.top_up.encode()),
                        key: Set(key),
                        legacy: Set(None),
                        level: Set(level.nav_key),
                        modified: Set(modified.naive_utc()),
                        name: Set(Some(name)),
                        restrictions: Set(restrictions),
                        solo_optionals: Set(None),
                        start_time: Set(start_time),
                        status: Set(status.into()),
                        meta_last_updated: Set(time.naive_utc()),
                    });
                }
            };
        }

        let tournament_rows = tournament_rows
            .into_iter()
            .chunks(100)
            .into_iter()
            .map(|ck| ck.collect::<Vec<_>>())
            .collect::<Vec<_>>();

        for chunk in tournament_rows {
            let _ = tournament::Entity::insert_many(chunk)
                .on_conflict(
                    OnConflict::columns([tournament::Column::Id, tournament::Column::ServiceId])
                        .update_columns([
                            tournament::Column::Id,
                            tournament::Column::ServiceId,
                            tournament::Column::Currency,
                            tournament::Column::FeePercentage,
                            tournament::Column::BuyIn,
                            tournament::Column::TopUp,
                            tournament::Column::Key,
                            tournament::Column::Legacy,
                            tournament::Column::Level,
                            tournament::Column::Modified,
                            tournament::Column::Name,
                            tournament::Column::Restrictions,
                            tournament::Column::SoloOptionals,
                            tournament::Column::StartTime,
                            tournament::Column::Status,
                            tournament::Column::MetaLastUpdated,
                        ])
                        .to_owned(),
                )
                .exec(&self.conn)
                .await
                .map_err(|e| {
                    warn!(e = ?e);
                    e
                })?;
        }

        let tournament_warrior_rows = tournament_warrior_rows
            .into_iter()
            .chunks(100)
            .into_iter()
            .map(|ck| ck.collect::<Vec<_>>())
            .collect::<Vec<_>>();

        for chunk in tournament_warrior_rows {
            let ids = chunk
                .iter()
                .map(|x| {
                    (
                        x.tournament_id.clone().unwrap(),
                        x.tournament_service_id.clone().unwrap(),
                    )
                })
                .collect::<Vec<_>>();
            self.conn
                .transaction::<_, (), DbErr>(|txn| {
                    Box::pin(async move {
                        // Remove all rows associated with the IDs
                        for (tid, sid) in ids {
                            tournament_fighter::Entity::delete_many()
                                .filter(
                                    Condition::all()
                                        .add(tournament_fighter::Column::TournamentId.eq(tid))
                                        .add(
                                            tournament_fighter::Column::TournamentServiceId.eq(sid),
                                        ),
                                )
                                .exec(txn)
                                .await?;
                        }

                        // Insert fresh warriors.
                        tournament_fighter::Entity::insert_many(chunk)
                            .on_conflict(
                                OnConflict::columns([
                                    tournament_fighter::Column::TournamentId,
                                    tournament_fighter::Column::TournamentServiceId,
                                    tournament_fighter::Column::FighterId,
                                ])
                                .update_columns([
                                    tournament_fighter::Column::TournamentId,
                                    tournament_fighter::Column::TournamentServiceId,
                                    tournament_fighter::Column::FighterId,
                                    tournament_fighter::Column::Account,
                                ])
                                .to_owned(),
                            )
                            .exec(txn)
                            .await?;

                        Ok(())
                    })
                })
                .await
                .map_err(|e| {
                    warn!(e = ?e);
                    e
                })?;
        }

        Ok(())
    }

    async fn get_tournament_batch(
        &self,
        page_size: u64,
        page_index: u64,
    ) -> Result<RawTournamentResponse, reqwest::Error> {
        backoff::future::retry(ExponentialBackoff::default(), || async {
            self.client
                .get("https://federation22.theredvillage.com/api/v2/tournaments".to_string())
                .query(&[("page_size", page_size), ("page_index", page_index)])
                .send()
                .await
                .and_then(|resp| resp.error_for_status())
                .map(|resp| resp.json::<RawTournamentResponse>())?
                .await
                .map_err(Error::Permanent)
        })
        .await
    }

    async fn retry_failed(&self) -> Result<()> {
        for page in meta_failed_tournament_request::Entity::find()
            .all(&self.conn)
            .await?
        {
            let batch = match self
                .get_tournament_batch(page.page_size as u64, page.page_index as u64)
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    warn!(e = ?e);
                    continue;
                }
            };

            let batch = TournamentResponse {
                pagination: batch.pagination,
                items: batch.items.into_iter().enumerate().filter_map(|(idx, item)| match serde_json::from_value(item) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        warn!(size = ?page.page_size, page = ?page.page_index, index = ?idx, e = ?e, "could not deserialize, skipping. this tournament will not be tried again!");
                        None
                    }
                })
                .collect()
            };

            if let Err(e) = self.insert_tournament_batch(batch.items).await {
                warn!(e = ?e);
                continue;
            }

            // If it's ok we delete the page.
            let _ = page.delete(&self.conn).await.map_err(|e| {
                warn!(e = ?e);
                e
            });
        }
        Ok(())
    }

    async fn get_tournament_detail(
        &self,
        id: i64,
        service_id: u64,
    ) -> Result<TournamentDetailResponse, reqwest::Error> {
        backoff::future::retry(ExponentialBackoff::default(), || async {
            self.client
                .get(format!("https://federation22.theredvillage.com/api/v2/battles/service/{service_id}/tournament/{id}"))
                .send()
                .await
                .and_then(|resp| resp.error_for_status())
                .map(|resp| resp.json::<TournamentDetailResponse>())?
                .await
                .map_err(|e| {
                    warn!(e = ?e, id = id, service_id = service_id, "could not get tournament detail");
                    Error::Permanent(e)
                })
        })
        .await
    }

    async fn insert_tournament_detail(
        &self,
        id: i64,
        service_id: u64,
        detail: TournamentDetailResponse,
    ) -> Result<(), DbErr> {
        debug!(id = id, service_id = service_id);

        // Champions
        let champions = detail
            .champions
            .iter()
            .map(|x| tournament_detail_champion::ActiveModel {
                tournament_id: Set(id),
                tournament_service_id: Set(service_id as i64),
                fighter_id: Set(x.token_id as i64),
                stance: Set(x.stance as i32),
            })
            .collect::<Vec<_>>();

        let _ = tournament_detail_champion::Entity::insert_many(champions)
            .on_conflict(
                OnConflict::columns([
                    tournament_detail_champion::Column::TournamentId,
                    tournament_detail_champion::Column::TournamentServiceId,
                    tournament_detail_champion::Column::FighterId,
                ])
                .update_columns([
                    tournament_detail_champion::Column::TournamentId,
                    tournament_detail_champion::Column::TournamentServiceId,
                    tournament_detail_champion::Column::FighterId,
                    tournament_detail_champion::Column::Stance,
                ])
                .to_owned(),
            )
            .exec(&self.conn)
            .await
            .map_err(|e| {
                warn!(e = ?e, id = id, service_id = service_id, "tournament_detail_champion");
                e
            });

        // Attack
        let attacks = detail
            .battles
            .iter()
            .flat_map(|battle| {
                battle.champions.iter().flat_map(|ca| {
                    ca.attack
                        .iter()
                        .map(|a| tournament_detail_attack::ActiveModel {
                            tournament_id: Set(id),
                            tournament_service_id: Set(service_id as i64),
                            fighter_id: Set(ca.id as i64),
                            round: Set(battle.round as i32),
                            special_attack: Set(a.special_attack),
                            speical_defend: Set(a.special_defend),
                            damage: Set(a.damage as i32),
                            order: Set(a.order as i32),
                        })
                })
            })
            .collect::<Vec<_>>();

        if !attacks.is_empty() {
            let _ = tournament_detail_attack::Entity::insert_many(attacks)
                .on_conflict(
                    OnConflict::columns([
                        tournament_detail_attack::Column::TournamentId,
                        tournament_detail_attack::Column::TournamentServiceId,
                        tournament_detail_attack::Column::Round,
                        tournament_detail_attack::Column::Order,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec(&self.conn)
                .await
                .map_err(|e| {
                    warn!(e = ?e, id = id, service_id = service_id, "tournament_detail_attack");
                    e
                });
        }

        Ok(())
    }
}
