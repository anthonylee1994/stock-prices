use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Serialize, Serializer};

/// A normalized quote as exposed by `GET /quotes`.
///
/// Every optional field is omitted from the JSON body when absent, matching the
/// way `undefined` properties disappeared from the previous implementation.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent_change: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_price: Option<f64>,
    #[serde(serialize_with = "serialize_timestamp", skip_serializing_if = "Option::is_none")]
    pub regular_market_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_close_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_market_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_market_change: Option<f64>,
    #[serde(serialize_with = "serialize_timestamp", skip_serializing_if = "Option::is_none")]
    pub pre_market_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_market_change_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_market_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_market_change: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_market_change_percent: Option<f64>,
    #[serde(serialize_with = "serialize_timestamp", skip_serializing_if = "Option::is_none")]
    pub post_market_time: Option<DateTime<Utc>>,
    #[serde(rename = "forwardPE", skip_serializing_if = "Option::is_none")]
    pub forward_pe: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_to_book: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dividend_yield: Option<f64>,
}

/// Emits `2026-04-29T09:30:00.000Z`, byte-identical to `JSON.stringify(date)`.
fn serialize_timestamp<S>(value: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(timestamp) => serializer.serialize_str(&timestamp.to_rfc3339_opts(SecondsFormat::Millis, true)),
        None => serializer.serialize_none(),
    }
}
