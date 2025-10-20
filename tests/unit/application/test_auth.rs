use chrono::Utc;
use ig_client::application::auth::{Session, WebsocketInfo};
use ig_client::model::auth::OAuthToken;

fn make_session(expires_in_secs: i64, with_oauth: bool) -> Session {
    let now = Utc::now().timestamp() as u64;
    Session {
        account_id: "ACC123".to_string(),
        client_id: "CLIENT1".to_string(),
        lightstreamer_endpoint: "https://ls.example.com".to_string(),
        cst: Some("CSTTOKEN".to_string()),
        x_security_token: Some("XSTOKEN".to_string()),
        oauth_token: if with_oauth {
            Some(OAuthToken {
                access_token: "AT".to_string(),
                refresh_token: "RT".to_string(),
                token_type: "Bearer".to_string(),
                expires_in: "3600".to_string(),
                scope: "read write".to_string(),
                created_at: Default::default(),
            })
        } else {
            None
        },
        api_version: if with_oauth { 3 } else { 2 },
        expires_at: now + (expires_in_secs as u64),
    }
}

#[test]
fn websocket_info_password_formats_and_empty_when_missing() {
    let ws = WebsocketInfo {
        server: "https://ls".into(),
        cst: Some("CST123".into()),
        x_security_token: Some("XST456".into()),
        account_id: "ACC123".into(),
    };
    assert_eq!(ws.get_ws_password(), "CST-CST123|XST-XST456");

    let ws_missing = WebsocketInfo {
        server: "https://ls".into(),
        cst: None,
        x_security_token: Some("XST456".into()),
        account_id: "ACC123".into(),
    };
    assert_eq!(ws_missing.get_ws_password(), "");
}

#[test]
fn session_is_oauth_and_ws_info_propagates() {
    let s_oauth = make_session(120, true);
    assert!(s_oauth.is_oauth());

    let s_v2 = make_session(120, false);
    assert!(!s_v2.is_oauth());

    // Websocket info
    let ws_info = s_v2.get_websocket_info();
    assert_eq!(ws_info.server, "https://ls.example.com/lightstreamer");
    assert_eq!(ws_info.cst.as_deref(), Some("CSTTOKEN"));
    assert_eq!(ws_info.x_security_token.as_deref(), Some("XSTOKEN"));
    assert_eq!(ws_info.account_id, "ACC123");
}

#[test]
fn session_expiry_checks_and_alias() {
    // Expires in 2 minutes
    let s = make_session(120, true);

    // With default margin (60s), should be valid
    assert!(!s.is_expired(None));

    // With larger margin (180s), should be considered expiring
    assert!(s.is_expired(Some(180)));

    // Alias should behave the same
    assert_eq!(s.needs_token_refresh(None), s.is_expired(None));

    // Seconds until expiry should be positive and <= 120
    let secs = s.seconds_until_expiry();
    assert!(secs <= 120 && secs > 0);
}
