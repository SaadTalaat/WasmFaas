use serde::{Deserialize, Serialize};
use serde_json::{to_value, Value as JsValue};

#[derive(Serialize, Deserialize)]
pub enum WSProtoKind {
    Invoke,
    Result,
}

#[derive(Serialize, Deserialize)]
pub struct WSProto {
    kind: WSProtoKind,
    body: JsValue,
}

impl WSProto {
    pub fn invoke_request(name: String, args: Vec<JsValue>) -> String {
        let body = InvokeMsg { name, args };
        let body = to_value(body).unwrap();
        let instance = Self {
            kind: WSProtoKind::Invoke,
            body,
        };
        serde_json::to_string(&instance).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct InvokeMsg {
    name: String,
    args: Vec<JsValue>,
}
