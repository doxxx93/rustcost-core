use tracing::info;

/// Runs only when in RUSTCOST_DEBUG_MODE
pub async fn run_debug() {
    info!("🔧 Debug mode: running debug tasks...");

    // Example: load unit prices and print them
    // let prices = ...;
    // dbg!(prices);

    info!("Debug tasks completed. Exiting...");
}