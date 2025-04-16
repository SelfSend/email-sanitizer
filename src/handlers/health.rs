use crate::models::HealthResponse;
use actix_web::Responder;

/// # Service Health Check Endpoint
///
/// Provides a liveness probe for the service, indicating whether the API is operational.
///
/// ## Response
///
/// - **200 OK**: Service is running and healthy
///   - Content-Type: `application/json`
///   - Body: [`HealthResponse`] containing:
///     - `status`: String indicating service status ("UP")
///     - `timestamp`: ISO 8601 timestamp of the check
///
/// ## Example Success Response
/// ```json
/// {
///   "status": "UP",
///   "timestamp": "2023-10-05T14:23:45.678Z"
/// }
/// ```
///
/// [`HealthResponse`]: crate::models::HealthResponse
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse::up())
}
