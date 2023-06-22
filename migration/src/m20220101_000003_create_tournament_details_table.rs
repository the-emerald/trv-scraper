use sea_orm_migration::prelude::*;

use crate::{
    m20220101_000001_create_fighter_table::Fighter,
    m20220101_000002_create_tournament_table::Tournament,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TournamentDetailChampion::Table)
                    .col(
                        ColumnDef::new(TournamentDetailChampion::TournamentId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentDetailChampion::TournamentServiceId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentDetailChampion::FighterId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentDetailChampion::Stance)
                            .unsigned()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tournament_id-tournament_detail_champion")
                            .from(
                                TournamentDetailChampion::Table,
                                (
                                    TournamentDetailChampion::TournamentId,
                                    TournamentDetailChampion::TournamentServiceId,
                                ),
                            )
                            .to(Tournament::Table, (Tournament::Id, Tournament::ServiceId)),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-fighter_id-tournament_detail_champion")
                            .from(
                                TournamentDetailChampion::Table,
                                TournamentDetailChampion::FighterId,
                            )
                            .to(Fighter::Table, Fighter::Id),
                    )
                    .primary_key(
                        Index::create()
                            .col(TournamentDetailChampion::TournamentId)
                            .col(TournamentDetailChampion::TournamentServiceId)
                            .col(TournamentDetailChampion::FighterId),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TournamentDetailRound::Table)
                    .col(
                        ColumnDef::new(TournamentDetailRound::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TournamentDetailRound::TournamentId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentDetailRound::TournamentServiceId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentDetailRound::Round)
                            .unsigned()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tournament_id-tournament_detail_round")
                            .from(
                                TournamentDetailRound::Table,
                                (
                                    TournamentDetailRound::TournamentId,
                                    TournamentDetailRound::TournamentServiceId,
                                ),
                            )
                            .to(Tournament::Table, (Tournament::Id, Tournament::ServiceId)),
                    )
                    .index(
                        Index::create()
                            .name("idx-tournament_detail_round")
                            .unique()
                            .col(TournamentDetailRound::TournamentId)
                            .col(TournamentDetailRound::TournamentServiceId)
                            .col(TournamentDetailRound::Round),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TournamentDetailAttack::Table)
                    .col(
                        ColumnDef::new(TournamentDetailAttack::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TournamentDetailAttack::FighterId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentDetailAttack::RoundId)
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentDetailAttack::SpecialAttack)
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentDetailAttack::SpeicalDefend)
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentDetailAttack::Damage)
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentDetailAttack::Order)
                            .unsigned()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-fighter_id-tournament_detail_attack")
                            .from(
                                TournamentDetailAttack::Table,
                                TournamentDetailAttack::FighterId,
                            )
                            .to(Fighter::Table, Fighter::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-round_id-tournament_detail_attack")
                            .from(
                                TournamentDetailAttack::Table,
                                TournamentDetailAttack::RoundId,
                            )
                            .to(TournamentDetailRound::Table, TournamentDetailRound::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(TournamentDetailChampion::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(TournamentDetailAttack::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum TournamentDetailChampion {
    Table,
    TournamentId,
    TournamentServiceId,
    FighterId,
    Stance,
}

#[derive(Iden)]
enum TournamentDetailRound {
    Table,
    Id,
    TournamentId,
    TournamentServiceId,
    Round,
}

#[derive(Iden)]
enum TournamentDetailAttack {
    Table,
    Id,
    FighterId,
    RoundId,
    SpecialAttack,
    SpeicalDefend,
    Damage,
    Order,
}
