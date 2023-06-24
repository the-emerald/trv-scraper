//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "meta_failed_tournament_request")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub page_size: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub page_index: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
