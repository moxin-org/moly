use reqwest::StatusCode;

pub fn enrich_http_error(status: StatusCode, original: &str, body: Option<&str>) -> String {
    let clarification = match status {
        StatusCode::TOO_MANY_REQUESTS => {
            "This usually means you've hit a rate limit, run out of quota/credits, or do not have access to this resource/model in your current plan."
        }
        StatusCode::UNAUTHORIZED => "This usually means your API key is invalid or expired.",
        StatusCode::FORBIDDEN => {
            "This usually means you do not have permission to access this resource."
        }
        StatusCode::BAD_REQUEST => {
            "This might be an error on our side. If the problem persists, please file an issue on GitHub."
        }
        x if x >= StatusCode::INTERNAL_SERVER_ERROR
            && x <= StatusCode::HTTP_VERSION_NOT_SUPPORTED =>
        {
            "A server error occurred. This is likely a temporary issue with the provider."
        }
        _ => "",
    };

    let mut result = original.to_string();

    if let Some(body) = body {
        if !body.trim().is_empty() {
            result.push_str(&format!("\n\nResponse: {}", body));
        }
    }

    if !clarification.is_empty() {
        result.push_str(&format!("\n\nNote: {}", clarification));
    }

    result
}
