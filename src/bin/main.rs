use chrono::Local;
use docql::{main as lib_main, GraphqlRequest, Runtime as RuntimeTrait};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::TryFrom as _;

struct Runtime {
    client: reqwest::Client,
}

#[async_trait::async_trait(?Send)]
impl RuntimeTrait for Runtime {
    type Error = String;

    async fn date(&self) -> Result<String, Self::Error> {
        Ok(Local::today().format("%Y-%m-%d").to_string())
    }

    async fn get_args(&self) -> Result<Vec<String>, Self::Error> {
        Ok(std::env::args().skip(1).collect())
    }

    async fn query(
        &self,
        url: &str,
        graphql: &GraphqlRequest,
        headers: HashMap<String, String>,
    ) -> Result<Value, Self::Error> {
        let mut header_map = HeaderMap::new();

        for (key, value) in headers {
            let name = HeaderName::try_from(key.as_str())
                .map_err(|e| format!("Invalid header name '{}': {}", key, e))?;
            let value = HeaderValue::from_str(&value)
                .map_err(|e| format!("Invalid header value '{}': {}", value, e))?;
            header_map.insert(name, value);
        }

        let response = self
            .client
            .post(url)
            .json(graphql)
            .headers(header_map)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        Ok(response)
    }

    async fn prepare_output_directory(&self, output: &str) -> Result<(), Self::Error> {
        tokio::fs::create_dir_all(output)
            .await
            .map_err(|e| e.to_string())
    }

    async fn write_file(
        &self,
        output: &str,
        file: &str,
        contents: &str,
    ) -> Result<(), Self::Error> {
        tokio::fs::write(format!("{}/{}", output, file), contents)
            .await
            .map_err(|e| e.to_string())
    }
}

#[tokio::main]
async fn main() {
    let runtime = Runtime {
        client: reqwest::Client::default(),
    };

    match lib_main(runtime).await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(err.exit_code());
        }
    };
}
