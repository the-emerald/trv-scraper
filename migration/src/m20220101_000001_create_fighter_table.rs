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
                    .col(ColumnDef::new(Fighter::Mum).big_unsigned())
                    .col(
                        ColumnDef::new(Fighter::MetaLastUpdated)
                            .date_time()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-mum-fighter")
                            .from(Fighter::Table, Fighter::Mum)
                            .to(Fighter::Table, Fighter::Id),
                    )
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

        manager
            .create_table(
                Table::create()
                    .table(FighterParent::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FighterParent::FighterId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FighterParent::ParentId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(FighterParent::FighterId)
                            .col(FighterParent::ParentId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-fighter_id-fighter_parent")
                            .from(FighterParent::Table, FighterParent::FighterId)
                            .to(Fighter::Table, Fighter::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-parent_id-fighter_parent")
                            .from(FighterParent::Table, FighterParent::ParentId)
                            .to(Fighter::Table, Fighter::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FighterParent::Table).to_owned())
            .await?;

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
pub(crate) enum Fighter {
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
    Mum,
    MetaLastUpdated,
}

#[derive(Iden)]
enum FighterTrait {
    Table,
    FighterId,
    TraitType,
    Value,
}

#[derive(Iden)]
enum FighterParent {
    Table,
    FighterId,
    ParentId,
}
