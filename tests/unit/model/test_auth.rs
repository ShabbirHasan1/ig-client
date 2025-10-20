use chrono::Utc;
use ig_client::model::auth::{
    OAuthToken, SecurityHeaders, SessionResponse, V2Response, V3Response,
};

#[test]
fn test_oauth_token_is_expired_not_expired() {
    let token = OAuthToken {
        access_token: "test_token".to_string(),
        refresh_token: "refresh_token".to_string(),
        scope: "scope".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: "3600".to_string(), // 1 hour
        created_at: Utc::now(),
    };

    // Should not be expired with 60 second margin
    assert!(!token.is_expired(60));
}

#[test]
fn test_oauth_token_is_expired_expired() {
    let token = OAuthToken {
        access_token: "test_token".to_string(),
        refresh_token: "refresh_token".to_string(),
        scope: "scope".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: "10".to_string(), // 10 seconds
        created_at: Utc::now() - chrono::Duration::seconds(20), // Created 20 seconds ago
    };

    // Should be expired
    assert!(token.is_expired(1));
}

#[test]
fn test_oauth_token_expire_at() {
    let token = OAuthToken {
        access_token: "test_token".to_string(),
        refresh_token: "refresh_token".to_string(),
        scope: "scope".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: "3600".to_string(),
        created_at: Utc::now(),
    };

    let expire_at = token.expire_at(60);
    assert!(expire_at > 0);
}

#[test]
fn test_oauth_token_clone() {
    let token = OAuthToken {
        access_token: "test_token".to_string(),
        refresh_token: "refresh_token".to_string(),
        scope: "scope".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: "3600".to_string(),
        created_at: Utc::now(),
    };

    let cloned = token.clone();
    assert_eq!(token.access_token, cloned.access_token);
    assert_eq!(token.refresh_token, cloned.refresh_token);
}

#[test]
fn test_oauth_token_serialization() {
    let token = OAuthToken {
        access_token: "test_token".to_string(),
        refresh_token: "refresh_token".to_string(),
        scope: "scope".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: "3600".to_string(),
        created_at: Utc::now(),
    };

    let json = serde_json::to_string(&token).unwrap();
    let deserialized: OAuthToken = serde_json::from_str(&json).unwrap();

    assert_eq!(token.access_token, deserialized.access_token);
    assert_eq!(token.expires_in, deserialized.expires_in);
}

#[test]
fn test_session_response_v3_is_v3() {
    let v3_response = SessionResponse::V3(V3Response {
        client_id: "client123".to_string(),
        account_id: "acc123".to_string(),
        timezone_offset: 0,
        lightstreamer_endpoint: "wss://test.com".to_string(),
        oauth_token: OAuthToken {
            access_token: "token".to_string(),
            refresh_token: "refresh".to_string(),
            scope: "scope".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: "3600".to_string(),
            created_at: Utc::now(),
        },
    });

    assert!(v3_response.is_v3());
    assert!(!v3_response.is_v2());
}

#[test]
fn test_session_response_v2_is_v2() {
    let v2_response = SessionResponse::V2(V2Response {
        account_type: "CFD".to_string(),
        account_info: ig_client::model::auth::AccountInfo {
            balance: 10000.0,
            deposit: 10000.0,
            profit_loss: 0.0,
            available: 10000.0,
        },
        currency_iso_code: "GBP".to_string(),
        currency_symbol: "£".to_string(),
        current_account_id: "acc123".to_string(),
        lightstreamer_endpoint: "wss://test.com".to_string(),
        accounts: vec![],
        client_id: "client123".to_string(),
        timezone_offset: 0,
        has_active_demo_accounts: true,
        has_active_live_accounts: false,
        trailing_stops_enabled: true,
        rerouting_environment: None,
        dealing_enabled: true,
        security_headers: None,
        expires_in: None,
        created_at: Utc::now(),
    });

    assert!(v2_response.is_v2());
    assert!(!v2_response.is_v3());
}

#[test]
fn test_session_response_get_session_v3() {
    let v3_response = SessionResponse::V3(V3Response {
        client_id: "client123".to_string(),
        account_id: "acc123".to_string(),
        timezone_offset: 0,
        lightstreamer_endpoint: "wss://test.com".to_string(),
        oauth_token: OAuthToken {
            access_token: "token".to_string(),
            refresh_token: "refresh".to_string(),
            scope: "scope".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: "3600".to_string(),
            created_at: Utc::now(),
        },
    });

    let session = v3_response.get_session();

    assert_eq!(session.account_id, "acc123");
    assert_eq!(session.client_id, "client123");
    assert_eq!(session.api_version, 3);
    assert!(session.oauth_token.is_some());
    assert!(session.cst.is_none());
}

#[test]
fn test_session_response_get_session_v2() {
    let v2_response = SessionResponse::V2(V2Response {
        account_type: "CFD".to_string(),
        account_info: ig_client::model::auth::AccountInfo {
            balance: 10000.0,
            deposit: 10000.0,
            profit_loss: 0.0,
            available: 10000.0,
        },
        currency_iso_code: "GBP".to_string(),
        currency_symbol: "£".to_string(),
        current_account_id: "acc123".to_string(),
        lightstreamer_endpoint: "wss://test.com".to_string(),
        accounts: vec![],
        client_id: "client123".to_string(),
        timezone_offset: 0,
        has_active_demo_accounts: true,
        has_active_live_accounts: false,
        trailing_stops_enabled: true,
        rerouting_environment: None,
        dealing_enabled: true,
        security_headers: Some(SecurityHeaders {
            x_ig_api_key: "api_key".to_string(),
            cst: "cst_token".to_string(),
            x_security_token: "x_token".to_string(),
        }),
        expires_in: None,
        created_at: Utc::now(),
    });

    let session = v2_response.get_session();

    assert_eq!(session.account_id, "acc123");
    assert_eq!(session.client_id, "client123");
    assert_eq!(session.api_version, 2);
    assert!(session.cst.is_some());
    assert!(session.x_security_token.is_some());
    assert!(session.oauth_token.is_none());
}

#[test]
fn test_session_response_is_expired_v3() {
    let v3_response = SessionResponse::V3(V3Response {
        client_id: "client123".to_string(),
        account_id: "acc123".to_string(),
        timezone_offset: 0,
        lightstreamer_endpoint: "wss://test.com".to_string(),
        oauth_token: OAuthToken {
            access_token: "token".to_string(),
            refresh_token: "refresh".to_string(),
            scope: "scope".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: "3600".to_string(),
            created_at: Utc::now(),
        },
    });

    // Should not be expired
    assert!(!v3_response.is_expired(60));
}

#[test]
fn test_session_response_clone() {
    let v3_response = SessionResponse::V3(V3Response {
        client_id: "client123".to_string(),
        account_id: "acc123".to_string(),
        timezone_offset: 0,
        lightstreamer_endpoint: "wss://test.com".to_string(),
        oauth_token: OAuthToken {
            access_token: "token".to_string(),
            refresh_token: "refresh".to_string(),
            scope: "scope".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: "3600".to_string(),
            created_at: Utc::now(),
        },
    });

    let _cloned = v3_response.clone();
}

#[test]
fn test_session_response_serialization() {
    let v3_response = SessionResponse::V3(V3Response {
        client_id: "client123".to_string(),
        account_id: "acc123".to_string(),
        timezone_offset: 0,
        lightstreamer_endpoint: "wss://test.com".to_string(),
        oauth_token: OAuthToken {
            access_token: "token".to_string(),
            refresh_token: "refresh".to_string(),
            scope: "scope".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: "3600".to_string(),
            created_at: Utc::now(),
        },
    });

    let json = serde_json::to_string(&v3_response).unwrap();
    let _deserialized: SessionResponse = serde_json::from_str(&json).unwrap();
}

#[test]
fn test_security_headers_clone() {
    let headers = SecurityHeaders {
        x_ig_api_key: "api_key".to_string(),
        cst: "cst_token".to_string(),
        x_security_token: "x_token".to_string(),
    };

    let cloned = headers.clone();
    assert_eq!(headers.cst, cloned.cst);
    assert_eq!(headers.x_security_token, cloned.x_security_token);
}
