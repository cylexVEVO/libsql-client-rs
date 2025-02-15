use crate::client::Config;
use anyhow::Result;
use async_trait::async_trait;

use crate::{BatchResult, ResultSet, Statement};

/// Database client. This is the main structure used to
/// communicate with the database.
pub struct Client {
    client: hrana_client::Client,
    client_future: hrana_client::ConnFut,
    stream: hrana_client::Stream,
}

impl Client {
    /// Creates a database client with JWT authentication.
    ///
    /// # Arguments
    /// * `url` - URL of the database endpoint
    /// * `token` - auth token
    pub async fn new(url: impl Into<String>, token: impl Into<String>) -> Result<Self> {
        let token = token.into();
        let url = url.into();
        let (client, client_future) =
            hrana_client::Client::connect(&url, if token.is_empty() { None } else { Some(token) })
                .await?;
        let stream = client.open_stream().await?;
        Ok(Self {
            client,
            client_future,
            stream,
        })
    }

    /// Creates a database client, given a `Url`
    ///
    /// # Arguments
    /// * `url` - `Url` object of the database endpoint. This cannot be a relative URL;
    ///
    /// # Examples
    ///
    /// ```
    /// # use libsql_client::reqwest::Client;
    /// use url::Url;
    ///
    /// let url = Url::parse("https://localhost:8080?jwt=<access token>").unwrap();
    /// let db = Client::from_url(url).unwrap();
    /// ```
    pub async fn from_url<T: TryInto<url::Url>>(url: T) -> anyhow::Result<Client>
    where
        <T as TryInto<url::Url>>::Error: std::fmt::Display,
    {
        let url: url::Url = url
            .try_into()
            .map_err(|e| anyhow::anyhow!(format!("{e}")))?;
        let url_str = if url.scheme() == "libsql" {
            let new_url = format!("wss://{}", url.as_str().strip_prefix("libsql://").unwrap());
            url::Url::parse(&new_url).unwrap().to_string()
        } else {
            url.to_string()
        };
        let mut params = url.query_pairs();
        // Try a jwt=XXX parameter first, continue if not found
        if let Some((_, token)) = params.find(|(param_key, _)| param_key == "jwt") {
            Client::new(url_str, token).await
        } else {
            Client::new(url_str, "").await
        }
    }

    /// Creates a database client from a `Config` object.
    pub async fn from_config(config: Config) -> Result<Self> {
        Self::new(config.url, config.auth_token.unwrap_or_default()).await
    }

    pub async fn shutdown(self) -> Result<()> {
        self.client.shutdown().await?;
        self.client_future.await?;
        Ok(())
    }
}

#[async_trait(?Send)]
impl crate::DatabaseClient for Client {
    async fn raw_batch(
        &self,
        stmts: impl IntoIterator<Item = impl Into<Statement>>,
    ) -> anyhow::Result<BatchResult> {
        let mut batch = hrana_client::proto::Batch::new();

        for stmt in stmts.into_iter() {
            let stmt: Statement = stmt.into();
            let mut hrana_stmt = hrana_client::proto::Stmt::new(stmt.sql, true);
            for param in stmt.args {
                hrana_stmt.bind(param);
            }
            batch.step(None, hrana_stmt);
        }
        self.stream
            .execute_batch(batch)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn execute(&self, stmt: impl Into<Statement>) -> Result<ResultSet> {
        let stmt: Statement = stmt.into();
        let mut hrana_stmt = hrana_client::proto::Stmt::new(stmt.sql, true);
        for param in stmt.args {
            hrana_stmt.bind(param);
        }

        self.stream
            .execute(hrana_stmt)
            .await
            .map(ResultSet::from)
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}
