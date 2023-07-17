use sea_orm_migration::{prelude::*, sea_orm::ConnectionTrait};

use crate::{
    m20220101_000001_create_fighter_table::Fighter,
    m20220101_000002_create_tournament_table::{Tournament, TournamentFighter},
    m20220101_000003_create_tournament_details_table::TournamentDetailChampion,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create stance column
        manager
            .alter_table(
                Table::alter()
                    .table(TournamentFighter::Table)
                    .add_column(ColumnDef::new(Alias::new("stance")).unsigned().not_null())
                    .to_owned(),
            )
            .await?;

        // Move stance to tournament fighter table
        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE tournament_fighter tf
            SET stance = tdc.stance
            FROM tournament_detail_champion tdc
            WHERE tdc.tournament_id = tf.tournament_id AND
            tdc.tournament_service_id = tf.tournament_service_id AND
            tdc.fighter_id = tf.fighter_id",
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(TournamentDetailChampion::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Recreate `tournament_detail_champion`
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

        // Move stance to detail table
        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE tournament_detail_champion tdc
            SET stance = tf.stance
            FROM tournament_fighter tf
            WHERE tdc.tournament_id = tf.tournament_id AND
            tdc.tournament_service_id = tf.tournament_service_id AND
            tdc.fighter_id = tf.fighter_id",
            )
            .await?;

        // Drop stance column
        manager
            .alter_table(
                Table::alter()
                    .table(TournamentFighter::Table)
                    .drop_column(Alias::new("stance"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

// /// Learn more at https://docs.rs/sea-query#iden
// #[derive(Iden)]
// enum TournamentDetailChampion {
//     Table,
//     TournamentId,
//     TournamentServiceId,
//     FighterId,
//     Stance,
// }

// #[derive(Iden)]
// enum TournamentDetailAttack {
//     Table,
//     TournamentId,        // p
//     TournamentServiceId, // p
//     FighterId,
//     Round, // p
//     Order, // p
//     SpecialAttack,
//     SpeicalDefend,
//     Damage,
// }
