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
                            .integer()
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
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Fighter::Table).to_owned())
            .await
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
}
