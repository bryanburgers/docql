use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// The GraphQL introspection query that will be run
pub const INTROSPECTION_QUERY: &str = include_str!("introspection_query.graphql");
/// The GraphQL introspection request that will be executed
pub const GRAPHQL_REQUEST: GraphqlRequest = GraphqlRequest {
    query: INTROSPECTION_QUERY,
    operation_name: "IntrospectionQuery",
};

/// A structure representing a GraphQL request.
#[derive(Debug, Serialize, Deserialize)]
pub struct GraphqlRequest {
    /// The GraphQL query that will be executed
    pub query: &'static str,
    /// The operation name within the query to execute
    pub operation_name: &'static str,
}

/// The trait that all tools that use this library must implement.
///
/// A runtime provides all of the "outside world" interaction that the library can use to do its
/// job.
///
/// This allows docql to be implemented as a native binary and as a wasm-module.
#[async_trait(?Send)]
pub trait Runtime {
    /// The error type that the runtime returns.
    type Error: ToString;

    /// Get the current date as an ISO-8601 date
    async fn date(&self) -> Result<String, Self::Error>;

    /// Get the arguments passed on the command (not including the binary name)
    ///
    /// Note that standard `argv` includes the binary name. This method expects the binary name to
    /// be stripped from the front. This makes implementing the WASM binary easier.
    async fn get_args(&self) -> Result<Vec<String>, Self::Error>;

    /// Run the given GraphQL request (the introspection query) against the URL, returning the JSON
    /// response.
    async fn query(
        &self,
        url: &str,
        graphql: &GraphqlRequest,
        headers: HashMap<String, String>,
    ) -> Result<Value, Self::Error>;

    /// Read a file from the filesystem.
    ///
    /// Used when rendering documentation based on an already downloaded schema.
    async fn read_file(&self, path: &str) -> Result<String, Self::Error>;

    /// Prepare the output directory.
    ///
    /// The runtime can use this to create the directory, etc.
    async fn prepare_output_directory(&self, output: &str) -> Result<(), Self::Error>;

    /// Write contents to the given file.
    async fn write_file(&self, output: &str, file: &str, contents: &str)
        -> Result<(), Self::Error>;
}
