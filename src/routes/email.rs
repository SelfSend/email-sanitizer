use crate::handlers::validation::{disposable, dnsmx, syntax};
use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct EmailRequest {
    email: String,
}

/// # Email Validation Endpoint
///
/// Validates an email address by checking three aspects:
/// 1. RFC-compliant syntax validation
/// 2. Domain DNS/MX record verification
/// 3. Disposable email domain check
///
/// ## Request
/// - Method: POST
/// - Body: JSON object with `email` field
///
/// ## Responses
/// - **200 OK**: Email is valid
/// - **400 Bad Request**:
///   - Invalid email syntax
///   - Domain has no valid MX/A/AAAA records
///   - Disposable email detected
/// - **500 Internal Server Error**: Database connection failed
///
/// ## Example Request
/// ```json
/// { "email": "user@example.com" }
/// ```
pub async fn validate_email(
    req: web::Json<EmailRequest>,
) -> Result<impl Responder, actix_web::Error> {
    let email = req.email.trim();

    // 1. Syntax validation
    if !syntax::is_valid_email(email) {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "INVALID_SYNTAX",
            "message": "Email address has invalid syntax"
        })));
    }

    // 2. DNS/MX validation (blocking task)
    let email_clone = email.to_owned();
    let dns_valid = web::block(move || dnsmx::validate_email_dns(&email_clone))
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("DNS validation error: {}", e))
        })?;

    if !dns_valid {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "INVALID_DOMAIN",
            "message": "Email domain has no valid DNS records"
        })));
    }

    // 3. Disposable email check
    match disposable::is_disposable_email(email).await {
        Ok(true) => Ok(HttpResponse::BadRequest().json(json!({
            "error": "DISPOSABLE_EMAIL",
            "message": "The email address domain is a provider of disposable email addresses"
        }))),
        Ok(false) => Ok(HttpResponse::Ok().json(json!({
            "status": "VALID",
            "message": "Email address is valid"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "error": "DATABASE_ERROR",
            "message": e.to_string()
        }))),
    }
}

/// Configures email validation routes under /api/v1
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/validate-email").route(web::post().to(validate_email)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test};
    use serde_json::json;

    #[actix_web::test]
    async fn test_valid_email() {
        let app = test::init_service(App::new().configure(configure_routes)).await;
        let req = test::TestRequest::post()
            .uri("/validate-email")
            .set_json(json!({ "email": "test@example.com" }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_invalid_syntax() {
        let app = test::init_service(App::new().configure(configure_routes)).await;
        let req = test::TestRequest::post()
            .uri("/validate-email")
            .set_json(json!({ "email": "invalid-email" }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 400);
    }
}
