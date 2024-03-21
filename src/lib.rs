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
    database: String,
}

impl<C: uclient::ClientExt> ArangoConnectionManager<C> {
    /// Create a new ArangoConnectionManager..
    pub fn new(url: String, method: AuthenticationMethod, database: String) -> Self {
        Self {
            url,
            method,
            phantom: PhantomData,
            database,
        }
    }
}

#[async_trait]
impl<C: uclient::ClientExt + Send + 'static> bb8::ManageConnection for ArangoConnectionManager<C> {
    type Connection = Database<C>;
    type Error = ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let conn = match &self.method {
            AuthenticationMethod::BasicAuth(username, password) => {
                GenericConnection::establish_basic_auth(&self.url, username, password).await
            }
            AuthenticationMethod::JWTAuth(username, password) => {
                GenericConnection::establish_jwt(&self.url, username, password).await
            }
            AuthenticationMethod::NoAuth => {
                GenericConnection::establish_without_auth(&self.url).await
            }
        }?;
        conn.db(&self.database).await
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        match conn.accessible_collections().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

