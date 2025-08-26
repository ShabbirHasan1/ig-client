use ig_client::application::services::MarketService;
use ig_client::utils::rate_limiter::RateLimitType;
use ig_client::{
    application::services::market_service::MarketServiceImpl, config::Config,
    session::auth::IgAuth, session::interface::IgAuthenticator,
    transport::http_client::IgHttpClientImpl, utils::logger::setup_logger,
};
use std::sync::Arc;
use tabled::{Table, Tabled};
use tracing::{error, info};

// Struct for displaying DBEntry data in a table format
#[derive(Tabled)]
struct DBEntryDisplay {
    #[tabled(rename = "Symbol")]
    symbol: String,
    #[tabled(rename = "Epic")]
    epic: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    instrument_type: String,
    #[tabled(rename = "Exchange")]
    exchange: String,
    #[tabled(rename = "Expiry")]
    expiry: String,
    #[tabled(rename = "Last Update")]
    last_update: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    // Create configuration using the default Config implementation
    let config = Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.7,
    ));
    
    // Create HTTP client
    let http_client = Arc::new(IgHttpClientImpl::new(Arc::clone(&config)));
    info!("HTTP client created");

    // Create market service
    let market_service = MarketServiceImpl::new(Arc::clone(&config), Arc::clone(&http_client));
    info!("Market service created");

    // Create authenticator
    let authenticator = IgAuth::new(&config);
    info!("Authenticator created");

    // Login to IG with detailed error handling
    info!("Attempting to login to IG...");
    let session = authenticator.login_and_switch_account(
        &config.credentials.account_id,
        Some(false)
    ).await?;
    


    // Get vec DB entries
    info!("\n=== Fetching Vec DB Entries ===");
    match market_service.get_vec_db_entries(&session).await {
        Ok(db_entries) => {
            if db_entries.is_empty() {
                info!("📊 No DB entries found");
            } else {
                info!("📊 Found {} DB entries", db_entries.len());
                
                // Convert DBEntry to DBEntryDisplay for table formatting
                let display_entries: Vec<DBEntryDisplay> = db_entries
                    .iter()
                    .map(|entry| DBEntryDisplay {
                        symbol: entry.symbol.clone(),
                        epic: if entry.epic.len() > 25 {
                            format!("{}...", &entry.epic[..22])
                        } else {
                            entry.epic.clone()
                        },
                        name: if entry.name.len() > 30 {
                            format!("{}...", &entry.name[..27])
                        } else {
                            entry.name.clone()
                        },
                        instrument_type: format!("{:?}", entry.instrument_type),
                        exchange: entry.exchange.clone(),
                        expiry: if entry.expiry.is_empty() {
                            "N/A".to_string()
                        } else {
                            entry.expiry.clone()
                        },
                        last_update: entry.last_update.format("%Y-%m-%d %H:%M:%S").to_string(),
                    })
                    .collect();
                
                // Create and display the table
                let table = Table::new(display_entries)
                    .with(tabled::settings::Style::rounded())
                    .with(tabled::settings::Modify::new(tabled::settings::object::Rows::first())
                        .with(tabled::settings::Alignment::center()))
                    .to_string();
                
                println!("\n🎯 Market DB Entries Table:");
                println!("{}", table);
                
                // Display summary statistics
                info!("\n📈 Summary Statistics:");
                let unique_symbols: std::collections::HashSet<String> = db_entries
                    .iter()
                    .map(|e| e.symbol.clone())
                    .filter(|s| !s.is_empty())
                    .collect();
                info!("  Total entries: {}", db_entries.len());
                info!("  Unique symbols: {}", unique_symbols.len());
                
                let instrument_types: std::collections::HashMap<String, usize> = db_entries
                    .iter()
                    .fold(std::collections::HashMap::new(), |mut acc, entry| {
                        let type_str = format!("{:?}", entry.instrument_type);
                        *acc.entry(type_str).or_insert(0) += 1;
                        acc
                    });
                
                info!("  Instrument types:");
                for (instrument_type, count) in instrument_types {
                    info!("    {}: {}", instrument_type, count);
                }
                
                let with_expiry = db_entries.iter().filter(|e| !e.expiry.is_empty()).count();
                let without_expiry = db_entries.len() - with_expiry;
                info!("  With expiry date: {}", with_expiry);
                info!("  Without expiry date: {}", without_expiry);
            }
        }
        Err(e) => {
            error!("❌ Failed to fetch vec DB entries: {:?}", e);
            error!("This could be due to:");
            error!("  - Session expired (try re-running)");
            error!("  - Network connectivity issues");
            error!("  - API rate limiting");
            error!("  - Account permissions");
            return Err(e.into());
        }
    }

    info!("\n=== Example completed successfully! ===");
    Ok(())
}