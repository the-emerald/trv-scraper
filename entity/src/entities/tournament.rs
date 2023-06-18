//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use super::sea_orm_active_enums::TournamentStatus;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "tournament")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub service_id: i32,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub currency: Vec<u8>,
    pub fee_percentage: i32,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub buy_in: Vec<u8>,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub top_up: Vec<u8>,
    pub key: String,
    pub legacy: Option<bool>,
    pub level: String,
    pub modified: DateTime,
    pub name: Option<String>,
    #[sea_orm(column_type = "JsonBinary")]
    pub restrictions: Json,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub solo_optionals: Option<Json>,
    pub start_time: DateTime,
    pub status: TournamentStatus,
    pub meta_last_updated: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::tournament_warrior::Entity")]
    TournamentWarrior,
}

impl Related<super::tournament_warrior::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TournamentWarrior.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
