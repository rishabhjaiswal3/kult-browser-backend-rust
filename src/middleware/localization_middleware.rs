use axum::{
    body::{to_bytes, Body},
    extract::Request,
    http::{
        header::{ACCEPT_LANGUAGE, CONTENT_LENGTH, CONTENT_TYPE, VARY},
        HeaderMap, HeaderValue, StatusCode,
    },
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Map, Value};

pub const DEFAULT_LOCALE: &str = "en";
const JSON_CONTENT_TYPE: &str = "application/json";
const LOCALE_OVERRIDE_HEADERS: [&str; 2] = ["x-locale", "x-language"];
const VARY_HEADERS: &str = "Accept-Language, X-Locale, X-Language";

#[derive(Debug, Clone)]
pub struct RequestLocale {
    requested: Option<String>,
}

impl RequestLocale {
    pub fn new(requested: Option<String>) -> Self {
        Self { requested }
    }

    pub fn requested_or_default(&self) -> &str {
        self.requested.as_deref().unwrap_or(DEFAULT_LOCALE)
    }
}

pub async fn localize_response(mut request: Request, next: Next) -> Response {
    let request_locale = RequestLocale::new(requested_locale_from_headers(request.headers()));
    request.extensions_mut().insert(request_locale.clone());

    let mut response = next.run(request).await;
    append_vary_header(response.headers_mut());

    if !is_json_response(&response) {
        return response;
    }

    let locale = request_locale.requested_or_default().to_string();
    let (parts, body) = response.into_parts();

    let body_bytes = match to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::error!(error = %error, "Failed to read JSON response body for localization");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "ok": false,
                    "message": "Failed to prepare localized response"
                })),
            )
                .into_response();
        }
    };

    let payload = match serde_json::from_slice::<Value>(&body_bytes) {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!(error = %error, "Skipping localization for non-JSON body");
            return Response::from_parts(parts, Body::from(body_bytes));
        }
    };

    let localized_payload = localize_json(payload, &locale);

    match serde_json::to_vec(&localized_payload) {
        Ok(serialized) => {
            let mut response = Response::from_parts(parts, Body::from(serialized));
            response.headers_mut().remove(CONTENT_LENGTH);
            response
        }
        Err(error) => {
            tracing::error!(error = %error, "Failed to serialize localized response body");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "ok": false,
                    "message": "Failed to serialize localized response"
                })),
            )
                .into_response()
        }
    }
}

fn requested_locale_from_headers(headers: &HeaderMap) -> Option<String> {
    for header_name in LOCALE_OVERRIDE_HEADERS {
        if let Some(locale) = headers
            .get(header_name)
            .and_then(|value| value.to_str().ok())
            .and_then(normalize_locale)
        {
            return Some(locale);
        }
    }

    headers
        .get(ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok())
        .and_then(|header| {
            header
                .split(',')
                .find_map(|entry| entry.split(';').next().and_then(normalize_locale))
        })
}

fn normalize_locale(locale: &str) -> Option<String> {
    let normalized = locale.trim().replace('_', "-").to_ascii_lowercase();

    if normalized.is_empty() || normalized == "*" {
        return None;
    }

    if normalized
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    {
        Some(normalized)
    } else {
        None
    }
}

fn append_vary_header(headers: &mut HeaderMap) {
    match headers.get(VARY).and_then(|value| value.to_str().ok()) {
        Some(existing) => {
            let mut vary_values: Vec<String> = existing
                .split(',')
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect();

            for header_name in ["Accept-Language", "X-Locale", "X-Language"] {
                if !vary_values
                    .iter()
                    .any(|value| value.eq_ignore_ascii_case(header_name))
                {
                    vary_values.push(header_name.to_string());
                }
            }

            let combined = vary_values.join(", ");
            if let Ok(value) = HeaderValue::from_str(&combined) {
                headers.insert(VARY, value);
            }
        }
        None => {
            headers.insert(VARY, HeaderValue::from_static(VARY_HEADERS));
        }
    }
}

fn is_json_response(response: &Response) -> bool {
    response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.starts_with(JSON_CONTENT_TYPE))
        .unwrap_or(false)
}

fn localize_json(value: Value, requested_locale: &str) -> Value {
    match value {
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(|item| localize_json(item, requested_locale))
                .collect(),
        ),
        Value::Object(map) => {
            if let Some(selected) = select_localized_variant(&map, requested_locale) {
                return localize_json(selected, requested_locale);
            }

            let localized_map = map
                .into_iter()
                .map(|(key, value)| (key, localize_json(value, requested_locale)))
                .collect::<Map<String, Value>>();

            Value::Object(localized_map)
        }
        other => other,
    }
}

fn select_localized_variant(map: &Map<String, Value>, requested_locale: &str) -> Option<Value> {
    let default_locale = map.get("default")?.as_str().and_then(normalize_locale)?;
    let variant_keys: Vec<&String> = map.keys().filter(|key| key.as_str() != "default").collect();

    if variant_keys.is_empty() || !variant_keys.iter().all(|key| looks_like_locale_key(key)) {
        return None;
    }

    for candidate in locale_candidates(requested_locale, &default_locale) {
        if let Some(value) = map.get(&candidate) {
            return Some(value.clone());
        }
    }

    variant_keys
        .into_iter()
        .find_map(|key| map.get(key).cloned())
}

fn locale_candidates(requested_locale: &str, default_locale: &str) -> Vec<String> {
    let mut candidates = Vec::new();

    push_locale_candidate(&mut candidates, normalize_locale(requested_locale));
    push_locale_candidate(
        &mut candidates,
        normalize_locale(requested_locale)
            .as_deref()
            .and_then(primary_locale),
    );
    push_locale_candidate(&mut candidates, Some(DEFAULT_LOCALE.to_string()));
    push_locale_candidate(&mut candidates, Some(default_locale.to_string()));
    push_locale_candidate(&mut candidates, primary_locale(default_locale));

    candidates
}

fn push_locale_candidate(candidates: &mut Vec<String>, candidate: Option<String>) {
    if let Some(candidate) = candidate {
        if !candidates.iter().any(|value| value == &candidate) {
            candidates.push(candidate);
        }
    }
}

fn primary_locale(locale: &str) -> Option<String> {
    locale.split('-').next().and_then(normalize_locale)
}

fn looks_like_locale_key(key: &str) -> bool {
    let normalized = match normalize_locale(key) {
        Some(value) => value,
        None => return false,
    };

    let mut parts = normalized.split('-');
    let Some(language) = parts.next() else {
        return false;
    };

    if language.len() != 2 || !language.chars().all(|ch| ch.is_ascii_lowercase()) {
        return false;
    }

    parts.all(|part| {
        (part.len() == 2 && part.chars().all(|ch| ch.is_ascii_alphabetic()))
            || (part.len() == 3 && part.chars().all(|ch| ch.is_ascii_digit()))
    })
}

#[cfg(test)]
mod tests {
    use super::{localize_json, requested_locale_from_headers, RequestLocale, DEFAULT_LOCALE};
    use axum::http::{header::ACCEPT_LANGUAGE, HeaderMap, HeaderValue};
    use serde_json::json;

    #[test]
    fn defaults_to_english_when_no_header_is_present() {
        let locale = RequestLocale::new(requested_locale_from_headers(&HeaderMap::new()));

        assert_eq!(locale.requested_or_default(), DEFAULT_LOCALE);
    }

    #[test]
    fn reads_accept_language_and_selects_requested_variant() {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"),
        );

        let payload = json!({
            "ok": true,
            "data": {
                "name": {
                    "default": "en",
                    "en": "Formula Speed Thrills",
                    "zh": "极速快感"
                }
            }
        });

        let locale = requested_locale_from_headers(&headers).expect("locale should exist");
        let localized = localize_json(payload, &locale);

        assert_eq!(
            localized,
            json!({
                "ok": true,
                "data": {
                    "name": "极速快感"
                }
            })
        );
    }

    #[test]
    fn falls_back_to_english_when_requested_variant_is_missing() {
        let payload = json!({
            "name": {
                "default": "en",
                "en": "Zero G Pool"
            }
        });

        let localized = localize_json(payload, "fr");

        assert_eq!(
            localized,
            json!({
                "name": "Zero G Pool"
            })
        );
    }

    #[test]
    fn localizes_nested_image_alt_fields_after_variant_selection() {
        let payload = json!({
            "thumbnail": {
                "horizontal": {
                    "default": "en",
                    "en": {
                        "url": "https://example.com/en.png",
                        "alt": {
                            "default": "en",
                            "en": "English alt",
                            "zh": "中文 alt"
                        }
                    },
                    "zh": {
                        "url": "https://example.com/zh.png",
                        "alt": {
                            "default": "en",
                            "en": "English alt",
                            "zh": "中文 alt"
                        }
                    }
                }
            }
        });

        let localized = localize_json(payload, "zh");

        assert_eq!(
            localized,
            json!({
                "thumbnail": {
                    "horizontal": {
                        "url": "https://example.com/zh.png",
                        "alt": "中文 alt"
                    }
                }
            })
        );
    }

    #[test]
    fn ignores_non_localized_objects_that_only_happen_to_have_default_key() {
        let payload = json!({
            "metadata": {
                "default": "standard",
                "url": "https://example.com"
            }
        });

        let localized = localize_json(payload.clone(), "zh");

        assert_eq!(localized, payload);
    }
}
