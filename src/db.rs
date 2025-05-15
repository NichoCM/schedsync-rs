use diesel::r2d2::{ConnectionManager, Pool};

/**
 * Based on the configured feature for this app, the Connection type will be set to the
 * appropriate diesel connection type.
 */
#[cfg(feature = "postgres")]
pub type Connection = diesel::PgConnection;
#[cfg(feature = "sqlite")]
pub type Connection = diesel::SqliteConnection;
#[cfg(feature = "mysql")]
pub type Connection = diesel::MysqlConnection;

pub type PooledConnection = diesel::r2d2::PooledConnection<ConnectionManager<Connection>>;

/**
 * Based on the configured feature for this app, the Backend type will be set to the
 * appropriate diesel backend type.
 */
#[cfg(feature = "postgres")]
pub type Backend = diesel::pg::Pg;
#[cfg(feature = "sqlite")]
pub type Backend = diesel::sqlite::Sqlite;
#[cfg(feature = "mysql")]
pub type Backend = diesel::mysql::Mysql;

/**
 * Get a connection pool for the given database URL.
 */
pub fn get_connection_pool(url: String) -> Pool<ConnectionManager<Connection>> {
    let manager = ConnectionManager::<Connection>::new(url);
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}