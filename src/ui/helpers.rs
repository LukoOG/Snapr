#[inline]
pub fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1 << 10;
    const MB: usize = 1 << 20;
    const GB: usize = 1 << 30;

    if bytes < KB {
        format!("{bytes} B")
    } else if bytes < MB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}
