use crate::auth::Claims;
use crate::conn::Conn;
use crate::error::Error;
use sqlx::query;

#[tracing::instrument]
pub async fn handler(claims: Claims, Conn(mut conn): Conn) -> Result<String, Error> {
    let aggregate = query!(
        "SELECT MAX(node) FROM operations WHERE document_id = $1",
        claims.document_id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|err| {
        tracing::error!(?err, "error querying");
        Error::internal_server_error("error querying")
    })?;

    Ok(aggregate.max.unwrap_or(0).to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::auth::Claims;
    use crate::conn::Conn;
    use sqlx::{pool::PoolConnection, Postgres};

    #[sqlx::test]
    async fn test_handler(pool: PoolConnection<Postgres>) {
        let res = handler(Claims::test(1, 1), Conn(pool)).await.unwrap();

        assert_eq!(res, "0");
    }
}
