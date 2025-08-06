use crate::infra::db::schema::updates;
use diesel::prelude::*;

pub type DocUpdatePayload = Vec<u8>;

#[derive(Queryable, Selectable)]
#[diesel(table_name = updates)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DocUpdate {
    pub id: i32,
    pub room_id: String,
    pub payload: DocUpdatePayload,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = updates)]
pub struct NewDocUpdate {
    pub room_id: String,
    pub payload: Vec<u8>,
}
