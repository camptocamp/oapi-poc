use std::env::VarError;

use axum::{
    body::BoxBody,
    headers::{
        authorization::{Basic, Credentials},
        Authorization,
    },
    http::{header::AUTHORIZATION, Method, Request, Response, StatusCode},
    response::IntoResponse,
};
use once_cell::sync::OnceCell;
use tower_http::auth::AuthorizeRequest;

use ogcapi_services::Error;

static BASIC: OnceCell<Basic> = OnceCell::new();

#[derive(Clone, Copy)]
pub(crate) struct Auth;

impl<B> AuthorizeRequest<B> for Auth {
    type ResponseBody = BoxBody;

    fn authorize(&mut self, request: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>> {
        match *request.method() {
            // Do not reqire authorization for GET and HEAD requests
            Method::GET | Method::HEAD => Ok(()),
            // Reqire basic authroization for everything else
            _ => {
                // Exempt STAC /search
                if request.uri() == "/search" {
                    return Ok(());
                }
                if let Some(auth) = request.headers().get(AUTHORIZATION) {
                    let basic = BASIC.get_or_try_init(|| -> Result<Basic, VarError> {
                        Ok(Authorization::basic(
                            &std::env::var("APP_USER")?,
                            &std::env::var("APP_PASSWORD")?,
                        )
                        .0)
                    });

                    match basic {
                        Ok(basic) => {
                            if Some(basic) == Credentials::decode(auth).as_ref() {
                                Ok(())
                            } else {
                                Err(Error::Exception(
                                    StatusCode::UNAUTHORIZED,
                                    "Invalid credentials".to_string(),
                                )
                                .into_response())
                            }
                        }
                        Err(_) => Err(Error::Exception(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Credentials must be set".to_string(),
                        )
                        .into_response()),
                    }
                } else {
                    Err(Error::Exception(
                        StatusCode::UNAUTHORIZED,
                        "Basic authorization required".to_string(),
                    )
                    .into_response())
                }
            }
        }
    }
}
