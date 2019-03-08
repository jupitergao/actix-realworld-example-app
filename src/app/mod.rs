use crate::db::{new_pool, DbExecutor};
use actix::prelude::{Addr, SyncArbiter};
use actix_web::{
    http::{header, Method},
    middleware::{cors::Cors, Logger},
    App, HttpRequest,
};
use std::env;

pub mod articles;
pub mod profiles;
pub mod tags;
pub mod users;

const NUM_DB_THREADS: usize = 4;

pub struct AppState {
    pub db: Addr<DbExecutor>,
}

fn index(_req: &HttpRequest<AppState>) -> &'static str {
    "Hello world!"
}

pub fn create() -> App<AppState> {
    let frontend_origin = env::var("FRONTEND_ORIGIN").ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let database_pool = new_pool(database_url).expect("Failed to create pool.");

    let database_address =
        SyncArbiter::start(NUM_DB_THREADS, move || DbExecutor(database_pool.clone()));

    let state = AppState {
        db: database_address.clone(),
    };

    App::with_state(state)
        .middleware(Logger::default())
        .configure(|app| {
            // check whether to enable CORS
            match frontend_origin {
                Some(ref origin) => Cors::for_app(app)
                    .allowed_origin(origin)
                    .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
                    .max_age(3600)
                    .register(),
                None => app,
            }
        })
        .resource("/", |r| r.f(index))
        .scope("/api", |scope| {
            // Users
            let scope = scope
                .resource("users", |r| {
                    r.method(Method::POST).with_async(users::register)
                })
                .resource("users/login", |r| {
                    r.method(Method::POST).with_async(users::login)
                })
                .resource("user", |r| {
                    r.method(Method::GET).with_async(users::get_current);
                    r.method(Method::PUT).with_async(users::update)
                });

            scope
        })
}
