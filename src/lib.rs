//! ArangoDB support for [bb8] based on the [arangors] crate.
//!
//! The library supports all authentication methods supported by `arangors`,
//! defined by `AuthenticationMethod`.
//!
//! [bb8]: https://crates.io/crates/bb8
//! [arangors]: https://crates.io/crates/arangors
//!
//! # Example
//!
//! Get all accessible databases using JWT authentication:
//!
//! ```
//! use bb8::Pool;
//! use bb8_arangodb::{ArangoConnectionManager, AuthenticationMethod};
//! use arangors::uclient::reqwest::ReqwestClient;
//!
//! tokio_test::block_on(async {
//!     let manager = ArangoConnectionManager::<ReqwestClient>::new(
//!         "http://localhost:8529".to_string(),
//!         AuthenticationMethod::JWTAuth("root".to_string(), "openSesame".to_string())
//!     );
//!
//!     let pool = Pool::builder().max_size(5).build(manager).await.unwrap();
//!
//!     let conn = pool.get().await.unwrap();
//!     let dbs = conn.accessible_databases().await.unwrap();
//!
//!     assert!(!dbs.is_empty());
//! });
//! ```
//!
//! Use basic authentication method:
//!
//! ```
//! use bb8::Pool;
//! use bb8_arangodb::{ArangoConnectionManager, AuthenticationMethod};
//! use arangors::uclient::reqwest::ReqwestClient;
//!
//! tokio_test::block_on(async {
//!     let manager = ArangoConnectionManager::<ReqwestClient>::new(
//!         "http://localhost:8529".to_string(),
//!         AuthenticationMethod::BasicAuth("root".to_string(), "openSesame".to_string())
//!     );
//!
//!     let pool = Pool::builder().max_size(5).build(manager).await.unwrap();
//!
//!     let conn = pool.get().await.unwrap();
//!     let dbs = conn.accessible_databases().await.unwrap();
//!
//!     assert!(!dbs.is_empty());
//! });
//! ```
//!
//! Using no authentication method (not recommended):
//!
//! ```no_run
//! use bb8::Pool;
//! use bb8_arangodb::{ArangoConnectionManager, AuthenticationMethod};
//! use arangors::uclient::reqwest::ReqwestClient;
//!
//! tokio_test::block_on(async {
//!     let manager = ArangoConnectionManager::<ReqwestClient>::new(
//!         "http://localhost:8529".to_string(),
//!         AuthenticationMethod::NoAuth
//!     );
//!
//!     let pool = Pool::builder().max_size(5).build(manager).await.unwrap();
//!
//!     // ...
//! });
//! ```

#![deny(missing_docs, missing_debug_implementations)]

use std::marker::PhantomData;

pub use arangors;
pub use bb8;

use arangors::{uclient, ClientError, Database, GenericConnection};
use async_trait::async_trait;

/// Kind of the authentication method to use when establishing a connection.
#[derive(Debug)]
pub enum AuthenticationMethod {
    /// Use basic authentication with a username and password for API calls.
    BasicAuth(String, String),
    /// Use JWT authentication with a token for API calls after authenticating
    /// with username and password.
    JWTAuth(String, String),
    /// Use no authentication for API calls. This is only recommended for local
    /// development.
    NoAuth,
}

/// A connection manager for ArangoDB.
#[derive(Debug)]
pub struct ArangoConnectionManager<C: uclient::ClientExt> {
    url: String,
    method: AuthenticationMethod,
    phantom: PhantomData<C>,
    database: Option<String>,
}

impl<C: uclient::ClientExt> ArangoConnectionManager<C> {
    /// Create a new ArangoConnectionManager..
    pub fn new(
        url: String,
        method: AuthenticationMethod,
        database: Option<impl AsRef<str>>,
    ) -> Self {
        Self {
            url,
            method,
            phantom: PhantomData,
            database: database.map(|f| f.as_ref().into()),
        }
    }
}

///
#[derive(Debug)]
pub enum ConnectionType<C: uclient::ClientExt + Send + 'static> {
    ///
    Generic(GenericConnection<C>),
    ///
    Database(Database<C>),
}

impl<C> ConnectionType<C>
where
    C: uclient::ClientExt + Send + 'static,
{
    async fn establish_basic_auth(
        url: &str,
        username: &str,
        password: &str,
        for_database: Option<&str>,
    ) -> Result<ConnectionType<C>, ClientError> {
        let connection = GenericConnection::establish_basic_auth(url, username, password).await?;
        if let Some(db) = for_database {
            let db_conn = connection.db(db).await?;
            Ok(ConnectionType::Database(db_conn))
        } else {
            Ok(ConnectionType::Generic(connection))
        }
    }

    async fn establish_jwt(
        url: &str,
        username: &str,
        password: &str,
        for_database: Option<&str>,
    ) -> Result<ConnectionType<C>, ClientError> {
        let connection = GenericConnection::establish_jwt(url, username, password).await?;
        if let Some(db) = for_database {
            let db_conn = connection.db(db).await?;
            Ok(ConnectionType::Database(db_conn))
        } else {
            Ok(ConnectionType::Generic(connection))
        }
    }
    async fn establish_without_auth(
        url: &str,
        for_database: Option<&str>,
    ) -> Result<ConnectionType<C>, ClientError> {
        let connection = GenericConnection::establish_without_auth(url).await?;
        if let Some(db) = for_database {
            let db_conn = connection.db(db).await?;
            Ok(ConnectionType::Database(db_conn))
        } else {
            Ok(ConnectionType::Generic(connection))
        }
    }
}

#[async_trait]
impl<C: uclient::ClientExt + Send + 'static> bb8::ManageConnection for ArangoConnectionManager<C> {
    type Connection = ConnectionType<C>;
    type Error = ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match &self.method {
            AuthenticationMethod::BasicAuth(username, password) => {
                Self::Connection::establish_basic_auth(
                    &self.url,
                    username,
                    password,
                    self.database.as_deref(),
                )
                .await
            }
            AuthenticationMethod::JWTAuth(username, password) => {
                Self::Connection::establish_jwt(
                    &self.url,
                    username,
                    password,
                    self.database.as_deref(),
                )
                .await
            }
            AuthenticationMethod::NoAuth => {
                Self::Connection::establish_without_auth(&self.url, self.database.as_deref()).await
            }
        }
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        match conn {
            ConnectionType::Generic(conn) => match conn.accessible_databases().await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            ConnectionType::Database(conn) => match conn.accessible_collections().await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
        }
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}
