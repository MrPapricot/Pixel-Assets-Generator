pub(crate) trait DBAdapter: Sized + Clone {
    async fn connect(url: &str, min_connection: u32, max_connection: u32) -> Result<Self, sqlx::Error>;
}