use async_trait::async_trait;
use docql::{main as lib_main, GraphqlRequest, Runtime as RuntimeTrait};
use serde_json::Value;
use wasm_bindgen::{prelude::*, JsCast as _};
use std::collections::HashMap;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);
}

#[wasm_bindgen]
#[rustfmt::skip]
extern "C" {
    #[wasm_bindgen(js_name = Runtime)]
    pub type Runtime;

    #[wasm_bindgen(method, catch)]
    async fn date(this: &Runtime) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(method, catch, js_name = getArgs)]
    async fn get_args(this: &Runtime) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(method, catch)]
    async fn query(this: &Runtime, url: String, graphql: JsValue, headers: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(method, catch, js_name = readFile)]
    async fn read_file(this: &Runtime, path: String) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(method, catch, js_name = prepareOutputDirectory)]
    async fn prepare_output_directory(this: &Runtime, output: String) -> Result<(), JsValue>;
    #[wasm_bindgen(method, catch, js_name = writeFile)]
    async fn write_file(this: &Runtime, output: String, file: String, contents: String) -> Result<(), JsValue>;
}

struct WasmRuntime(Runtime);

#[async_trait(?Send)]
impl RuntimeTrait for WasmRuntime {
    type Error = String;

    async fn date(&self) -> Result<String, Self::Error> {
        let date = self.0.date().await.map_err(javascript_to_string)?;
        Ok(javascript_to_string(date))
    }

    async fn get_args(&self) -> Result<Vec<String>, Self::Error> {
        let args = self.0.get_args().await.map_err(javascript_to_string)?;
        let args = js_sys::Array::from(&args);
        let args_str: Vec<_> = args.iter().map(javascript_to_string).collect();
        Ok(args_str)
    }

    async fn query(&self, url: &str, graphql: &GraphqlRequest, headers: HashMap<String, String>) -> Result<Value, Self::Error> {
        let request = JsValue::from_serde(graphql)
            .map_err(|err| format!("Failed to convert request from serde: {}", err))?;
        let headers = JsValue::from_serde(&headers)
            .map_err(|err| format!("Failed to convert headers from serde: {}", err))?;
        let s = self.0.query(url.to_string(), request, headers).await.map_err(javascript_to_string)?;
        let r = s
            .into_serde()
            .map_err(|err| format!("Failed to convert result into serde: {}", err))?;
        Ok(r)
    }

    async fn read_file(&self, path: &str) -> Result<String, Self::Error> {
        let s = self.0.read_file(path.to_string()).await.map_err(javascript_to_string)?;
        Ok(javascript_to_string(s))
    }

    async fn prepare_output_directory(&self, output: &str) -> Result<(), Self::Error> {
        self.0
            .prepare_output_directory(output.to_string())
            .await
            .map_err(javascript_to_string)?;
        Ok(())
    }

    async fn write_file(
        &self,
        output: &str,
        file: &str,
        contents: &str,
    ) -> Result<(), Self::Error> {
        self.0
            .write_file(output.to_string(), file.to_string(), contents.to_string())
            .await
            .map_err(javascript_to_string)?;
        Ok(())
    }
}

fn javascript_to_string(value: JsValue) -> String {
    if value.is_undefined() { return "<undefined>".to_string(); }
    if value.is_null() { return "<null>".to_string(); }
    if let Some(b) = value.as_bool() { return format!("{}", b); }
    if let Some(n) = value.as_f64() { return format!("{}", n); }
    if let Some(s) = value.as_string() { return s; }

    match value.dyn_into::<js_sys::Object>() {
        Ok(object) => {
            let js_str = object.to_string();
            js_str.as_string().unwrap()
        }
        Err(value) => {
            match value.dyn_into::<js_sys::Symbol>() {
                Ok(symbol) => {
                    let js_str = symbol.to_string();
                    js_str.as_string().unwrap()
                }
                Err(value) => {
                    format!("{:?}", value)
                }
            }
        }
    }
}

#[wasm_bindgen(catch)]
pub async fn main(runtime: Runtime) -> Result<(), JsValue> {
    let runtime = WasmRuntime(runtime);

    lib_main(runtime)
        .await.map_err(|e| {
            let error = js_sys::Error::new(&e.to_string());
            let exit_code = e.exit_code();
            js_sys::Reflect::set(&error, &JsValue::from_str("exitCode"), &JsValue::from_f64(exit_code as f64)).unwrap();
            error
        })?;

    Ok(())
}
