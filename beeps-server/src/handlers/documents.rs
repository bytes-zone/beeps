use crate::conn::Conn;
use crate::error::Error;
use crate::jwt::Claims;
use axum::Json;
use beeps_core::sync::documents;
use sqlx::query;

#[tracing::instrument]
pub async fn handler(Conn(mut conn): Conn, claims: Claims) -> Result<Json<documents::Resp>, Error> {
    let documents = query!(
        "SELECT documents.id as id, updated_at, created_at \
        FROM documents \
        JOIN accounts ON accounts.id = documents.owner_id \
        WHERE accounts.email = $1",
        claims.sub,
    )
    .fetch_all(&mut *conn)
    .await?;

    Ok(Json(documents::Resp {
        documents: documents
            .into_iter()
            .map(|row| documents::Document {
                id: row.id,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect(),
    }))
}
