use reqwest::StatusCode;

pub fn enrich_http_error(status: StatusCode, original: &str) -> String {
    let clarification = match status {
        StatusCode::TOO_MANY_REQUESTS => {
            "This usually means you've hit a rate limit, run out of quota/credits, or do not have access to this resource/model in your current plan."
        }
        StatusCode::UNAUTHORIZED => "This usually means your API key is invalid or expired.",
        StatusCode::FORBIDDEN => {
            "This usually means you do not have permission to access this resource."
        }
        StatusCode::BAD_REQUEST => {
            "This looks like an error on our side. Please file an issue on GitHub."
        }
        x if x >= StatusCode::INTERNAL_SERVER_ERROR
            && x <= StatusCode::HTTP_VERSION_NOT_SUPPORTED =>
        {
            "A server error occurred. This is likely a temporary issue with the provider."
        }
        _ => "",
    };

    if clarification.is_empty() {
        original.to_string()
    } else {
        format!("{original}\n\nNote: {clarification}")
    }
}
