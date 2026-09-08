use chrono::{DateTime, Utc, Local, TimeZone};

#[inline]
pub fn format_bytes(bytes: f64) -> String {
    const KB: f64 = 1000.0;//1 << 10;
    const MB: f64 = 1_000_000.0;
    const GB: f64 = 1_000_000_000.0;

    if bytes < KB {
        return format!("{bytes} B")
    } else if bytes < MB {
        return format!("{:.2} KB", bytes / KB)
    } else if bytes < GB {
        return format!("{:.2} MB", bytes / MB)
    } else {
        return format!("{:.2} GB", bytes / GB)
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

    // 4. Format into a concise history layout: "Sep 8, 2:14 PM"
    // %b = Abbreviated month name (e.g., Sep)
    // %-d = Day of the month without padding (e.g., 8)
    // %-I = 12-hour clock without padding (e.g., 2)
    // %M = Minute, zero-padded (e.g., 14)
    // %p = AM/PM indicator (e.g., PM)
    datetime_local.format("%b %-d, %-I:%M %p").to_string()
}
