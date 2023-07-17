use sea_orm_migration::{prelude::*, sea_query::extension::postgres::Type};

use crate::m20220101_000001_create_fighter_table::Fighter;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(TournamentStatus::Type)
                    .values([
                        TournamentStatus::Completed,
                        TournamentStatus::Cancelled,
                        TournamentStatus::Created,
                        TournamentStatus::Fought,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Tournament::Table)
                    .col(ColumnDef::new(Tournament::Id).big_integer().not_null())
                    .col(ColumnDef::new(Tournament::ServiceId).unsigned().not_null())
                    .col(
                        ColumnDef::new(Tournament::Currency)
                            .binary_len(20)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Tournament::FeePercentage)
                            .unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Tournament::BuyIn).binary_len(32).not_null())
                    .col(ColumnDef::new(Tournament::TopUp).binary_len(32).not_null())
                    .col(ColumnDef::new(Tournament::Key).string().not_null())
                    .col(ColumnDef::new(Tournament::Legacy).boolean())
                    .col(ColumnDef::new(Tournament::Level).string().not_null())
                    .col(ColumnDef::new(Tournament::Modified).date_time().not_null())
                    .col(ColumnDef::new(Tournament::Name).string())
                    .col(
                        ColumnDef::new(Tournament::Restrictions)
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Tournament::SoloOptionals).json_binary())
                    .col(ColumnDef::new(Tournament::StartTime).date_time().not_null())
                    .col(
                        ColumnDef::new(Tournament::Status)
                            .custom(Alias::new("tournament_status"))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Tournament::MetaLastUpdated)
                            .date_time()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(Tournament::Id)
                            .col(Tournament::ServiceId),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TournamentFighter::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TournamentFighter::TournamentId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentFighter::TournamentServiceId)
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentFighter::FighterId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TournamentFighter::Account).binary_len(20))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tournament_id-tournament_fighter")
                            .from(
                                TournamentFighter::Table,
                                (
                                    TournamentFighter::TournamentId,
                                    TournamentFighter::TournamentServiceId,
                                ),
                            )
                            .to(Tournament::Table, (Tournament::Id, Tournament::ServiceId)),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-fighter_id-tournament_fighter")
                            .from(TournamentFighter::Table, TournamentFighter::FighterId)
                            .to(Fighter::Table, Fighter::Id),
                    )
                    .primary_key(
                        Index::create()
                            .col(TournamentFighter::TournamentId)
                            .col(TournamentFighter::TournamentServiceId)
                            .col(TournamentFighter::FighterId),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(MetaFailedTournamentRequest::Table)
                    .col(
                        ColumnDef::new(MetaFailedTournamentRequest::PageSize)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MetaFailedTournamentRequest::PageIndex)
                            .big_unsigned()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(MetaFailedTournamentRequest::PageSize)
                            .col(MetaFailedTournamentRequest::PageIndex),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(MetaLastPage::Table)
                    .col(
                        ColumnDef::new(MetaLastPage::PageSize)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MetaLastPage::PageIndex)
                            .big_unsigned()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(MetaFailedTournamentRequest::PageSize)
                            .col(MetaFailedTournamentRequest::PageIndex),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TournamentFighter::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Tournament::Table).to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(MetaFailedTournamentRequest::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(MetaLastPage::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(TournamentStatus::Type).to_owned())
            .await?;

        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub(crate) enum Tournament {
    Table,
    Id,
    ServiceId,
    Currency,
    FeePercentage,
    BuyIn,
    TopUp,
    Key,
    Legacy, // bool, nullable
    Level,
    Modified,
    Name, // string, nullable
    Restrictions,
    SoloOptionals, // json, nullable
    StartTime,
    Status,
    MetaLastUpdated,
}

#[derive(Iden)]
pub enum TournamentFighter {
    Table,
    TournamentId,
    TournamentServiceId,
    FighterId,
    Account,
}

#[derive(Iden)]
enum MetaFailedTournamentRequest {
    Table,
    PageSize,
    PageIndex,
}

enum TournamentStatus {
    Type,
    Completed,
    Cancelled,
    Created,
    Fought,
}

impl Iden for TournamentStatus {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                TournamentStatus::Type => "tournament_status",
                TournamentStatus::Completed => "completed",
                TournamentStatus::Cancelled => "cancelled",
                TournamentStatus::Created => "created",
                TournamentStatus::Fought => "fought",
            }
        )
        .unwrap()
    }
}

#[derive(Iden)]
enum MetaLastPage {
    Table,
    PageSize,
    PageIndex,
}
