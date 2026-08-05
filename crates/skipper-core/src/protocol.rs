use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub command: String,
    pub args: serde_json::Value,
    pub request_id: String,
}

impl Request {
    pub fn new(command: impl Into<String>, args: serde_json::Value) -> Self {
        Self {
            command: command.into().trim().to_string(),
            args,
            request_id: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos().to_string())
                .unwrap_or_else(|_| "0".into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponseStatus {
    Ok,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: ResponseStatus,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub request_id: String,
}

impl Response {
    pub fn ok(
        message: impl Into<String>,
        data: Option<serde_json::Value>,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            status: ResponseStatus::Ok,
            message: message.into(),
            data,
            request_id: request_id.into(),
        }
    }

    pub fn error(message: impl Into<String>, request_id: impl Into<String>) -> Self {
        Self {
            status: ResponseStatus::Error,
            message: message.into(),
            data: None,
            request_id: request_id.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub enum StreamMessage {
    Stdout(String),
    Stderr(String),
    PromptDetected { command: String, pattern: String },
    PasswordInjected { command: String },
    ProcessExited { code: i32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_response_serialization() {
        let req = Request::new("activate ", serde_json::json!({}));
        let json_req = serde_json::to_string(&req).expect("Failed to serialize Request");
        let parsed_req: Request =
            serde_json::from_str(&json_req).expect("Failed to deserialize Request");
        assert_eq!(parsed_req.command, "activate");

        let resp = Response::ok("Activated", None, &req.request_id);
        let json_resp = serde_json::to_string(&resp).expect("Failed to serialize Response");
        let parsed_resp: Response =
            serde_json::from_str(&json_resp).expect("Failed to deserialize Response");
        assert_eq!(parsed_resp.status, ResponseStatus::Ok);
        assert_eq!(parsed_resp.message, "Activated");
    }
}
