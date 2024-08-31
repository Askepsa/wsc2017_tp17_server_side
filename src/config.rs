use crate::routes::{
    auth::{login, logout},
    place::{
        get::get_places,
        slug::{delete::delete_place, get::find_place},
    },
    route::search::shortest_paths,
    DatabasePool,
};
use actix_web::{
    web::{self, Data, ServiceConfig},
    HttpResponse,
};
use sqlx::postgres::PgPoolOptions;
use std::{collections::HashMap, fs, io};

#[derive(Clone)]
pub struct ServerConfig {
    pub db_pool: Data<DatabasePool>,
    pub env: HashMap<String, String>,
}

impl ServerConfig {
    pub async fn new() -> Self {
        let env = read_env().expect("Reading of env failed");
        let db_url = env.get("DATABASE_URL").expect("env not found");
        let db_pool: Data<DatabasePool> = web::Data::new(DatabasePool {
            pool: PgPoolOptions::new()
                .connect(db_url)
                .await
                .expect("Connection to database not failed"),
        });

        Self { db_pool, env }
    }

    pub fn config(&self, cfg: &mut ServiceConfig) {
        cfg.app_data(self.db_pool.clone());
        cfg.route("/", web::get().to(HttpResponse::Ok))
            .service(
                web::scope("/v1")
                    .service(
                        web::scope("/auth")
                            .route("/login", web::post().to(login))
                            .route("/logout", web::get().to(logout)),
                    )
                    .service(
                        web::resource("/place/{id}")
                            .get(find_place)
                            .delete(delete_place),
                    )
                    .route("/place", web::get().to(get_places))
                    .service(
                        web::scope("/route").service(
                            web::resource("/search/{from_place_id}/{to_place_id}/{departure_time}")
                                .get(shortest_paths),
                        ),
                    ),
            );
    }
}

fn read_env() -> Result<HashMap<String, String>, io::Error> {
    let buf = fs::read_to_string(".env")?;
    let vec: Vec<&str> = buf.as_str().split('\n').collect();
    let hash_map: HashMap<String, String> = vec
        .into_iter()
        .map(|str| match str.split_once('=') {
            Some((key, val)) => (key.to_string(), val.to_string()),
            None => ("".into(), "".into()),
        })
        .filter(|val| *val != ("".into(), "".into()))
        .collect();

    Ok(hash_map)
}
