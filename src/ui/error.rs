pub fn print_error(error: Box<dyn std::error::Error + Send + Sync>) {
    eprintln!("✗ {}", error);
}