use crate::common;
use ig_client::prelude::*;
use tracing::info;

#[test]
#[ignore]
fn test_login() {
    setup_logger();
    // Get a session
    let session = common::login_with_account_switch();

    // Verify the session contains the expected fields
    assert!(session.cst.is_some(), "CST token should be present");
    assert!(
        session.x_security_token.is_some(),
        "Security token should be present"
    );
    assert!(
        !session.account_id.is_empty(),
        "Account ID should not be empty"
    );

    info!("Login successful. Account ID: {}", session.account_id);
}

// test_account_switch removed - account switching is now handled automatically
// by the Client during initialization based on the configured account_id
