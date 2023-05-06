use serde_json::Value as JsValue;

#[derive(Debug)]
pub enum RegistryMsg {
    InvokeResult(String),
}

#[derive(Debug)]
pub enum NodeMsg {
    Invoke(String, Vec<JsValue>),
}
