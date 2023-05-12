use crate::db::models::FunctionType;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsValue;
use tokio::sync::oneshot::Sender;
/// WSProto: the protocol between
/// the axum WS handler and WS clients.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WSProto {
    Invoke {
        // TODO: Change to UUID?
        request_id: String,
        name: String,
        uri: String,
        signature: FunctionType,
        args: Vec<JsValue>,
    },
    Result {
        request_id: String,
        content: JsValue,
    },
}

impl WSProto {
    pub fn invoke_request(
        request_id: String,
        name: String,
        uri: String,
        signature: FunctionType,
        args: Vec<JsValue>,
    ) -> WSProto {
        Self::Invoke {
            request_id,
            name,
            uri,
            signature,
            args,
        }
    }
    pub fn from_json(body: &str) -> Result<Self, serde_json::Error> {
        // TODO: should return result
        serde_json::from_str(body)
    }

    pub fn to_json(&self) -> String {
        // Shouldn't fail..
        serde_json::to_string(self).unwrap()
    }
}

impl Into<RegistryMsg> for WSProto {
    fn into(self) -> RegistryMsg {
        match self {
            Self::Result { content, .. } => RegistryMsg::InvokeResult(content),
            _ => panic!("Cannot cast {:?} to RegistryMsg", self),
        }
    }
}

#[derive(Debug)]
pub enum RegistryMsg {
    InvokeResult(JsValue),
    Disconnected
}

#[derive(Debug)]
pub enum NodeMsg {
    Invoke {
        name: String,
        uri: String,
        signature: FunctionType,
        args: Vec<JsValue>,
        sender: Sender<RegistryMsg>,
    },
}
