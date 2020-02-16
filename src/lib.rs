//! Crate `ruma_api` contains core types used to define the requests and responses for each endpoint
//! in the various [Matrix](https://matrix.org) API specifications.
//! These types can be shared by client and server code for all Matrix APIs.
//!
//! When implementing a new Matrix API, each endpoint has a request type which implements
//! `Endpoint`, and a response type connected via an associated type.
//!
//! An implementation of `Endpoint` contains all the information about the HTTP method, the path and
//! input parameters for requests, and the structure of a successful response.
//! Such types can then be used by client code to make requests, and by server code to fulfill
//! those requests.

#![warn(rust_2018_idioms)]
#![deny(missing_copy_implementations, missing_debug_implementations, missing_docs)]

use http::Method;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

/// Generates a `ruma_api::Endpoint` from a concise definition.
///
/// The macro expects the following structure as input:
///
/// ```text
/// ruma_api! {
///     metadata {
///         description: &'static str,
///         method: http::Method,
///         name: &'static str,
///         path: &'static str,
///         rate_limited: bool,
///         requires_authentication: bool,
///     }
///
///     request {
///         // Struct fields for each piece of data required
///         // to make a request to this API endpoint.
///     }
///
///     response {
///         // Struct fields for each piece of data expected
///         // in the response from this API endpoint.
///     }
/// }
/// ```
///
/// This will generate a `ruma_api::Metadata` value to be used for the `ruma_api::Endpoint`'s
/// associated constant, single `Request` and `Response` structs, and the necessary trait
/// implementations to convert the request into a `http::Request` and to create a response from a
/// `http::Response` and vice versa.
///
/// The details of each of the three sections of the macros are documented below.
///
/// ## Metadata
///
/// *   `description`: A short description of what the endpoint does.
/// *   `method`: The HTTP method used for requests to the endpoint.
///     It's not necessary to import `http::Method`'s associated constants. Just write
///     the value as if it was imported, e.g. `GET`.
/// *   `name`: A unique name for the endpoint.
///     Generally this will be the same as the containing module.
/// *   `path`: The path component of the URL for the endpoint, e.g. "/foo/bar".
///     Components of the path that are parameterized can indicate a varible by using a Rust
///     identifier prefixed with a colon, e.g. `/foo/:some_parameter`.
///     A corresponding query string parameter will be expected in the request struct (see below
///     for details).
/// *   `rate_limited`: Whether or not the endpoint enforces rate limiting on requests.
/// *   `requires_authentication`: Whether or not the endpoint requires a valid access token.
///
/// ## Request
///
/// The request block contains normal struct field definitions.
/// Doc comments and attributes are allowed as normal.
/// There are also a few special attributes available to control how the struct is converted into a
/// `http::Request`:
///
/// *   `#[ruma_api(header = HEADER_NAME)]`: Fields with this attribute will be treated as HTTP
///     headers on the request.
///     The value must implement `AsRef<str>`.
///     Generally this is a `String`.
///     The attribute value shown above as `HEADER_NAME` must be a header name constant from
///     `http::header`, e.g. `CONTENT_TYPE`.
/// *   `#[ruma_api(path)]`: Fields with this attribute will be inserted into the matching path
///     component of the request URL.
/// *   `#[ruma_api(query)]`: Fields with this attribute will be inserting into the URL's query
///     string.
/// *   `#[ruma_api(query_map)]`: Instead of individual query fields, one query_map field, of any
///     type that implements `IntoIterator<Item = (String, String)>` (e.g.
///     `HashMap<String, String>`, can be used for cases where an endpoint supports arbitrary query
///     parameters.
///
/// Any field that does not include one of these attributes will be part of the request's JSON
/// body.
///
/// ## Response
///
/// Like the request block, the response block consists of normal struct field definitions.
/// Doc comments and attributes are allowed as normal.
/// There is also a special attribute available to control how the struct is created from a
/// `http::Request`:
///
/// *   `#[ruma_api(header = HEADER_NAME)]`: Fields with this attribute will be treated as HTTP
///     headers on the response.
///     The value must implement `AsRef<str>`.
///     Generally this is a `String`.
///     The attribute value shown above as `HEADER_NAME` must be a header name constant from
///     `http::header`, e.g. `CONTENT_TYPE`.
///
/// Any field that does not include the above attribute will be expected in the response's JSON
/// body.
///
/// ## Newtype bodies
///
/// Both the request and response block also support "newtype bodies" by using the
/// `#[ruma_api(body)]` attribute on a field. If present on a field, the entire request or response
/// body will be treated as the value of the field. This allows you to treat the entire request or
/// response body as a specific type, rather than a JSON object with named fields. Only one field in
/// each struct can be marked with this attribute. It is an error to have a newtype body field and
/// normal body fields within the same struct.
///
/// There is another kind of newtype body that is enabled with `#[ruma_api(raw_body)]`. It is used
/// for endpoints in which the request or response body can be arbitrary bytes instead of a JSON
/// objects. A field with `#[ruma_api(raw_body)]` needs to have the type `Vec<u8>`.
///
/// # Examples
///
/// ```
/// pub mod some_endpoint {
///     use ruma_api_macros::ruma_api;
///
///     ruma_api! {
///         metadata {
///             description: "Does something.",
///             method: POST,
///             name: "some_endpoint",
///             path: "/_matrix/some/endpoint/:baz",
///             rate_limited: false,
///             requires_authentication: false,
///         }
///
///         request {
///             pub foo: String,
///
///             #[ruma_api(header = CONTENT_TYPE)]
///             pub content_type: String,
///
///             #[ruma_api(query)]
///             pub bar: String,
///
///             #[ruma_api(path)]
///             pub baz: String,
///         }
///
///         response {
///             #[ruma_api(header = CONTENT_TYPE)]
///             pub content_type: String,
///
///             pub value: String,
///         }
///     }
/// }
///
/// pub mod newtype_body_endpoint {
///     use ruma_api_macros::ruma_api;
///     use serde::{Deserialize, Serialize};
///
///     #[derive(Clone, Debug, Deserialize, Serialize)]
///     pub struct MyCustomType {
///         pub foo: String,
///     }
///
///     ruma_api! {
///         metadata {
///             description: "Does something.",
///             method: PUT,
///             name: "newtype_body_endpoint",
///             path: "/_matrix/some/newtype/body/endpoint",
///             rate_limited: false,
///             requires_authentication: false,
///         }
///
///         request {
///             #[ruma_api(body)]
///             pub file: Vec<u8>,
///         }
///
///         response {
///             #[ruma_api(body)]
///             pub my_custom_type: MyCustomType,
///         }
///     }
/// }
/// ```
///
/// ## Fallible deserialization
///
/// All request and response types also derive [`Outgoing`][Outgoing]. As such, to allow fallible
/// deserialization, you can use the `#[wrap_incoming]` attribute. For details, see the
/// documentation for [the derive macro](derive.Outgoing.html).
// TODO: Explain the concept of fallible deserialization before jumping to `ruma_api::Outgoing`
#[cfg(feature = "with-ruma-api-macros")]
pub use ruma_api_macros::ruma_api;

#[cfg(feature = "with-ruma-api-macros")]
pub use ruma_api_macros::Outgoing;

pub mod error;
/// This module is used to support the generated code from ruma-api-macros.
/// It is not considered part of ruma-api's public API.
#[cfg(feature = "with-ruma-api-macros")]
#[doc(hidden)]
pub mod exports {
    pub use http;
    pub use percent_encoding;
    pub use serde;
    pub use serde_json;
    pub use serde_urlencoded;
    pub use url;
}

use error::{FromHttpRequestError, FromHttpResponseError, IntoHttpError};

/// A type that can be sent to another party that understands the matrix protocol. If any of the
/// fields of `Self` don't implement serde's `Deserialize`, you can derive this trait to generate a
/// corresponding 'Incoming' type that supports deserialization. This is useful for things like
/// ruma_events' `EventResult` type. For more details, see the [derive macro's documentation][doc].
///
/// [doc]: derive.Outgoing.html
// TODO: Better explain how this trait relates to serde's traits
pub trait Outgoing {
    /// The 'Incoming' variant of `Self`.
    type Incoming;
}

/// A Matrix API endpoint.
///
/// The type implementing this trait contains any data needed to make a request to the endpoint.
pub trait Endpoint: Outgoing + TryInto<http::Request<Vec<u8>>, Error = IntoHttpError>
where
    <Self as Outgoing>::Incoming: TryFrom<http::Request<Vec<u8>>, Error = FromHttpRequestError>,
    <Self::Response as Outgoing>::Incoming:
        TryFrom<http::Response<Vec<u8>>, Error = FromHttpResponseError>,
{
    /// Data returned in a successful response from the endpoint.
    type Response: Outgoing + TryInto<http::Response<Vec<u8>>, Error = IntoHttpError>;

    /// Metadata about the endpoint.
    const METADATA: Metadata;
}

/// Metadata about an API endpoint.
#[derive(Clone, Debug)]
pub struct Metadata {
    /// A human-readable description of the endpoint.
    pub description: &'static str,

    /// The HTTP method used by this endpoint.
    pub method: Method,

    /// A unique identifier for this endpoint.
    pub name: &'static str,

    /// The path of this endpoint's URL, with variable names where path parameters should be filled
    /// in during a request.
    pub path: &'static str,

    /// Whether or not this endpoint is rate limited by the server.
    pub rate_limited: bool,

    /// Whether or not the server requires an authenticated user for this endpoint.
    pub requires_authentication: bool,
}

#[serde(tag = "errcode")]
#[derive(Debug, Clone, Serialize, Deserialize)]
enum MatrixErrorKind {
    #[serde(rename = "M_FORBIDDEN")]
    Forbidden,
    #[serde(rename = "M_UNKNOWN_TOKEN")]
    UnknownToken,
    #[serde(rename = "M_MISSING_TOKEN")]
    MissingToken,
    #[serde(rename = "M_BAD_JSON")]
    BadJson,
    #[serde(rename = "M_NOT_JSON")]
    NotJson,
    #[serde(rename = "M_NOT_FOUND")]
    NotFound,
    #[serde(rename = "M_LIMIT_EXCEEDED")]
    LimitExceeded,
    #[serde(rename = "M_UNKNOWN")]
    Unknown,
    #[serde(rename = "M_UNRECOGNIZED")]
    Unrecognized,
    #[serde(rename = "M_UNAUTHORIZED")]
    Unauthorized,
    #[serde(rename = "M_USER_IN_USE")]
    UserInUse,
    #[serde(rename = "M_INVALID_USERNAME")]
    InvalidUsername,
    #[serde(rename = "M_ROOM_IN_USE")]
    RoomInUse,
    #[serde(rename = "M_INVALID_ROOM_STATE")]
    InvalidRoomState,
    #[serde(rename = "M_THREEPID_IN_USE")]
    ThreepidInUse,
    #[serde(rename = "M_THREEPID_NOT_FOUND")]
    ThreepidNotFound,
    #[serde(rename = "M_THREEPID_AUTH_FAILED")]
    ThreepidAuthFailed,
    #[serde(rename = "M_THREEPID_DENIED")]
    ThreepidDenied,
    #[serde(rename = "M_SERVER_NOT_TRUSTED")]
    ServerNotTrusted,
    #[serde(rename = "M_UNSUPPORTED_ROOM_VERSION")]
    UnsupportedRoomVersion,
    #[serde(rename = "M_INCOMPATIBLE_ROOM_VERSION")]
    IncompatibleRoomVersion,
    #[serde(rename = "M_BAD_STATE")]
    BadState,
    #[serde(rename = "M_GUEST_ACCESS_FORBIDDEN")]
    GuestAccessForbidden,
    #[serde(rename = "M_CAPTCHA_NEEDED")]
    CaptchaNeeded,
    #[serde(rename = "M_CAPTCHA_INVALID")]
    CaptchaInvalid,
    #[serde(rename = "M_MISSING_PARAM")]
    MissingParam,
    #[serde(rename = "M_INVALID_PARAM")]
    InvalidParam,
    #[serde(rename = "M_TOO_LARGE")]
    TooLarge,
    #[serde(rename = "M_EXCLUSIVE")]
    Exclusive,
}

/// A Matrix Error
///
/// The type implementing this trait contains any data needed to construct a matrix error.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MatrixError {
    /// A value which can be used to handle an error message
    #[serde(flatten)]
    kind: MatrixErrorKind,
    /// The http status code
    #[serde(skip)]
    status_code: http::StatusCode,
    /// A human-readable error message, usually a sentence explaining what went wrong.
    #[serde(rename = "error")]
    message: String,
}

impl From<MatrixError> for http::Response<Vec<u8>> {
    fn from(error: MatrixError) -> http::Response<Vec<u8>> {
        http::Response::builder()
            .header(http::header::CONTENT_TYPE, "application/json")
            .status(error.status_code)
            .body(serde_json::to_vec(&error).unwrap())
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    /// PUT /_matrix/client/r0/directory/room/:room_alias
    pub mod create {
        use std::{convert::TryFrom, ops::Deref};

        use http::{header::CONTENT_TYPE, method::Method};
        use ruma_identifiers::{RoomAliasId, RoomId};
        use serde::{Deserialize, Serialize};

        use crate::{
            error::{
                FromHttpRequestError, FromHttpResponseError, IntoHttpError,
                RequestDeserializationError, ServerError,
            },
            Endpoint, Metadata, Outgoing,
        };

        /// A request to create a new room alias.
        #[derive(Debug)]
        pub struct Request {
            pub room_id: RoomId,         // body
            pub room_alias: RoomAliasId, // path
        }

        impl Outgoing for Request {
            type Incoming = Self;
        }

        impl Endpoint for Request {
            type Response = Response;

            const METADATA: Metadata = Metadata {
                description: "Add an alias to a room.",
                method: Method::PUT,
                name: "create_alias",
                path: "/_matrix/client/r0/directory/room/:room_alias",
                rate_limited: false,
                requires_authentication: true,
            };
        }

        impl TryFrom<Request> for http::Request<Vec<u8>> {
            type Error = IntoHttpError;

            fn try_from(request: Request) -> Result<http::Request<Vec<u8>>, Self::Error> {
                let metadata = Request::METADATA;

                let path = metadata
                    .path
                    .to_string()
                    .replace(":room_alias", &request.room_alias.to_string());

                let request_body = RequestBody { room_id: request.room_id };

                let http_request = http::Request::builder()
                    .method(metadata.method)
                    .uri(path)
                    .body(serde_json::to_vec(&request_body)?)
                    .expect("http request building to succeed");

                Ok(http_request)
            }
        }

        impl TryFrom<http::Request<Vec<u8>>> for Request {
            type Error = FromHttpRequestError;

            fn try_from(request: http::Request<Vec<u8>>) -> Result<Self, Self::Error> {
                let request_body: RequestBody =
                    match serde_json::from_slice(request.body().as_slice()) {
                        Ok(body) => body,
                        Err(err) => {
                            return Err(RequestDeserializationError::new(err, request).into());
                        }
                    };
                let path_segments: Vec<&str> = request.uri().path()[1..].split('/').collect();
                Ok(Request {
                    room_id: request_body.room_id,
                    room_alias: {
                        let segment = path_segments.get(5).unwrap().as_bytes();
                        let decoded = percent_encoding::percent_decode(segment).decode_utf8_lossy();
                        match serde_json::from_str(decoded.deref()) {
                            Ok(id) => id,
                            Err(err) => {
                                return Err(RequestDeserializationError::new(err, request).into())
                            }
                        }
                    },
                })
            }
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct RequestBody {
            room_id: RoomId,
        }

        /// The response to a request to create a new room alias.
        #[derive(Clone, Copy, Debug)]
        pub struct Response;

        impl Outgoing for Response {
            type Incoming = Self;
        }

        impl TryFrom<http::Response<Vec<u8>>> for Response {
            type Error = FromHttpResponseError;

            fn try_from(http_response: http::Response<Vec<u8>>) -> Result<Response, Self::Error> {
                if http_response.status().as_u16() < 400 {
                    Ok(Response)
                } else {
                    Err(FromHttpResponseError::Http(ServerError::new(http_response)))
                }
            }
        }

        impl TryFrom<Response> for http::Response<Vec<u8>> {
            type Error = IntoHttpError;

            fn try_from(_: Response) -> Result<http::Response<Vec<u8>>, Self::Error> {
                let response = http::Response::builder()
                    .header(CONTENT_TYPE, "application/json")
                    .body(b"{}".to_vec())
                    .unwrap();

                Ok(response)
            }
        }
    }
}
