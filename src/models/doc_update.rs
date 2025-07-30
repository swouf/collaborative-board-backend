use diesel::prelude::*;
use crate::infra::db::schema::updates;

#[derive(Queryable, Selectable)]
#[diesel(table_name = updates)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct DocUpdate {
    pub id: i32,
    pub room_id: String,
    pub payload: Vec<u8>,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = updates)]
pub struct NewDocUpdate {
    pub room_id: String,
    pub payload: Vec<u8>,
}