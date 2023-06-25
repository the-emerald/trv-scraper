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
                    .table(TournamentDetailAttack::Table)
                    .col(
                        ColumnDef::new(TournamentDetailAttack::TournamentId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentDetailAttack::TournamentServiceId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentDetailAttack::FighterId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TournamentDetailAttack::Round)
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
                            .name("fk-tournament_id-tournament_detail_attack")
                            .from(
                                TournamentDetailAttack::Table,
                                (
                                    TournamentDetailAttack::TournamentId,
                                    TournamentDetailAttack::TournamentServiceId,
                                ),
                            )
                            .to(Tournament::Table, (Tournament::Id, Tournament::ServiceId)),
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
                    .primary_key(
                        Index::create()
                            .col(TournamentDetailAttack::TournamentId)
                            .col(TournamentDetailAttack::TournamentServiceId)
                            .col(TournamentDetailAttack::Round)
                            .col(TournamentDetailAttack::Order),
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
pub enum TournamentDetailChampion {
    Table,
    TournamentId,
    TournamentServiceId,
    FighterId,
    Stance,
}

#[derive(Iden)]
enum TournamentDetailAttack {
    Table,
    TournamentId,        // p
    TournamentServiceId, // p
    FighterId,
    Round, // p
    Order, // p
    SpecialAttack,
    SpeicalDefend,
    Damage,
}
