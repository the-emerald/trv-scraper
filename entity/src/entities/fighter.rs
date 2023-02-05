//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.6

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "fighter")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub wisdom_point: i32,
    pub strength_from: i32,
    pub strength_to: i32,
    pub attack_from: i32,
    pub attack_to: i32,
    pub defence_from: i32,
    pub defence_to: i32,
    pub omega_from: i32,
    pub omega_to: i32,
    pub mum: Option<i64>,
    pub last_updated: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::Mum",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    SelfRef,
    #[sea_orm(has_many = "super::fighter_trait::Entity")]
    FighterTrait,
}

impl Related<super::fighter_trait::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FighterTrait.def()
    }
}

impl Related<super::tournament::Entity> for Entity {
    fn to() -> RelationDef {
        super::tournament_warrior::Relation::Tournament.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::tournament_warrior::Relation::Fighter.def().rev())
    }
}

impl Related<super::tournament::Entity> for Entity {
    fn to() -> RelationDef {
        super::tournament_solo_warrior::Relation::Tournament.def()
    }
    fn via() -> Option<RelationDef> {
        Some(
            super::tournament_solo_warrior::Relation::Fighter
                .def()
                .rev(),
        )
    }
}

impl ActiveModelBehavior for ActiveModel {}
