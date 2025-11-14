use actix_web::{web, HttpResponse, Responder};
use sqlx::SqlitePool;
use tera::{Tera, Context};
use sqlx::Row; // <-- Add this

use crate::models::Task;
use serde_json::json;

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

/// Bulk create tasks from a JSON array. Returns created IDs.
pub async fn bulk_create(
    db: web::Data<SqlitePool>,
    items: web::Json<Vec<Task>>,
) -> impl Responder {
    let mut tx = db.begin().await.expect("Failed to begin transaction");
    let mut ids = Vec::new();
    for t in items.into_inner().into_iter() {
        let res = sqlx::query("INSERT INTO task (title, completed) VALUES (?, ?)")
            .bind(&t.title)
            .bind(t.completed.unwrap_or(false))
            .execute(&mut tx)
            .await
            .expect("Failed to insert task");
        let id = res.last_insert_rowid();
        ids.push(id);
    }
    tx.commit().await.expect("Failed to commit");
    HttpResponse::Ok().json(json!({ "created": ids }))
}

/// Bulk read tasks. Optional query param `limit` to restrict number returned.
pub async fn bulk_read(
    db: web::Data<SqlitePool>,
    params: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let mut q = String::from("SELECT id, title, completed FROM task");
    if let Some(limit) = params.get("limit") {
        if let Ok(n) = limit.parse::<i64>() {
            q.push_str(" LIMIT ");
            q.push_str(&n.to_string());
        }
    }

    let rows = sqlx::query(&q)
        .fetch_all(db.get_ref())
        .await
        .expect("Failed to fetch tasks");

    let tasks: Vec<Task> = rows.into_iter().map(|row| Task {
        id: Some(row.get("id")),
        title: row.get("title"),
        completed: Some(row.get("completed")),
    }).collect();

    HttpResponse::Ok().json(tasks)
}

/// Bulk update tasks. Accepts JSON array of Task objects with id set.
pub async fn bulk_update(
    db: web::Data<SqlitePool>,
    items: web::Json<Vec<Task>>,
) -> impl Responder {
    let mut tx = db.begin().await.expect("Failed to begin transaction");
    let mut updated = 0i64;
    for t in items.into_inner().into_iter() {
        if let Some(id) = t.id {
            let res = sqlx::query("UPDATE task SET title = ?, completed = ? WHERE id = ?")
                .bind(&t.title)
                .bind(t.completed.unwrap_or(false))
                .bind(id)
                .execute(&mut tx)
                .await
                .expect("Failed to update task");
            updated += res.rows_affected() as i64;
        }
    }
    tx.commit().await.expect("Failed to commit");
    HttpResponse::Ok().json(json!({ "updated": updated }))
}

/// Bulk delete tasks. Accepts JSON array of IDs.
pub async fn bulk_delete(
    db: web::Data<SqlitePool>,
    ids: web::Json<Vec<i64>>,
) -> impl Responder {
    let mut tx = db.begin().await.expect("Failed to begin transaction");
    let mut deleted = 0i64;
    for id in ids.into_inner().into_iter() {
        let res = sqlx::query("DELETE FROM task WHERE id = ?")
            .bind(id)
            .execute(&mut tx)
            .await
            .expect("Failed to delete task");
        deleted += res.rows_affected() as i64;
    }
    tx.commit().await.expect("Failed to commit");
    HttpResponse::Ok().json(json!({ "deleted": deleted }))
}
