//! Generate static HTML documentation for GraphQL APIs.
//!
//!
//! ## Overview
//!
//! [GraphiQL] is great. So are tools like [Altair] and [Insomnia]. But they aren't
//! necessarily enough.
//!
//! `docql` comes in when you want documentation for GraphQL APIs that lives in a
//! shared place. Having HTML documentation allows teams to link to specific
//! objects and fields to enhance conversation, reference the docs when away from
//! the computer, and generally have a place to see the entire GraphQL schema at a
//! glance.
//!
//! [GraphiQL]: https://github.com/graphql/graphiql
//! [Altair]: https://altair.sirmuel.design/
//! [Insomnia]: https://insomnia.rest/graphql/
//!
//! ## Examples
//!
//! * [GitHub v4 API][github v4]: [generated][github v4 generated]
//! * [GraphQL's example Star Wars API][swapi]: [generated][swapi generated]
//!
//! [github v4]: https://docs.github.com/en/graphql
//! [swapi]: https://swapi.graph.cool/
//! [github v4 generated]: https://bryanburgers.github.io/docql/github/
//! [swapi generated]: https://bryanburgers.github.io/docql/swapi/
//!
//!
//! ## Use
//!
//! There are two ways to use `docql`.
//!
//! ### npx
//!
//! The easiest way to get started is to run `docql` off of the npm registry.
//!
//! ```text
//! npx docql -e $API -o ./doc
//! ```
//!
//!
//! ### native binaries
//!
//! If native binaries are more your style and you have access to [Rust]'s `cargo`,
//! you can install with `cargo install`.
//!
//! ```text
//! cargo install docql
//! docql -e $API -o ./doc
//! ```
//!
//! [crates.io]: https://crates.io
//! [Rust]: https://rust-lang.org
//!
//!
//! ## Command line options
//!
//! ```text
//! USAGE:
//!     docql [OPTIONS] --output <path> <--endpoint <url>|--schema <path>>
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! OPTIONS:
//!     -e, --endpoint <url>        The URL of the GraphQL endpoint to document
//!     -x, --header <header>...    Additional headers when executing the GraphQL introspection query (e.g. `-x
//!                                 "Authorization: Bearer abcdef"`
//!     -n, --name <name>           The name to give to the schema (used in the title of the page) [default: GraphQL Schema]
//!     -o, --output <path>         The directory to put the generated documentation
//!     -s, --schema <path>         The output of a GraphQL introspection query already stored locally
//! ```
#![deny(missing_docs)]
use chrono::NaiveDate;
use clap::{App, AppSettings, Arg, ArgGroup};
use futures::stream::{StreamExt as _, TryStreamExt as _};
use std::collections::HashMap;

mod error;
mod handlebars_helpers;
mod renderer;
mod runtime;
mod schema;
pub use error::{Error, Result};
use renderer::Renderer;
pub use runtime::{GraphqlRequest, Runtime, GRAPHQL_REQUEST, INTROSPECTION_QUERY};

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// The primary entrypoint to run the application.
///
/// This function uses the runtime to get arguments, fetch the GraphQL schema, and write out the
/// results to the output directory.
pub async fn main(runtime: impl Runtime) -> Result<()> {
    let args = runtime
        .get_args()
        .await
        .map_err(|err| Error::Args(err.to_string()))?;

    let matches = App::new("docql")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Generate documentation for a GraphQL API")
        .setting(AppSettings::NoBinaryName)
        .arg(
            Arg::with_name("endpoint")
                .short("e")
                .long("endpoint")
                .help("The URL of the GraphQL endpoint to document")
                .takes_value(true)
                .value_name("url")
                .validator(|s| match s.parse::<url::Url>() {
                    Ok(url) => {
                        if url.scheme() == "http" || url.scheme() == "https" {
                            Ok(())
                        } else {
                            Err("Endpoint is not an http or https URL".to_string())
                        }
                    }
                    Err(e) => Err(e.to_string()),
                }),
        )
        .arg(
            Arg::with_name("schema")
                .short("s")
                .long("schema")
                .alias("schema-file")
                .help("The output of a GraphQL introspection query already stored locally")
                .takes_value(true)
                .value_name("path")
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("The directory to put the generated documentation")
                .required(true)
                .takes_value(true)
                .value_name("path"),
        )
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .help("The name to give to the schema (used in the title of the page)")
                .takes_value(true)
                .default_value("GraphQL Schema"),
        )
        .arg(
            Arg::with_name("header")
                .short("x")
                .long("header")
                .help("Additional headers when executing the GraphQL introspection query (e.g. `-x \"Authorization: Bearer abcdef\"`")
                .number_of_values(1)
                .multiple(true)
                .takes_value(true)
                .conflicts_with("schema")
                .validator(|s| {
                    let mut parts = s.splitn(2, ":").skip(1);
                    parts.next().ok_or_else(|| "Header must include a name, a colon, and a value".to_string())?;
                    Ok(())
                })
        )
        .group(
            ArgGroup::with_name("source")
                .args(&["endpoint", "schema"])
                .required(true)
        )
        .get_matches_from_safe(args)?;

    let output = matches.value_of("output").unwrap();
    let name = matches.value_of("name").unwrap();

    let source = if let Some(url) = matches.value_of("endpoint") {
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert("user-agent".to_string(), USER_AGENT.to_string());

        if let Some(header_opts) = matches.values_of("header") {
            for header in header_opts {
                // This is known to be safe because we validate it in clap's Arg::validator
                let mut parts = header.splitn(2, ":");
                let name = parts.next().unwrap().trim();
                let value = parts.next().unwrap().trim();
                headers.insert(name.to_string(), value.to_string());
            }
        }

        Source::Endpoint { url, headers }
    } else {
        let path = matches.value_of("schema").unwrap();
        Source::Schema { path }
    };

    let date = runtime
        .date()
        .await
        .map_err(|e| Error::Date(e.to_string()))?;
    let date =
        NaiveDate::parse_from_str(&date, "%Y-%m-%d").map_err(|e| Error::Date(e.to_string()))?;

    let graphql_response = source.get_json(&runtime).await?;
    let schema = graphql_response.data.schema;

    runtime
        .prepare_output_directory(&output)
        .await
        .map_err(|e| Error::PrepareOutputDirectory(output.to_string(), e.to_string()))?;

    let renderer = Renderer::new(name.to_string(), date, &schema)?;

    let index_content = renderer.render_index()?;
    let index_filename = "index.html".to_string();
    runtime
        .write_file(&output, &index_filename, &index_content)
        .await
        .map_err(|e| Error::WriteFile(index_filename, e.to_string()))?;
    let style_filename = "style.css".to_string();
    runtime
        .write_file(
            &output,
            &style_filename,
            include_str!("templates/style.css"),
        )
        .await
        .map_err(|e| Error::WriteFile(style_filename, e.to_string()))?;

    futures::stream::iter(&schema.types)
        .map(|t| write_type(&runtime, &output, &renderer, t))
        .buffered(10)
        .try_collect()
        .await?;

    Ok(())
}

enum Source<'a> {
    Endpoint {
        url: &'a str,
        headers: HashMap<String, String>,
    },
    Schema {
        path: &'a str,
    },
}

impl Source<'_> {
    async fn get_json(self, runtime: &impl Runtime) -> Result<schema::GraphQLResponse> {
        match self {
            Self::Endpoint { url, headers } => Self::get_json_endpoint(url, headers, runtime).await,
            Self::Schema { path } => Self::get_json_schema(path, runtime).await,
        }
    }

    async fn get_json_endpoint(
        url: &str,
        headers: HashMap<String, String>,
        runtime: &impl Runtime,
    ) -> Result<schema::GraphQLResponse> {
        let value = runtime
            .query(url, &runtime::GRAPHQL_REQUEST, headers)
            .await
            .map_err(|e| Error::Query(e.to_string()))?;
        let graphql_response: schema::GraphQLResponse = serde_json::from_value(value)?;
        Ok(graphql_response)
    }

    async fn get_json_schema(
        path: &str,
        runtime: &impl Runtime,
    ) -> Result<schema::GraphQLResponse> {
        let s = runtime
            .read_file(path)
            .await
            .map_err(|e| Error::ReadSchemaFile(e.to_string()))?;

        let graphql_response: schema::GraphQLResponse = serde_json::from_str(&s)?;
        Ok(graphql_response)
    }
}

async fn write_type(
    runtime: &impl Runtime,
    output: &str,
    renderer: &Renderer<'_>,
    full_type: &schema::FullType,
) -> Result<()> {
    let file_name = format!("{}.{}.html", full_type.kind.prefix(), full_type.name);

    let content = match full_type.kind {
        schema::Kind::Object => Some(renderer.render_object(&full_type)?),
        schema::Kind::InputObject => Some(renderer.render_input_object(&full_type)?),
        schema::Kind::Scalar => Some(renderer.render_scalar(&full_type)?),
        schema::Kind::Enum => Some(renderer.render_enum(&full_type)?),
        schema::Kind::Interface => Some(renderer.render_interface(&full_type)?),
        schema::Kind::Union => Some(renderer.render_union(&full_type)?),
        schema::Kind::List => None,
        schema::Kind::NonNull => None,
    };

    if let Some(content) = content {
        runtime
            .write_file(output, &file_name, &content)
            .await
            .map_err(|e| Error::WriteFile(file_name, e.to_string()))?;
    }

    Ok(())
}
