use chrono::Utc;

// 2012-08-06 05:17:00 UTC
const CURIOSTY_LANDING_DATE_IN_UNIX_TS: i64 = 1344230220;

/// Calculated no of Martian sols elapsed since Curiosity landing date
pub fn calculate_no_of_martian_sol_elapsed(datetime: chrono::DateTime<Utc>) -> i64 {
    // formula: ⌈(Δ • 86400 / 88775.245)⌉ where Δ is diff between date and Curiosity landing date in days
    let diff: f64 = (datetime.timestamp() - CURIOSTY_LANDING_DATE_IN_UNIX_TS) as f64;
    return (diff / 88775.245).ceil() as i64;
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use super::*;

    #[test]
    fn test_martian_sol_formula() {
        // One Martian year is 668.6 sols, approx. 687 Earth days.
        let ts = 687 * 86_400;
        let date = chrono::DateTime::from_timestamp(CURIOSTY_LANDING_DATE_IN_UNIX_TS + ts, 0)
            .expect("Failed to create Datetime");

        let martian_sols = calculate_no_of_martian_sol_elapsed(date);

        // Should be 669 because of ceil func
        assert_eq!(martian_sols, 669);

        // https://rust-lang-nursery.github.io/rust-cookbook/datetime/parse.html#parse-string-into-datetime-struct
        let date = chrono::DateTime::parse_from_rfc3339("2026-02-10T00:00:00+01:00")
            .expect("Failed to parse date")
            .to_utc();

        let martian_sols = calculate_no_of_martian_sol_elapsed(date);

        // Newest API response
        assert_eq!(martian_sols, 4804);
    }
}
