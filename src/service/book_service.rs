use std::sync::Arc;

use sqlx::Row;
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use tracing::{error, info};
use crate::model::api::{CreateBookRequest, QueryBookRequest};
use crate::model::row::BookRow;
use crate::model::service_state::ServiceState;
use crate::service::error::RepoError;

#[derive(Iden)]
enum Book {
    Table,
    Id, 
    Name,
    Author,
}

pub async fn fetch_book(service_state: Arc<ServiceState>, id: i32) -> Result<Option<BookRow>, RepoError> {

    let (sql, values) = Query::select()
        .columns([sea_query::Asterisk])
        .from(Book::Table)
        .and_where(Expr::col(Book::Id).eq(id))
        .build_sqlx(PostgresQueryBuilder);
    
    let book = sqlx::query_as_with::<_, BookRow, _>(&sql, values)
        .fetch_optional(&service_state.pg_pool)
        .await
        .map_err(|e| map_sql_fetch_error(e, id))?;

    Ok(book)
}

pub async fn create_book(service_state: Arc<ServiceState>, create_book: CreateBookRequest) -> Result<BookRow, RepoError> {

    let (sql, values) = Query::insert()
        .into_table(Book::Table)
        .columns([Book::Name, Book::Author])
        .values([create_book.name.as_str().into(), create_book.author.as_str().into()])
        .map_err(|e| map_create_values_error(e, &create_book))?
        .returning_col(Book::Id)
        .build_sqlx(PostgresQueryBuilder);

    let row = sqlx::query_with(&sql, values)
        .fetch_one(&service_state.pg_pool)
        .await
        .map_err(|e| map_create_error(e, &create_book))?;

    let book_id: i32 = row
        .try_get(0)
        .map_err(|e| map_create_error(e, &create_book))?;
    
    info!("Created book: {book_id}");

    fetch_book(service_state, book_id)
        .await?
        .ok_or(RepoError::CreateBookError)
}

pub async fn list_books(service_state: Arc<ServiceState>) -> Result<Vec<BookRow>, RepoError> {

    let (sql, values) = Query::select()
        .columns([sea_query::Asterisk])
        .from(Book::Table)
        .build_sqlx(PostgresQueryBuilder);

   let books = sqlx::query_as_with::<_, BookRow, _>(&sql, values)
        .fetch_all(&service_state.pg_pool)
        .await
        .map_err(|e| map_sql_list_error(e))?;

    Ok(books)
}

pub async fn find_books(service_state: Arc<ServiceState>, query_book: QueryBookRequest) -> Result<Vec<BookRow>, RepoError> {

    let (sql, values) = Query::select()
        .columns([sea_query::Asterisk])
        .from(Book::Table)
        .conditions(
            query_book.author.is_some(),
            |q| {
                if let Some(ref author) = query_book.author {
                    q.and_where(Expr::col(Book::Author).eq(author));
                }
            },
            |_| {}
        )
        .conditions(
            query_book.name.is_some(),
            |q| {
                if let Some(ref name) = query_book.name {
                    q.and_where(Expr::col(Book::Name).eq(name));
                }
            },
            |_| {}
        )
        .build_sqlx(PostgresQueryBuilder);

    info!("Query: {}", sql);
    info!("Values: {:?}", values);
    
    let books = sqlx::query_as_with::<_, BookRow, _>(&sql, values)
        .fetch_all(&service_state.pg_pool)
        .await
        .map_err(|e| map_find_error(e, &query_book))?;

    Ok(books)
}

fn map_sql_fetch_error(err: sqlx::Error, id : i32) -> RepoError {
    error!("Could not fetch  book: {id}, error: {:?}", err);
    RepoError::FetchBookError
}

fn map_sql_list_error(err: sqlx::Error) -> RepoError {
    error!("Could not list books, error: {:?}", err);
    RepoError::ListBookError
}

fn map_create_values_error(err: sea_query::error::Error, create_book: &CreateBookRequest) -> RepoError {
    error!("Could not create book: {:?}, error: {:?}", create_book, err);
    RepoError::CreateBookValuesError
}

fn map_create_error(err: sqlx::Error, create_book: &CreateBookRequest) -> RepoError {
    error!("Could not create book: {:?}, error: {:?}", create_book, err);
    RepoError::CreateBookError
}

fn map_find_error(err: sqlx::Error, query_book: &QueryBookRequest) -> RepoError {
    error!("Could not find books: {:?}, error: {:?}", query_book, err);
    RepoError::QueryBookError
}