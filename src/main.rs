use actix_web::{App, HttpServer, web};
use crate::db::init_db;
use crate::task::views::task_routes;
use tera::Tera;

mod config;
mod db;
mod models;
mod task;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_pool = init_db().await;

    // Load templates
    let tera = Tera::new("src/templates/**/*").unwrap();

    println!("Server running at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(tera.clone()))
            .configure(task_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
