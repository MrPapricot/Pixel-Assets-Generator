pub(crate) trait DBAdapter: Sized + Clone {
    async fn connect(url: &str) -> Result<Self, sqlx::Error>;
}