use std::collections::HashMap;

use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use serde::{Deserialize, Serialize};


#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),  // 400 Bad Request

    #[error("Permission denied: {0}")]
    PermissionDenied(String),  // 403 Forbidden

    #[error("Not found: {0}")]
    NotFound(String),  // 404 Not Found

    #[error("Conflict: {0}")]
    Conflict(String),  // 409 Conflict

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),  // 500 Internal Server Error

    #[error("Internal server error: {0}")]
    InternalServerError(#[from] anyhow::Error),  // 500 Internal Server Error
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub status: u16,  // StatusCode as u16
    pub message: String,
    pub errors: Option<HashMap<String, Vec<String>>>,
}


impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, field_errors) = match self {
            AppError::ValidationError(errs) => {
                let mut map = HashMap::new();
                for (field, errors) in errs.field_errors() {
                    let messages = errors.iter()
                        .map(|e| e.code.to_string())
                        .collect();
                    map.insert(field.to_string(), messages);
                }
                (StatusCode::BAD_REQUEST, "Validation failed".into(), Some(map))
            }

            AppError::PermissionDenied(err) => (StatusCode::FORBIDDEN, err, None),
            AppError::NotFound(err) => (StatusCode::NOT_FOUND, err, None),

            AppError::Conflict(field) => {
                let mut map = HashMap::new();
                map.insert(field.clone(), vec![format!("{} already exists", field)]);

                (StatusCode::CONFLICT, "Resource conflict".into(), Some(map))
            },

            AppError::DatabaseError(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", err), None),
            AppError::InternalServerError(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error: {}", err), None),
        };

        Json(ApiErrorResponse {
            status: status.as_u16(),
            message,
            errors: field_errors,
        }).into_response()
    }
}


