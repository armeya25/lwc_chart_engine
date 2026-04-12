use once_cell::sync::Lazy;
use crate::types::IndicatorType;
use serde_json::json;

static INDICATOR_SCHEMAS: Lazy<serde_json::Value> = Lazy::new(|| {
    let toml_str = include_str!("indicators.toml");
    let toml_val: toml::Value = toml::from_str(toml_str).expect("Failed to parse indicators.toml");
    serde_json::to_value(&toml_val).expect("Failed to convert indicator TOML to JSON")
});

pub fn get_indicator_schemas() -> String {
    INDICATOR_SCHEMAS.to_string()
}

pub fn get_indicator_params_schema(ind_type: IndicatorType) -> serde_json::Value {
    let all_json = get_indicator_schemas();
    let all: serde_json::Value = serde_json::from_str(&all_json).unwrap();
    let key = match ind_type {
        IndicatorType::Sma => "sma",
        IndicatorType::Ema => "ema",
        IndicatorType::Rsi => "rsi",
        IndicatorType::Macd => "macd",
        IndicatorType::BollingerBands => "bollingerbands",
        IndicatorType::Atr => "atr",
        IndicatorType::Stochastic => "stochastic",
        IndicatorType::Cci => "cci",
        IndicatorType::Vwap => "vwap",
        IndicatorType::WilliamsR => "williamsr",
        IndicatorType::Dema => "dema",
        IndicatorType::Tema => "tema",
        IndicatorType::Wma => "wma",
        IndicatorType::Hma => "hma",
        IndicatorType::Mfi => "mfi",
        IndicatorType::Roc => "roc",
        IndicatorType::KeltnerChannels => "keltnerchannels",
        IndicatorType::DonchianChannels => "donchianchannels",
        IndicatorType::Obv => "obv",
        IndicatorType::Adl => "adl",
    };
    all.get(key).and_then(|v| v.get("params")).cloned().unwrap_or(json!({}))
}

pub fn get_sub_series_info(ind_type: IndicatorType) -> Vec<(&'static str, &'static str, &'static str)> {
    match ind_type {
        IndicatorType::Macd => vec![
            ("main", "MACD", "#2196F3"),
            ("signal", "Signal", "#FF5252"),
            ("hist", "Histogram", "rgba(31, 150, 243, 0.5)")
        ],
        IndicatorType::BollingerBands => vec![
            ("main", "Mid", "rgba(31, 150, 243, 0.4)"),
            ("upper", "Upper", "rgba(31, 150, 243, 0.4)"),
            ("lower", "Lower", "rgba(31, 150, 243, 0.4)")
        ],
        IndicatorType::KeltnerChannels | IndicatorType::DonchianChannels => vec![
            ("main", "Mid", "rgba(156, 39, 176, 0.4)"),
            ("upper", "Upper", "rgba(156, 39, 176, 0.4)"),
            ("lower", "Lower", "rgba(156, 39, 176, 0.4)")
        ],
        IndicatorType::Stochastic => vec![
            ("main", "%K", "#1565C0"),
            ("d", "%D", "#E53935")
        ],
        _ => vec![("main", "Value", "#2962FF")],
    }
}

pub fn is_oscillator(ind_type: IndicatorType) -> bool {
    match ind_type {
        IndicatorType::Rsi | IndicatorType::Macd | IndicatorType::Atr | 
        IndicatorType::Stochastic | IndicatorType::Cci | IndicatorType::WilliamsR |
        IndicatorType::Mfi | IndicatorType::Roc | IndicatorType::Obv | IndicatorType::Adl => true,
        _ => false,
    }
}
