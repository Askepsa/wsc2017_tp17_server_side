use sqlx::postgres::PgPool;

pub struct DatabasePool {
    pub pool: PgPool,
}
