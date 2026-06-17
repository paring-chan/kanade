use sqlx::{AssertSqlSafe, PgPool, Postgres};
use uuid::Uuid;

pub trait DatabaseSecurityExt: Send + Sync {
    fn begin_as(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = crate::Result<sqlx::Transaction<'_, Postgres>>> + Send;
}

impl DatabaseSecurityExt for PgPool {
    #[instrument(skip(self), err(Debug))]
    async fn begin_as(&self, user_id: Uuid) -> crate::Result<sqlx::Transaction<'_, Postgres>> {
        let mut tx = self.begin().await?;

        sqlx::query(AssertSqlSafe(format!(
            r#"SET LOCAL app.auth_user_id ='{}'"#,
            user_id
        )))
        .execute(&mut *tx)
        .await?;

        Ok(tx)
    }
}
