use std::collections::HashMap;

use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use lettre::transport::smtp;
use serde::{Deserialize, Serialize};


#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),  // 400 Bad Request

    #[error("Invalid input: {0}")]
    Invalid(String),  // 400 Bad Request

    #[error("Permission denied: {0}")]
    PermissionDenied(String),  // 403 Forbidden

    #[error("Not found: {0}")]
    NotFound(String),  // 404 Not Found

    #[error("Conflict: {0}")]
    Conflict(String),  // 409 Conflict

    #[error("Expired: {0}")]
    Expired(String),  // 410 Gone

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),  // 500 Internal Server Error

    #[error("SMTP error: {0}")]
    Smtp(#[from] smtp::Error),  // 500 Internal Server Error

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),  // 500 Internal Server Error
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
            AppError::Validation(errs) => {
                let mut map = HashMap::new();
                for (field, errors) in errs.field_errors() {
                    let messages = errors.iter()
                        .map(|e| e.code.to_string())
                        .collect();
                    map.insert(field.to_string(), messages);
                }
                (StatusCode::BAD_REQUEST, "Validation failed".into(), Some(map))
            }

            AppError::Invalid(err) => (StatusCode::BAD_REQUEST, err, None),
            AppError::PermissionDenied(err) => (StatusCode::FORBIDDEN, err, None),
            AppError::NotFound(err) => (StatusCode::NOT_FOUND, err, None),
            AppError::Expired(err) => (StatusCode::GONE, err, None),
            AppError::Conflict(field) => {
                let mut map = HashMap::new();
                map.insert(field.clone(), vec![format!("{} already exists", field)]);

                (StatusCode::CONFLICT, "Resource conflict".into(), Some(map))
            },

            AppError::Database(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", err), None),
            AppError::Smtp(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("SMTP error: {}", err), None),
            AppError::Internal(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error: {}", err), None),
        };

        Json(ApiErrorResponse {
            status: status.as_u16(),
            message,
            errors: field_errors,
        }).into_response()
    }
}


