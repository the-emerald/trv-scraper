use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Fighter::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Fighter::Id)
                            .big_unsigned()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Fighter::WisdomPoint).integer().not_null())
                    .col(ColumnDef::new(Fighter::StrengthFrom).integer().not_null())
                    .col(ColumnDef::new(Fighter::StrengthTo).integer().not_null())
                    .col(ColumnDef::new(Fighter::AttackFrom).integer().not_null())
                    .col(ColumnDef::new(Fighter::AttackTo).integer().not_null())
                    .col(ColumnDef::new(Fighter::DefenceFrom).integer().not_null())
                    .col(ColumnDef::new(Fighter::DefenceTo).integer().not_null())
                    .col(ColumnDef::new(Fighter::OmegaFrom).integer().not_null())
                    .col(ColumnDef::new(Fighter::OmegaTo).integer().not_null())
                    .col(ColumnDef::new(Fighter::LastUpdated).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(FighterTrait::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FighterTrait::FighterId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(FighterTrait::TraitType).string().not_null())
                    .col(ColumnDef::new(FighterTrait::Value).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-fighter_id-fighter_trait")
                            .from(FighterTrait::Table, FighterTrait::FighterId)
                            .to(Fighter::Table, Fighter::Id),
                    )
                    .primary_key(
                        Index::create()
                            .col(FighterTrait::FighterId)
                            .col(FighterTrait::TraitType),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FighterTrait::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Fighter::Table).to_owned())
            .await?;

        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Fighter {
    Table,
    Id,
    WisdomPoint,
    StrengthFrom,
    StrengthTo,
    AttackFrom,
    AttackTo,
    DefenceFrom,
    DefenceTo,
    OmegaFrom,
    OmegaTo,
    LastUpdated,
}

#[derive(Iden)]
enum FighterTrait {
    Table,
    FighterId,
    TraitType,
    Value,
}
