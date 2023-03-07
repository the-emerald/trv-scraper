use sea_orm_migration::prelude::*;

use crate::m20220101_000001_create_fighter_table::Fighter;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tournament::Table)
                    .col(
                        ColumnDef::new(Tournament::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
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
                    .col(ColumnDef::new(Tournament::Status).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TournamentWarrior::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TournamentWarrior::TournamentId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentWarrior::WarriorId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TournamentWarrior::Account).binary_len(20))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tournament_id-tournament_warrior")
                            .from(TournamentWarrior::Table, TournamentWarrior::TournamentId)
                            .to(Tournament::Table, Tournament::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-warrior_id-tournament_warrior")
                            .from(TournamentWarrior::Table, TournamentWarrior::WarriorId)
                            .to(Fighter::Table, Fighter::Id),
                    )
                    .primary_key(
                        Index::create()
                            .col(TournamentWarrior::TournamentId)
                            .col(TournamentWarrior::WarriorId),
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
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MetaFailedTournamentRequest::PageIndex)
                            .unsigned()
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
            .drop_table(Table::drop().table(TournamentWarrior::Table).to_owned())
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

        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Tournament {
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
}

#[derive(Iden)]
enum TournamentWarrior {
    Table,
    TournamentId,
    WarriorId,
    Account,
}

#[derive(Iden)]
enum MetaFailedTournamentRequest {
    Table,
    PageSize,
    PageIndex,
}
