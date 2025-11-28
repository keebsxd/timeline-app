use axum::{
    routing::{get, post, put, delete},
    Router, http::StatusCode, response::IntoResponse, Json, extract::Path,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber;
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Serialize, Deserialize, Clone)]
struct Event {
    id: uuid::Uuid,
    title: String,
    description: Option<String>,
    start_date: chrono::NaiveDateTime,
    end_date: Option<chrono::NaiveDateTime>,
    location: Option<String>,
    image_url: Option<String>,
    category: Option<String>,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone)]
struct EventCreate {
    title: String,
    description: Option<String>,
    start_date: chrono::NaiveDateTime,
    end_date: Option<chrono::NaiveDateTime>,
    location: Option<String>,
    image_url: Option<String>,
    category: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct EventUpdate {
    title: Option<String>,
    description: Option<String>,
    start_date: Option<chrono::NaiveDateTime>,
    end_date: Option<chrono::NaiveDateTime>,
    location: Option<String>,
    image_url: Option<String>,
    category: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct PaginatedResponse<T> {
    data: Vec<T>,
    total: i64,
    page: i32,
    limit: i32,
    pages: i32,
}

async fn get_events(
    pool: PgPool,
    page: Option<i32>,
    limit: Option<i32>,
    search: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Json<PaginatedResponse<Event>>, StatusCode> {
    let page = page.unwrap_or(1).max(1);
    let limit = limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    let mut query = "SELECT * FROM events".to_string();
    let mut params = vec![];

    if let Some(search) = &search {
        query += " WHERE title ILIKE $1 OR description ILIKE $1";
        params.push(format!("%{}%", search));
    }

    if let (Some(start), Some(end)) = (&start_date, &end_date) {
        query += " AND start_date BETWEEN $2 AND $3";
        params.push(start.clone());
        params.push(end.clone());
    }

    query += " ORDER BY start_date DESC LIMIT $4 OFFSET $5";
    params.push(limit.to_string());
    params.push(offset.to_string());

    let rows = sqlx::query(&query)
        .bind(params[0].clone())
        .bind(params[1].clone())
        .bind(params[2].clone())
        .bind(params[3].clone())
        .bind(params[4].clone())
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = sqlx::query("SELECT COUNT(*) FROM events")
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .get::<i64, _>(0);

    let events: Vec<Event> = rows
        .into_iter()
        .map(|row| Event {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            start_date: row.get("start_date"),
            end_date: row.get("end_date"),
            location: row.get("location"),
            image_url: row.get("image_url"),
            category: row.get("category"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .collect();

    Ok(Json(PaginatedResponse {
        data: events,
        total,
        page,
        limit,
        pages: (total as f64 / limit as f64).ceil() as i32,
    }))
}

async fn get_event(
    pool: PgPool,
    id: Path<uuid::Uuid>,
) -> Result<Json<Event>, StatusCode> {
    let event = sqlx::query_as!(
        Event,
        "SELECT * FROM events WHERE id = $1",
        id.0
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(event))
}

async fn create_event(
    pool: PgPool,
    Json(payload): Json<EventCreate>,
) -> Result<Json<Event>, StatusCode> {
    let id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now().naive_utc();

    let event = sqlx::query_as!(
        Event,
        r#"
        INSERT INTO events (id, title, description, start_date, end_date, location, image_url, category, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#,
        id,
        payload.title,
        payload.description,
        payload.start_date,
        payload.end_date,
        payload.location,
        payload.image_url,
        payload.category,
        now,
        now
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(event))
}

async fn update_event(
    pool: PgPool,
    id: Path<uuid::Uuid>,
    Json(payload): Json<EventUpdate>,
) -> Result<Json<Event>, StatusCode> {
    let now = chrono::Utc::now().naive_utc();

    let mut query = "UPDATE events SET updated_at = $1".to_string();
    let mut params = vec![now];

    if let Some(title) = &payload.title {
        query += ", title = $2";
        params.push(title.clone());
    }
    if let Some(description) = &payload.description {
        query += ", description = $3";
        params.push(description.clone());
    }
    if let Some(start_date) = &payload.start_date {
        query += ", start_date = $4";
        params.push(start_date.clone());
    }
    if let Some(end_date) = &payload.end_date {
        query += ", end_date = $5";
        params.push(end_date.clone());
    }
    if let Some(location) = &payload.location {
        query += ", location = $6";
        params.push(location.clone());
    }
    if let Some(image_url) = &payload.image_url {
        query += ", image_url = $7";
        params.push(image_url.clone());
    }
    if let Some(category) = &payload.category {
        query += ", category = $8";
        params.push(category.clone());
    }

    query += " WHERE id = $9 RETURNING *";

    params.push(id.0);

    let event = sqlx::query_as(&query)
        .bind(&params[0])
        .bind(&params[1])
        .bind(&params[2])
        .bind(&params[3])
        .bind(&params[4])
        .bind(&params[5])
        .bind(&params[6])
        .bind(&params[7])
        .bind(&params[8])
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(event))
}

async fn delete_event(
    pool: PgPool,
    id: Path<uuid::Uuid>,
) -> Result<Json<()>, StatusCode> {
    sqlx::query("DELETE FROM events WHERE id = $1")
        .bind(id.0)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(()))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let pool = PgPool::connect("postgres://user:password@localhost/timeline").await.unwrap();
    
    // Create table if not exists
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title VARCHAR(255) NOT NULL,
            description TEXT,
            start_date TIMESTAMP NOT NULL,
            end_date TIMESTAMP,
            location VARCHAR(255),
            image_url VARCHAR(512),
            category VARCHAR(100),
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMP NOT NULL DEFAULT NOW()
        )
        "#,
    ).execute(&pool).await.unwrap();

    let app = Router::new()
        .route("/api/events", get(get_events).post(create_event))
        .route("/api/events/:id", get(get_event).put(update_event).delete(delete_event))
        .with_state(pool)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
