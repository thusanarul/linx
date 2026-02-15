pub fn i64_from_string<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let s = String::deserialize(deserializer)?;
    Ok(s.parse::<i64>().ok())
}

pub fn sole_from_string<'de, D>(deserializer: D) -> Result<crate::Sole, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let s = String::deserialize(deserializer)?;
    Ok(crate::Sole(
        s.parse::<i64>().map_err(serde::de::Error::custom)?,
    ))
}

pub fn naivedate_from_string<'de, D>(deserializer: D) -> Result<chrono::NaiveDate, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let s = String::deserialize(deserializer)?;

    Ok(chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)?)
}

pub fn naivetime_from_string<'de, D>(deserializer: D) -> Result<chrono::NaiveTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let s = String::deserialize(deserializer)?;

    Ok(chrono::NaiveTime::parse_from_str(&s, "%H:%M").map_err(serde::de::Error::custom)?)
}
