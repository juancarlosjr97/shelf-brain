use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{
    error::SdkError,
    operation::{delete_item::DeleteItemError, put_item::PutItemError, query::QueryError},
    types::AttributeValue,
    Client,
};
use chrono::{DateTime, SecondsFormat, Utc};
use lambda_http::{
    http::{Method, StatusCode},
    run, service_fn, Body, Error, Request, RequestExt, Response,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, sync::Arc};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    table_name: String,
    dynamodb: Client,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum ShelfType {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum ItemStatus {
    Active,
    Archived,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemRecord {
    pk: String,
    sk: String,
    entity_type: String,
    id: String,
    user_id: String,
    title: String,
    shelf_type: ShelfType,
    category: Option<String>,
    due_at: Option<String>,
    notes: Option<String>,
    status: ItemStatus,
    created_at: String,
    updated_at: String,
    last_reviewed_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CategoryRecord {
    pk: String,
    sk: String,
    entity_type: String,
    id: String,
    user_id: String,
    name: String,
    created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NotificationPreferenceRecord {
    pk: String,
    sk: String,
    entity_type: String,
    user_id: String,
    shelf_type: ShelfType,
    frequency: String,
    enabled: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateItemRequest {
    title: String,
    shelf_type: Option<ShelfType>,
    category: Option<String>,
    due_at: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateItemRequest {
    title: Option<String>,
    shelf_type: Option<ShelfType>,
    category: Option<Option<String>>,
    due_at: Option<Option<String>>,
    notes: Option<Option<String>>,
    status: Option<ItemStatus>,
    last_reviewed_at: Option<Option<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MoveItemRequest {
    shelf_type: ShelfType,
}

#[derive(Debug, Deserialize)]
struct CreateCategoryRequest {
    name: String,
}

#[derive(Debug, Deserialize)]
struct UpsertPreferenceRequest {
    frequency: String,
    enabled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiItem {
    id: String,
    title: String,
    shelf_type: ShelfType,
    category: Option<String>,
    due_at: Option<String>,
    notes: Option<String>,
    status: ItemStatus,
    created_at: String,
    updated_at: String,
    last_reviewed_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShelfSummary {
    shelf_type: ShelfType,
    name: &'static str,
    attention_label: &'static str,
    count: usize,
    preview_items: Vec<ApiItem>,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .without_time()
        .init();

    let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
    let state = Arc::new(AppState {
        table_name: env::var("TABLE_NAME").expect("TABLE_NAME must be set"),
        dynamodb: Client::new(&config),
    });

    run(service_fn(move |request| {
        let state = state.clone();
        async move { route_request(request, state).await }
    }))
    .await
}

async fn route_request(request: Request, state: Arc<AppState>) -> Result<Response<Body>, Error> {
    let method = request.method().clone();
    let path = normalize_path(request.raw_http_path());
    let user_id = request
        .headers()
        .get("x-user-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("demo-user")
        .to_string();

    let response = match (method, path.as_slice()) {
        (Method::GET, ["health"]) | (Method::GET, []) => json(StatusCode::OK, &serde_json::json!({"ok": true})),
        (Method::GET, ["shelves"]) => list_shelves(&state, &user_id).await,
        (Method::GET, ["items"]) => list_items(&state, &user_id, request.query_string_parameters()).await,
        (Method::POST, ["items"]) => create_item(&state, &user_id, request.body()).await,
        (Method::GET, ["items", item_id]) => get_item(&state, &user_id, item_id).await,
        (Method::PATCH, ["items", item_id]) => update_item(&state, &user_id, item_id, request.body()).await,
        (Method::POST, ["items", item_id, "move"]) => move_item(&state, &user_id, item_id, request.body()).await,
        (Method::POST, ["items", item_id, "archive"]) => archive_item(&state, &user_id, item_id).await,
        (Method::DELETE, ["items", item_id]) => delete_item(&state, &user_id, item_id).await,
        (Method::GET, ["categories"]) => list_categories(&state, &user_id).await,
        (Method::POST, ["categories"]) => create_category(&state, &user_id, request.body()).await,
        (Method::GET, ["notification-preferences"]) => list_preferences(&state, &user_id).await,
        (Method::PUT, ["notification-preferences", shelf_type]) => {
            upsert_preference(&state, &user_id, shelf_type, request.body()).await
        }
        _ => json_error(StatusCode::NOT_FOUND, "route not found"),
    };

    Ok(response)
}

async fn list_shelves(state: &AppState, user_id: &str) -> Response<Body> {
    match query_user_records(state, user_id, "ITEM#").await {
        Ok(records) => {
            let mut items: Vec<ApiItem> = records
                .into_iter()
                .filter_map(|record| serde_dynamo::from_item::<_, ItemRecord>(record).ok())
                .filter(|item| item.status == ItemStatus::Active)
                .map(ApiItem::from)
                .collect();
            items.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

            let shelves = [
                (ShelfType::Top, "Top Shelf", "Needs attention now or soon"),
                (ShelfType::Middle, "Middle Shelf", "Important, revisit soon"),
                (ShelfType::Bottom, "Bottom Shelf", "Future ideas, low pressure"),
            ]
            .into_iter()
            .map(|(shelf_type, name, attention_label)| {
                let shelf_items: Vec<ApiItem> = items
                    .iter()
                    .filter(|item| item.shelf_type == shelf_type)
                    .cloned()
                    .collect();
                ShelfSummary {
                    shelf_type,
                    name,
                    attention_label,
                    count: shelf_items.len(),
                    preview_items: shelf_items.into_iter().take(3).collect(),
                }
            })
            .collect::<Vec<_>>();

            json(StatusCode::OK, &shelves)
        }
        Err(error) => dynamodb_error(error),
    }
}

async fn list_items(
    state: &AppState,
    user_id: &str,
    query: lambda_http::query_map::QueryMap,
) -> Response<Body> {
    match query_user_records(state, user_id, "ITEM#").await {
        Ok(records) => {
            let shelf = query.first("shelf").and_then(parse_shelf_type);
            let status = query
                .first("status")
                .and_then(parse_status)
                .unwrap_or(ItemStatus::Active);
            let category = query.first("category");

            let mut items: Vec<ApiItem> = records
                .into_iter()
                .filter_map(|record| serde_dynamo::from_item::<_, ItemRecord>(record).ok())
                .filter(|item| shelf.map_or(true, |shelf| item.shelf_type == shelf))
                .filter(|item| item.status == status)
                .filter(|item| category.map_or(true, |category| item.category.as_deref() == Some(category)))
                .map(ApiItem::from)
                .collect();

            items.sort_by(|left, right| {
                left.due_at
                    .cmp(&right.due_at)
                    .then_with(|| right.updated_at.cmp(&left.updated_at))
            });

            json(StatusCode::OK, &items)
        }
        Err(error) => dynamodb_error(error),
    }
}

async fn create_item(state: &AppState, user_id: &str, body: &Body) -> Response<Body> {
    let payload: CreateItemRequest = match parse_body(body) {
        Ok(payload) => payload,
        Err(response) => return response,
    };
    let title = payload.title.trim();
    if title.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "title is required");
    }

    let now = now();
    let item = ItemRecord {
        pk: user_pk(user_id),
        sk: format!("ITEM#{}", Uuid::new_v4()),
        entity_type: "item".to_string(),
        id: Uuid::new_v4().to_string(),
        user_id: user_id.to_string(),
        title: title.to_string(),
        shelf_type: payload.shelf_type.unwrap_or(ShelfType::Middle),
        category: clean_optional(payload.category),
        due_at: clean_optional(payload.due_at),
        notes: clean_optional(payload.notes),
        status: ItemStatus::Active,
        created_at: now.clone(),
        updated_at: now,
        last_reviewed_at: None,
    };

    put_record(state, &item).await.map_or_else(dynamodb_error, |_| json(StatusCode::CREATED, &ApiItem::from(item)))
}

async fn get_item(state: &AppState, user_id: &str, item_id: &str) -> Response<Body> {
    match find_item(state, user_id, item_id).await {
        Ok(Some(item)) => json(StatusCode::OK, &ApiItem::from(item)),
        Ok(None) => json_error(StatusCode::NOT_FOUND, "item not found"),
        Err(error) => dynamodb_error(error),
    }
}

async fn update_item(state: &AppState, user_id: &str, item_id: &str, body: &Body) -> Response<Body> {
    let payload: UpdateItemRequest = match parse_body(body) {
        Ok(payload) => payload,
        Err(response) => return response,
    };

    let mut item = match find_item(state, user_id, item_id).await {
        Ok(Some(item)) => item,
        Ok(None) => return json_error(StatusCode::NOT_FOUND, "item not found"),
        Err(error) => return dynamodb_error(error),
    };

    if let Some(title) = payload.title {
        let title = title.trim();
        if title.is_empty() {
            return json_error(StatusCode::BAD_REQUEST, "title cannot be empty");
        }
        item.title = title.to_string();
    }
    if let Some(shelf_type) = payload.shelf_type {
        item.shelf_type = shelf_type;
    }
    if let Some(category) = payload.category {
        item.category = clean_optional(category);
    }
    if let Some(due_at) = payload.due_at {
        item.due_at = clean_optional(due_at);
    }
    if let Some(notes) = payload.notes {
        item.notes = clean_optional(notes);
    }
    if let Some(status) = payload.status {
        item.status = status;
    }
    if let Some(last_reviewed_at) = payload.last_reviewed_at {
        item.last_reviewed_at = clean_optional(last_reviewed_at);
    }
    item.updated_at = now();

    put_record(state, &item)
        .await
        .map_or_else(dynamodb_error, |_| json(StatusCode::OK, &ApiItem::from(item)))
}

async fn move_item(state: &AppState, user_id: &str, item_id: &str, body: &Body) -> Response<Body> {
    let payload: MoveItemRequest = match parse_body(body) {
        Ok(payload) => payload,
        Err(response) => return response,
    };
    update_item(
        state,
        user_id,
        item_id,
        &Body::Text(
            serde_json::to_string(&serde_json::json!({ "shelfType": payload.shelf_type })).unwrap(),
        ),
    )
    .await
}

async fn archive_item(state: &AppState, user_id: &str, item_id: &str) -> Response<Body> {
    update_item(
        state,
        user_id,
        item_id,
        &Body::Text("{\"status\":\"archived\"}".to_string()),
    )
    .await
}

async fn delete_item(state: &AppState, user_id: &str, item_id: &str) -> Response<Body> {
    let item = match find_item(state, user_id, item_id).await {
        Ok(Some(item)) => item,
        Ok(None) => return json_error(StatusCode::NOT_FOUND, "item not found"),
        Err(error) => return dynamodb_error(error),
    };

    let result = state
        .dynamodb
        .delete_item()
        .table_name(&state.table_name)
        .key("pk", AttributeValue::S(item.pk))
        .key("sk", AttributeValue::S(item.sk))
        .send()
        .await;

    match result {
        Ok(_) => json(StatusCode::OK, &serde_json::json!({"deleted": true})),
        Err(error) => dynamodb_delete_error(error),
    }
}

async fn list_categories(state: &AppState, user_id: &str) -> Response<Body> {
    match query_user_records(state, user_id, "CATEGORY#").await {
        Ok(records) => {
            let mut categories: Vec<CategoryRecord> = records
                .into_iter()
                .filter_map(|record| serde_dynamo::from_item(record).ok())
                .collect();
            categories.sort_by(|left, right| left.name.cmp(&right.name));
            json(StatusCode::OK, &categories)
        }
        Err(error) => dynamodb_error(error),
    }
}

async fn create_category(state: &AppState, user_id: &str, body: &Body) -> Response<Body> {
    let payload: CreateCategoryRequest = match parse_body(body) {
        Ok(payload) => payload,
        Err(response) => return response,
    };
    let name = payload.name.trim();
    if name.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "name is required");
    }

    let category = CategoryRecord {
        pk: user_pk(user_id),
        sk: format!("CATEGORY#{}", Uuid::new_v4()),
        entity_type: "category".to_string(),
        id: Uuid::new_v4().to_string(),
        user_id: user_id.to_string(),
        name: name.to_string(),
        created_at: now(),
    };

    put_record(state, &category).await.map_or_else(dynamodb_error, |_| json(StatusCode::CREATED, &category))
}

async fn list_preferences(state: &AppState, user_id: &str) -> Response<Body> {
    match query_user_records(state, user_id, "PREF#").await {
        Ok(records) => {
            let mut preferences: Vec<NotificationPreferenceRecord> = records
                .into_iter()
                .filter_map(|record| serde_dynamo::from_item(record).ok())
                .collect();
            preferences.sort_by(|left, right| shelf_sort(left.shelf_type).cmp(&shelf_sort(right.shelf_type)));
            json(StatusCode::OK, &preferences)
        }
        Err(error) => dynamodb_error(error),
    }
}

async fn upsert_preference(
    state: &AppState,
    user_id: &str,
    shelf_type: &str,
    body: &Body,
) -> Response<Body> {
    let shelf_type = match parse_shelf_type(shelf_type) {
        Some(shelf_type) => shelf_type,
        None => return json_error(StatusCode::BAD_REQUEST, "invalid shelf type"),
    };
    let payload: UpsertPreferenceRequest = match parse_body(body) {
        Ok(payload) => payload,
        Err(response) => return response,
    };
    let now = now();
    let preference = NotificationPreferenceRecord {
        pk: user_pk(user_id),
        sk: format!("PREF#{}", shelf_key(shelf_type)),
        entity_type: "notificationPreference".to_string(),
        user_id: user_id.to_string(),
        shelf_type,
        frequency: payload.frequency,
        enabled: payload.enabled,
        created_at: now.clone(),
        updated_at: now,
    };

    put_record(state, &preference)
        .await
        .map_or_else(dynamodb_error, |_| json(StatusCode::OK, &preference))
}

async fn query_user_records(
    state: &AppState,
    user_id: &str,
    sk_prefix: &str,
) -> Result<Vec<HashMap<String, AttributeValue>>, SdkError<QueryError>> {
    let output = state
        .dynamodb
        .query()
        .table_name(&state.table_name)
        .key_condition_expression("pk = :pk AND begins_with(sk, :sk)")
        .expression_attribute_values(":pk", AttributeValue::S(user_pk(user_id)))
        .expression_attribute_values(":sk", AttributeValue::S(sk_prefix.to_string()))
        .send()
        .await?;

    Ok(output.items.unwrap_or_default())
}

async fn find_item(
    state: &AppState,
    user_id: &str,
    item_id: &str,
) -> Result<Option<ItemRecord>, SdkError<QueryError>> {
    let items = query_user_records(state, user_id, "ITEM#").await?;
    Ok(items
        .into_iter()
        .filter_map(|record| serde_dynamo::from_item::<_, ItemRecord>(record).ok())
        .find(|item| item.id == item_id))
}

async fn put_record<T>(state: &AppState, record: &T) -> Result<(), SdkError<PutItemError>>
where
    T: Serialize,
{
    let item = serde_dynamo::to_item(record).expect("record serializes to DynamoDB item");
    state
        .dynamodb
        .put_item()
        .table_name(&state.table_name)
        .set_item(Some(item))
        .send()
        .await?;
    Ok(())
}

impl From<ItemRecord> for ApiItem {
    fn from(item: ItemRecord) -> Self {
        Self {
            id: item.id,
            title: item.title,
            shelf_type: item.shelf_type,
            category: item.category,
            due_at: item.due_at,
            notes: item.notes,
            status: item.status,
            created_at: item.created_at,
            updated_at: item.updated_at,
            last_reviewed_at: item.last_reviewed_at,
        }
    }
}

impl Clone for ApiItem {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            title: self.title.clone(),
            shelf_type: self.shelf_type,
            category: self.category.clone(),
            due_at: self.due_at.clone(),
            notes: self.notes.clone(),
            status: self.status,
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
            last_reviewed_at: self.last_reviewed_at.clone(),
        }
    }
}

fn parse_body<T>(body: &Body) -> Result<T, Response<Body>>
where
    T: for<'de> Deserialize<'de>,
{
    let bytes = match body {
        Body::Text(text) => text.as_bytes(),
        Body::Binary(bytes) => bytes.as_slice(),
        Body::Empty => &[],
    };
    serde_json::from_slice(bytes).map_err(|_| json_error(StatusCode::BAD_REQUEST, "invalid JSON body"))
}

fn normalize_path(path: &str) -> Vec<&str> {
    path.trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn json<T>(status: StatusCode, value: &T) -> Response<Body>
where
    T: Serialize,
{
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .header("access-control-allow-origin", "*")
        .body(Body::Text(serde_json::to_string(value).unwrap()))
        .unwrap()
}

fn json_error(status: StatusCode, message: &str) -> Response<Body> {
    json(
        status,
        &ErrorBody {
            error: message.to_string(),
        },
    )
}

fn dynamodb_error<T>(error: SdkError<T>) -> Response<Body>
where
    T: std::fmt::Debug,
{
    tracing::error!(?error, "dynamodb request failed");
    json_error(StatusCode::INTERNAL_SERVER_ERROR, "database request failed")
}

fn dynamodb_delete_error(error: SdkError<DeleteItemError>) -> Response<Body> {
    tracing::error!(?error, "dynamodb delete failed");
    json_error(StatusCode::INTERNAL_SERVER_ERROR, "database request failed")
}

fn now() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn user_pk(user_id: &str) -> String {
    format!("USER#{user_id}")
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

fn parse_shelf_type(value: &str) -> Option<ShelfType> {
    match value {
        "top" => Some(ShelfType::Top),
        "middle" => Some(ShelfType::Middle),
        "bottom" => Some(ShelfType::Bottom),
        _ => None,
    }
}

fn parse_status(value: &str) -> Option<ItemStatus> {
    match value {
        "active" => Some(ItemStatus::Active),
        "archived" => Some(ItemStatus::Archived),
        _ => None,
    }
}

fn shelf_key(shelf_type: ShelfType) -> &'static str {
    match shelf_type {
        ShelfType::Top => "top",
        ShelfType::Middle => "middle",
        ShelfType::Bottom => "bottom",
    }
}

fn shelf_sort(shelf_type: ShelfType) -> usize {
    match shelf_type {
        ShelfType::Top => 0,
        ShelfType::Middle => 1,
        ShelfType::Bottom => 2,
    }
}
