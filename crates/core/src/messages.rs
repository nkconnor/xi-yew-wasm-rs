use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
};
use uuid::Uuid;
use serde_json::Value;



/// ViewIds are the primary means of routing messages between
/// xi-core and a client view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ViewId(usize);

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientStartedParams {
    #[serde(default)]
    pub config_dir: Option<PathBuf>,
    /// Path to additional plugins, included by the client.
    #[serde(default)]
    pub client_extras_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrollParams(Vec<usize>);

#[derive(Debug, Serialize, Deserialize)]
pub struct NewViewParams{
    pub file_path: Option<String>
}

// always { method: "", params: "", .. sometimes extra, like id: "" }

/// Sent from client to server, this shared model is used for all server communication
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method")]
pub enum ClientMessage {
    Ping { v: i64 },

    // CloseView { view_id: ViewId },
    // Save { view_id: ViewId, file_path: String },
    ClientStarted {
        params: ClientStartedParams
    },

    NewView { id: usize, params: NewViewParams },

    Scroll{
        params: ScrollParams
    },

    GetVersion
}

impl ClientMessage {
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }

    pub fn from_binary(b: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(b)
    }

    pub fn to_binary(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(&self)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigChangedParamsChanges{
    auto_indent: bool,
    autodetect_whitespace: bool,
    font_face: String,
    font_size: u64,
    line_ending: String,
    plugin_search_path: Vec<String>,
    save_with_newline:  bool,
    scroll_past_end: bool,
    surrounding_pairs: Vec<Vec<String>>,
    tab_size: u64,
    translate_tabs_to_spaces: bool,
    use_tab_stops: bool,
    word_wrap: bool,
    wrap_width: u64
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag="type")]
pub enum Annotation {
    Selection {
        n: u64,
        ranges: Vec<Vec<u64>>
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Line {
    cursor: Option<Vec<u64>>,
    ln: u64,
    styles: Vec<u64>,
    text: String,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum OpType {
    #[serde(rename = "ins")]
    Insert,
    Skip,
    Invalidate,
    Copy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct UpdateOp {
    op: OpType,
    n: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    lines: Option<Vec<Line>>,
    #[serde(rename = "ln")]
    #[serde(skip_serializing_if = "Option::is_none")]
    first_line_number: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateUpdateParams{
    annotations: Vec<Annotation>,
    ops: Vec<UpdateOp>,
    pristine: bool
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag="method", content="params")]
pub enum Method {
    AvailableLanguages{
        languages: Vec<String>
    },
    AvailableThemes{
        themes: Vec<String>
    },
    AvailablePlugins {
        plugins: Vec<String>,
        view_id: String
    },
    ConfigChanged {
        changes: ConfigChangedParamsChanges,
        view_id: String
    },
    LanguageChanged {
        language_id: String,
        view_id: String
    },
    Update {
        update: UpdateUpdateParams,
        view_id: String
    },
    ScrollTo {
        col: u64,
        line: u64,
        view_id: String
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum Notification {
    Result {
        id: u64,
        result: String
    }
}

/// Sent from server to client, this shared model is used for all client communication
#[allow(variant_size_differences)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerMessage {
    Connected {
        connection_id: Uuid,
        user_id: Uuid,
        b: bool
    },
    ServerError {
        reason: String,
        content: String
    },
    Pong {
        v: i64
    },
    EditorMethod(Method),
    EditorNotification(Notification),
}



impl ServerMessage {

    pub fn from_xi_json(s: &str) -> Result<Self, serde_json::Error> {
        let v: Value = serde_json::from_str(s).unwrap();
        match v.get("result") {
            Some(id) => {
                serde_json::from_value(v).map(|n: Notification| ServerMessage::EditorNotification(n))
            },
            None => {
                serde_json::from_value(v).map(|m: Method| ServerMessage::EditorMethod(m))
            }
        }
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }

    pub fn from_binary(b: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(b)
    }

    pub fn to_binary(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(&self)
    }
}