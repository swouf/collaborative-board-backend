use deadpool_diesel::Runtime::Tokio1;
use deadpool_diesel::postgres::{Manager, Pool};
use diesel::prelude::*;
use tracing::{Level, event, span};

use crate::infra::db::schema::updates::dsl::*;
use crate::models::doc_update::DocUpdate;

pub async fn setup_connection_pool(db_url: String) -> Pool {
    let _ = span!(Level::DEBUG, "Connecting to database").enter();
    // set up connection pool
    event!(Level::DEBUG, "Databas URL: {}", &db_url);
    let manager = Manager::new(db_url, Tokio1);
    let pool = Pool::builder(manager).build().unwrap();

    let conn = pool.get().await.unwrap();
    let _res = conn
        .interact(|conn| {
            updates
                .select(DocUpdate::as_select())
                .load(conn)
                .expect("Oups");
        })
        .await
        .unwrap();

    // event!(Level::DEBUG, "Content: {}", res);
    // run the migrations on server startup
    // {
    //     let conn = pool.get().await.unwrap();
    //     conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
    //         .await
    //         .unwrap()
    //         .unwrap();
    // }
    pool
}
