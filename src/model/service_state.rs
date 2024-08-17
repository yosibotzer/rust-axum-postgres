use sqlx::Postgres;

#[derive(Debug)]
pub struct ServiceState {
    pub pg_pool: sqlx::Pool<Postgres>
}   