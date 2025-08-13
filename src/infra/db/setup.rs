use std::error::Error;

use deadpool_diesel::Runtime::Tokio1;
use deadpool_diesel::postgres::{Manager, Pool};
use diesel::pg::Pg;
use diesel::prelude::*;
use tracing::{Level, event, span};

use crate::infra::db::schema::updates::dsl::*;
use crate::models::doc_update::DocUpdate;

use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

fn run_migrations(
    connection: &mut impl MigrationHarness<Pg>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    match connection.run_pending_migrations(MIGRATIONS) {
        Ok(applied_migrations) => {
            event!(
                Level::INFO,
                "Applied {} migrations.",
                applied_migrations.len()
            );
            Ok(())
        }
        Err(err) => {
            event!(Level::ERROR, "Error applying migrations.\n{}", err);
            Err(err)
        }
    }
}

pub async fn setup_connection_pool(db_url: String) -> Pool {
    let _ = span!(Level::DEBUG, "Connecting to database").enter();
    // set up connection pool
    event!(Level::DEBUG, "Database URL: {}", &db_url);
    let manager = Manager::new(db_url, Tokio1);
    let pool = Pool::builder(manager).build().unwrap();

    let conn = pool.get().await.unwrap();
    conn.interact(|conn| {
        run_migrations(conn).expect("Migrations failed.");
    })
    .await
    .unwrap();

    conn.interact(|conn| {
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
