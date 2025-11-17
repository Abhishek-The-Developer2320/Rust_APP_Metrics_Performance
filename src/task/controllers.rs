use actix_web::{web, HttpResponse, Responder};
use sqlx::SqlitePool;
use tera::{Tera, Context};
use sqlx::Row; 
use std::time::Instant;

use crate::models::Task;

/// Show all tasks (index page)
pub async fn index_html(
    db: web::Data<SqlitePool>,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let rows = sqlx::query("SELECT id, title, completed FROM task")
        .fetch_all(db.get_ref())
        .await
        .expect("Failed to fetch tasks");

    let tasks: Vec<Task> = rows.into_iter().map(|row| Task {
        id: Some(row.get("id")),
        title: row.get("title"),
        completed: Some(row.get("completed")),
    }).collect();

    let mut ctx = Context::new();
    ctx.insert("tasks", &tasks);

    let rendered = tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)

}



/// Show form to create a new task
pub async fn create_html_form(tmpl: web::Data<Tera>) -> impl Responder {
    let rendered = tmpl.render("create.html", &Context::new()).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// Handle form submission to create a task
pub async fn create_html(
    db: web::Data<SqlitePool>,
    form: web::Form<Task>
) -> impl Responder {
    sqlx::query(
        "INSERT INTO task (title, completed) VALUES (?, ?)"
    )
    .bind(&form.title)
    .bind(form.completed.unwrap_or(false))
    .execute(db.get_ref())
    .await
    .expect("Failed to insert task");

    HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish()
}

/// Show edit form for a task
pub async fn edit_html_form(
    db: web::Data<SqlitePool>,
    tmpl: web::Data<Tera>,
    task_id: web::Path<i64>
) -> impl Responder {
    let row = sqlx::query(
        "SELECT id, title, completed FROM task WHERE id = ?"
    )
    .bind(task_id.into_inner())
    .fetch_one(db.get_ref())
    .await
    .expect("Task not found");

    let task = Task {
        id: Some(row.get("id")),
        title: row.get("title"),
        completed: Some(row.get("completed")),
    };

    let mut ctx = Context::new();
    ctx.insert("task", &task);

    let rendered = tmpl.render("edit.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

/// Handle edit form submission
pub async fn edit_html(
    db: web::Data<SqlitePool>,
    task_id: web::Path<i64>,
    form: web::Form<Task>
) -> impl Responder {
    sqlx::query(
        "UPDATE task SET title = ?, completed = ? WHERE id = ?"
    )
    .bind(&form.title)
    .bind(form.completed.unwrap_or(false))
    .bind(task_id.into_inner())
    .execute(db.get_ref())
    .await
    .expect("Failed to update task");

    HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish()
}

/// Delete a task
pub async fn delete_html(
    db: web::Data<SqlitePool>,
    task_id: web::Path<i64>
) -> impl Responder {
    sqlx::query(
        "DELETE FROM task WHERE id = ?"
    )
    .bind(task_id.into_inner())
    .execute(db.get_ref())
    .await
    .expect("Failed to delete task");

    HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish()
}

/// Bulk create: accept JSON array of tasks (title/completed optional) and insert them
pub async fn bulk_create(
    db: web::Data<SqlitePool>,
    items: web::Json<Vec<Task>>,
) -> impl Responder {
    let mut created: i64 = 0;
    for it in items.iter() {
        let res = sqlx::query("INSERT INTO task (title, completed) VALUES (?, ?)")
            .bind(&it.title)
            .bind(it.completed.unwrap_or(false))
            .execute(db.get_ref())
            .await;
        if res.is_ok() { created += 1; }
    }
    HttpResponse::Ok().json(serde_json::json!({"created_count": created}))
}

/// Bulk read: return all tasks as JSON
pub async fn bulk_read(
    db: web::Data<SqlitePool>
) -> impl Responder {
    let rows = match sqlx::query("SELECT id, title, completed FROM task").fetch_all(db.get_ref()).await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(format!("DB error: {}", e)),
    };

    let tasks: Vec<Task> = rows.into_iter().map(|row| Task {
        id: Some(row.get("id")),
        title: row.get("title"),
        completed: Some(row.get("completed")),
    }).collect();

    HttpResponse::Ok().json(tasks)
}

/// Bulk update: accept JSON array of {id, title, completed} and update matching rows
pub async fn bulk_update(
    db: web::Data<SqlitePool>,
    items: web::Json<Vec<Task>>,
) -> impl Responder {
    let mut updated: i64 = 0;
    for it in items.iter() {
        if let Some(id) = it.id {
            let res = sqlx::query("UPDATE task SET title = ?, completed = ? WHERE id = ?")
                .bind(&it.title)
                .bind(it.completed.unwrap_or(false))
                .bind(id)
                .execute(db.get_ref())
                .await;
            if let Ok(r) = res {
                if r.rows_affected() > 0 { updated += r.rows_affected() as i64; }
            }
        }
    }
    HttpResponse::Ok().json(serde_json::json!({"updated": updated}))
}

/// Bulk delete: accept JSON array of ids and delete them
pub async fn bulk_delete(
    db: web::Data<SqlitePool>,
    ids: web::Json<Vec<i64>>,
) -> impl Responder {
    let mut deleted: i64 = 0;
    for id in ids.iter() {
        let res = sqlx::query("DELETE FROM task WHERE id = ?")
            .bind(id)
            .execute(db.get_ref())
            .await;
        if let Ok(r) = res {
            if r.rows_affected() > 0 { deleted += r.rows_affected() as i64; }
        }
    }
    HttpResponse::Ok().json(serde_json::json!({"deleted": deleted}))
}
