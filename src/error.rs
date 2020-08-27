/// All of the possible errors that can occur
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The call to get the current date from the runtime failed
    #[error("Failed to retrieve current date: {0}")]
    Date(String),

    /// The call to get the command-line arguments from the runtime failed
    #[error("Failed to retrieve args: {0}")]
    Args(String),

    /// The call to get the GraphQL Schema from the runtime via a GraphQL endpoint failed
    #[error("Failed to execute introspection query: {0}")]
    Query(String),

    /// The call to get the GraphQL Schema from the runtime via a schema file failed
    #[error("Failed to read schema file: {0}")]
    ReadSchemaFile(String),

    /// The call to the runtime to prepare the output directory failed
    #[error("Failed to prepare output directory '{0}': {1}")]
    PrepareOutputDirectory(String, String),

    /// The call to the runtime to write a file failed
    #[error("Failed to write file '{0}': {1}")]
    WriteFile(String, String),

    /// Parsing the GraphQL schema as a serde object failed
    #[error("Failed to parse GraphQL Introspection response: {0}")]
    Serde(#[from] serde_json::Error),

    /// Loading a handlebars template failed
    #[error("Failed to load handlebars template: {0}")]
    HandlebarsTemplate(#[from] handlebars::TemplateError),

    /// Rendering a handlebars template failed
    #[error("Failed to render handlebars template: {0}")]
    HandlebarsRender(#[from] handlebars::RenderError),

    /// An error occurred parsing arguments
    #[error("Failed to parse arguments: {0}")]
    ClapError(#[from] clap::Error),
}

impl Error {
    /// The process exit code to use for each type of error
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::ClapError(_) => 2,
            Self::Date(_) => 10,
            Self::Args(_) => 11,
            Self::Query(_) => 12,
            Self::ReadSchemaFile(_) => 13,
            Self::PrepareOutputDirectory(_, _) => 20,
            Self::WriteFile(_, _) => 21,
            Self::Serde(_) => 30,
            Self::HandlebarsTemplate(_) | Self::HandlebarsRender(_) => 31,
        }
    }
}

/// Alias for a `Result` with the error type `docql::Error`.
pub type Result<T, E = Error> = std::result::Result<T, E>;
