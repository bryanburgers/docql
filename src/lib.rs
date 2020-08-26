//! Generate static HTML documentation for GraphQL endpoints
#![deny(missing_docs)]
use chrono::NaiveDate;
use clap::{App, AppSettings, Arg};
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
        .about("Generate documentation for a GraphQL Endpoint")
        .setting(AppSettings::NoBinaryName)
        .arg(
            Arg::with_name("endpoint")
                .short("e")
                .long("endpoint")
                .help("The URL of the GraphQL endpoint to document")
                .required(true)
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
                .validator(|s| {
                    let mut parts = s.splitn(2, ":").skip(1);
                    parts.next().ok_or_else(|| "Header must include a name, a colon, and a value".to_string())?;
                    Ok(())
                })
        )
        .get_matches_from_safe(args)?;

    let url = matches.value_of("endpoint").unwrap();
    let output = matches.value_of("output").unwrap();
    let name = matches.value_of("name").unwrap();

    let date = runtime
        .date()
        .await
        .map_err(|e| Error::Date(e.to_string()))?;
    let date =
        NaiveDate::parse_from_str(&date, "%Y-%m-%d").map_err(|e| Error::Date(e.to_string()))?;

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

    let value = runtime
        .query(url, &runtime::GRAPHQL_REQUEST, headers)
        .await
        .map_err(|e| Error::Query(e.to_string()))?;
    let graphql_response: schema::GraphQLResponse = serde_json::from_value(value)?;
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
