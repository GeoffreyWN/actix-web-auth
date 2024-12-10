use actix_web::{HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::dtos::Response;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

#[derive(Debug)]
pub enum ErrorMessage {
    EmptyPassword,
    ExceededMaxPasswordLength(usize),
    HashingError,
    InvalidHashFormat,
    InvalidToken,
    ServerError,
    WrongCredentials,
    EmailExists,
    UserNoLongerExists,
    TokenNotProvided,
    PermissionDenied,
}

impl Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str().to_owned())
    }
}

impl Into<String> for ErrorMessage {
    fn into(self) -> String {
        self.to_string()
    }
}

impl ErrorMessage {
    fn to_str(&self) -> String {
        match self {
            ErrorMessage::ServerError => "Server Error. Please try again later".to_string(),
            ErrorMessage::WrongCredentials => "Email or password is wrong".to_string(),
            ErrorMessage::EmailExists => "Email already exists".to_string(),
            ErrorMessage::UserNoLongerExists => "User does not exist".to_string(),
            ErrorMessage::EmptyPassword => "Password is empty".to_string(),
            ErrorMessage::HashingError => "Error hashing password".to_string(),
            ErrorMessage::InvalidHashFormat => "Invalid password hash format".to_string(),
            ErrorMessage::ExceededMaxPasswordLength(max_length) => {
                format!("Max password length is {}", max_length)
            }
            ErrorMessage::InvalidToken => "Invalid token".to_string(),
            ErrorMessage::TokenNotProvided => {
                "You are not logged in, please provide a token".to_string()
            }
            ErrorMessage::PermissionDenied => {
                "You are not authorized to perform this action".to_string()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpError {
    pub message: String,
    pub status: u16,
}

impl HttpError {
    pub fn new(message: impl Into<String>, status: u16) -> Self {
        HttpError {
            message: message.into(),
            status,
        }
    }

    pub fn server_error(message: impl Into<String>) -> Self {
        HttpError {
            message: message.into(),
            status: 500,
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        HttpError {
            message: message.into(),
            status: 400,
        }
    }

    pub fn unique_constraint_violation(message: impl Into<String>) -> Self {
        HttpError {
            message: message.into(),
            status: 409,
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        HttpError {
            message: message.into(),
            status: 401,
        }
    }

    pub fn into_http_response(self) -> HttpResponse {
        match self.status {
            400 => HttpResponse::BadRequest().json(Response {
                status: "fail",
                message: self.message.into(),
            }),
            401 => HttpResponse::Unauthorized().json(Response {
                status: "fail",
                message: self.message.into(),
            }),
            409 => HttpResponse::Conflict().json(Response {
                status: "fail",
                message: self.message.into(),
            }),
            500 => HttpResponse::InternalServerError().json(Response {
                status: "fail",
                message: self.message.into(),
            }),
            _ => {
                eprintln!(
                    "Warning: Missing patter match. Converted status code {} for 500",
                    self.status
                );
                HttpResponse::InternalServerError().json(Response {
                    status: "error",
                    message: self.message.into(),
                })
            }
        }
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HttpError: message {}, status: {}",
            self.message, self.status
        )
    }
}

impl std::error::Error for HttpError {}

impl ResponseError for HttpError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        let cloned = self.clone();
        cloned.into_http_response()
    }
}