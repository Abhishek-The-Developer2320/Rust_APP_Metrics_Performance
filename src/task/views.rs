use actix_web::web;
use crate::task::controllers;

/// Configure task routes
pub fn task_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Index page - list all tasks
        .route("/", web::get().to(controllers::index_html))
        
        // Create task
        .route("/create", web::get().to(controllers::create_html_form))
        .route("/create", web::post().to(controllers::create_html))
        
        // Edit task
        .route("/edit/{id}", web::get().to(controllers::edit_html_form))
        .route("/edit/{id}", web::post().to(controllers::edit_html))
        
        // Delete task
    .route("/delete/{id}", web::post().to(controllers::delete_html))
    .route("/bulk_create", web::post().to(controllers::bulk_create))
    .route("/bulk_read", web::get().to(controllers::bulk_read))
    .route("/bulk_update", web::put().to(controllers::bulk_update))
    .route("/bulk_delete", web::delete().to(controllers::bulk_delete));
}
