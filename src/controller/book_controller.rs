
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::routing::{get, post};
use crate::model::api::{CreateBookRequest, QueryBookRequest};
use crate::model::row::BookRow;

use crate::model::service_state::ServiceState;
use crate::service::error::RepoError;
use crate::service::book_service;

pub fn get_book_routes(service_state: ServiceState) -> Router {
    Router::new()
        .route("/book", post(create_book))
        .route("/book/find", post(find_books))
        .route("/book", get(list_books))
        .route("/book/:id", get(fetch_book))
        .with_state(Arc::new(service_state))
}

async fn create_book(State(service_state): State<Arc<ServiceState>>, Json(payload): Json<CreateBookRequest>) 
-> Result<(StatusCode, Json<BookRow>), RepoError> {

    let result = book_service::create_book(service_state, payload).await?;
    
    Ok((StatusCode::CREATED, Json(result)))
}

async fn find_books(State(service_state): State<Arc<ServiceState>>, Json(payload): Json<QueryBookRequest>) 
-> Result<Json<Vec<BookRow>>, RepoError>  {

    let result = book_service::find_books(service_state, payload).await?;
    
    Ok(Json(result))
}

async fn fetch_book(State(service_state): State<Arc<ServiceState>>, Path(id): Path<i32>) 
-> Result<Response, RepoError> {

    let result = book_service::fetch_book(service_state, id).await?;

    match result {
        Some(book) => Ok(Json(book).into_response()),
        None => Ok(StatusCode::NO_CONTENT.into_response())
    }
}


async fn list_books(State(service_state): State<Arc<ServiceState>>) 
-> Result<Json<Vec<BookRow>>, RepoError>  {

    let result = book_service::list_books(service_state).await?;
    
    Ok(Json(result))
}

impl IntoResponse for RepoError {

    fn into_response(self) -> Response {
        match self {
            _ => StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
