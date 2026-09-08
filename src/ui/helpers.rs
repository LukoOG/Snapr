use chrono::{DateTime, Utc, Local, TimeZone};

#[inline]
pub fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1 << 10;
    const MB: usize = 1 << 20;
    const GB: usize = 1 << 30;

    if bytes < KB {
        return format!("{bytes} B")
    } else if bytes < MB {
        return format!("{:.2} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        return format!("{:.2} MB", bytes as f64 / MB as f64)
    } else {
        return format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}

#[inline]
pub fn format_timestamp(timestamp_secs: u64) -> String {
    // 1. Convert the u64 to an i64 safely
    let i64_timestamp = timestamp_secs as i64;
    
    // 2. Create a UTC DateTime from the timestamp
    let datetime_utc = Utc.timestamp_opt(i64_timestamp, 0)
        .single()
        .expect("Invalid timestamp");
        
    // 3. Optional: Convert to local time if required for presentation
    let datetime_local: DateTime<Local> = DateTime::from(datetime_utc);

    // 4. Format into your exact layout: "Sep 8, 2026 · 2:14 PM"
    // %b = Abbreviated month name (e.g., Sep)
    // %e = Day of the month, space-padded (e.g.,  8)
    // %Y = Four-digit year (e.g., 2026)
    // %l = Hour, space-padded, 12-hour clock (e.g.,  2)
    // %M = Minute, zero-padded (e.g., 14)
    // %p = AM/PM indicator (e.g., PM)
    datetime_local.format("%b %e, %Y · %l:%M %p").to_string()
}