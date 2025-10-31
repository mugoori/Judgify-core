/// Simple binary to verify .env loading works
fn main() {
    // Load .env file
    if let Err(e) = dotenvy::from_path("../.env") {
        eprintln!("❌ Failed to load .env file: {}", e);
        std::process::exit(1);
    }

    // Check if OPENAI_API_KEY is set
    match std::env::var("OPENAI_API_KEY") {
        Ok(key) => {
            let masked = if key.len() > 10 {
                format!("{}...{}", &key[..7], &key[key.len()-4..])
            } else {
                "***".to_string()
            };
            println!("✅ OPENAI_API_KEY is set: {}", masked);
        }
        Err(_) => {
            eprintln!("❌ OPENAI_API_KEY is NOT set");
            std::process::exit(1);
        }
    }
}
