use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc};

#[derive(Serialize, Deserialize, Clone)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub start_date: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
    pub location: Option<String>,
    pub image_url: Option<String>,
    pub category: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EventCreate {
    pub title: String,
    pub description: Option<String>,
    pub start_date: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
    pub location: Option<String>,
    pub image_url: Option<String>,
    pub category: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EventUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
    pub location: Option<String>,
    pub image_url: Option<String>,
    pub category: Option<String>,
}
