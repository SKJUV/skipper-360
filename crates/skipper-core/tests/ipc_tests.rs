use skipper_core::{Request, Response, ResponseStatus};

#[test]
fn test_ipc_request_response_roundtrip() {
    let req = Request::new("status", serde_json::json!({ "verbose": true }));
    assert_eq!(req.command, "status");

    let resp = Response::ok(
        "System is operational",
        Some(serde_json::json!({ "pid": 1234 })),
        &req.request_id,
    );
    assert_eq!(resp.status, ResponseStatus::Ok);
    assert_eq!(resp.request_id, req.request_id);

    let err_resp = Response::error("Daemon error", &req.request_id);
    assert_eq!(err_resp.status, ResponseStatus::Error);
}
