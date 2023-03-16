use anyhow::Result;
use api::tournament::{RawTournamentResponse, Status, Tournament, TournamentResponse};
use backoff::{Error, ExponentialBackoff};
use chrono::Utc;
use entity::entities::{meta_failed_tournament_request, tournament, tournament_warrior};
use ethers_core::abi::AbiEncode;
use itertools::Itertools;
use sea_orm::{
    sea_query::OnConflict, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, Set, TransactionTrait,
};
use tracing::{info, instrument, warn};

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

            if let Err(e) = self.insert_into_database(batch.items).await {
                warn!(page = ?batch.pagination, e = ?e, "page failed to insert");

                // If the page insert fails we will try again sometime later
                if let Err(e) = self
                    .insert_failed_page(self.page_size, next_page_index)
                    .await
                {
                    warn!(size = ?self.page_size, index = ?next_page_index, e = ?e, "could not register failed page. this page will not be tried again!");
                }
            }

            if !batch.pagination.has_next_page {
                break;
            } else {
                next_page_index += 1;
            }
        }

        Ok(())
    }

    /// Get the starting page number.
    /// Note that this value may result in rescanning of existing tournaments, but should never miss a page.
    async fn get_starting_page(&self) -> Result<u64> {
        let rows = tournament::Entity::find().count(&self.conn).await?;
        Ok(rows / self.page_size)
    }

    /// Insert a failed page into the database to be tried later.
    async fn insert_failed_page(&self, page_size: u64, page_index: u64) -> Result<(), DbErr> {
        use meta_failed_tournament_request::*;

        Entity::insert(meta_failed_tournament_request::ActiveModel {
            page_size: Set(page_size as i32),
            page_index: Set(page_index as i32),
        })
        .on_conflict(
            OnConflict::columns([Column::PageIndex, Column::PageSize])
                .do_nothing()
                .to_owned(),
        )
        .exec(&self.conn)
        .await?;

        Ok(())
    }

    /// Insert a response batch into the database.
    async fn insert_into_database(&self, tournaments: Vec<Tournament>) -> Result<()> {
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
                        tournament_warrior::ActiveModel {
                            tournament_id: Set(tournament_id),
                            tournament_service_id: Set(service_id as i32),
                            warrior_id: Set(sw.id as i64),
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
                        tournament_warrior::ActiveModel {
                            tournament_id: Set(tournament_id),
                            tournament_service_id: Set(service_id as i32),
                            warrior_id: Set(sw.id as i64),
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
                        tournament_warrior::ActiveModel {
                            tournament_id: Set(tournament_id),
                            tournament_service_id: Set(service_id as i32),
                            warrior_id: Set(sw.id as i64),
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
                        tournament_warrior::ActiveModel {
                            tournament_id: Set(tournament_id),
                            tournament_service_id: Set(service_id as i32),
                            warrior_id: Set(sw.id as i64),
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
                            tournament_warrior::Entity::delete_many()
                                .filter(
                                    Condition::all()
                                        .add(tournament_warrior::Column::TournamentId.eq(tid))
                                        .add(
                                            tournament_warrior::Column::TournamentServiceId.eq(sid),
                                        ),
                                )
                                .exec(txn)
                                .await?;
                        }

                        // // Remove all rows associated with the IDs
                        // tournament_warrior::Entity::delete_many()
                        //     .filter(tournament_warrior::Column::TournamentId.is_in(ids))
                        //     .exec(txn)
                        //     .await?;

                        // Insert fresh warriors.
                        tournament_warrior::Entity::insert_many(chunk)
                            .on_conflict(
                                OnConflict::columns([
                                    tournament_warrior::Column::TournamentId,
                                    tournament_warrior::Column::TournamentServiceId,
                                    tournament_warrior::Column::WarriorId,
                                ])
                                .update_columns([
                                    tournament_warrior::Column::TournamentId,
                                    tournament_warrior::Column::TournamentServiceId,
                                    tournament_warrior::Column::WarriorId,
                                    tournament_warrior::Column::Account,
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
            // .map(|raw| raw.into())
        })
        .await
    }
}
