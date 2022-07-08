use actix_web::{http::header::HeaderMap, Error};
use crypto::{hmac::Hmac, mac::Mac, sha1::Sha1};

pub trait InvalidItems<T> {
    fn invalid_auth(self) -> Result<T, Error>;

    fn invalid_header(self) -> Result<T, Error>;
}

impl<T> InvalidItems<T> for Option<T> {
    fn invalid_auth(self) -> Result<T, Error> {
        self.ok_or(Error::from(actix_web::error::ErrorBadRequest(
            "Invalid authorization header",
        )))
    }

    fn invalid_header(self) -> Result<T, Error> {
        self.ok_or(Error::from(actix_web::error::ErrorBadRequest(
            "Invalid header",
        )))
    }
}

impl<T, E: core::fmt::Debug> InvalidItems<T> for Result<T, E> {
    fn invalid_auth(self) -> Result<T, Error> {
        self.or_else(|_| {
            Err(Error::from(actix_web::error::ErrorBadRequest(
                "Invalid authorization header",
            )))
        })
    }

    fn invalid_header(self) -> Result<T, Error> {
        self.or_else(|_| {
            Err(Error::from(actix_web::error::ErrorBadRequest(
                "Invalid header",
            )))
        })
    }
}

pub struct AuthData {
    pub email: String,
    pub token: String,
}

/// Parse the authorization header and return the email and token.
///
/// The authorization header is in the format of:
/// `"Basic {base64(email:player_token)}"`
pub fn safe_basic_auth_decoder(auth_header: &str) -> Result<AuthData, Error> {
    // split auth header from "Basic {auth}"
    let auth_header = auth_header.split_whitespace().collect::<Vec<_>>();

    // check if auth header is "Basic"
    if *auth_header.get(0).invalid_auth()? != "Basic" {
        return Err(Error::from(actix_web::error::ErrorBadRequest(
            "Invalid authorization header",
        )));
    }

    // obtain the base64 string from the header
    let auth_header = auth_header.get(1).invalid_auth()?;
    // decode base64 from the string
    let auth_header = base64::decode(auth_header).invalid_auth()?;

    // convert the bytes to a string
    let auth_header = String::from_utf8(auth_header).invalid_auth()?;

    println!("testing basic auth header decoding: {}", auth_header);

    // get the username and password from the email:player_token
    let auth_header = auth_header.split(":").collect::<Vec<_>>();
    let player_email = auth_header.get(0).invalid_auth()?;
    let player_token = auth_header.get(1).invalid_auth()?;

    println!("email: {}, token: {}", player_email, player_token);

    Ok(AuthData {
        email: player_email.to_string(),
        token: player_token.to_string(),
    })
}

impl From<&str> for AuthData {
    fn from(auth_header: &str) -> Self {
        let auth_header = safe_basic_auth_decoder(auth_header).unwrap();
        auth_header
    }
}

impl From<HeaderMap> for AuthData {
    fn from(headers: HeaderMap) -> Self {
        let auth_header = headers.get("authorization").unwrap().to_str().unwrap();

        safe_basic_auth_decoder(auth_header).unwrap()
    }
}

// This is an unsafe version of the utility function, `safe_basic_auth_decoder`, and should only be used when the auth header has already gone through the utility function.
// pub fn basic_auth_decoder(auth_header: &str) -> AuthData {
//     // split auth header from "Basic {auth}"
//     let auth_header = auth_header.split_whitespace().collect::<Vec<_>>();

//     // obtain the base64 string from the header
//     let auth_header = auth_header.get(1).unwrap();
//     // decode base64 from the string
//     let auth_header = base64::decode(auth_header).unwrap();

//     // convert the bytes to a string
//     let auth_header = String::from_utf8(auth_header).unwrap();

//     println!("testing basic auth header decoding: {}", auth_header);

//     // get the username and password from the email:player_token
//     let auth_header = auth_header.split(":").collect::<Vec<_>>();
//     let player_email = auth_header.get(0).unwrap();
//     let player_token = auth_header.get(1).unwrap();

//     println!("email: {}, token: {}", player_email, player_token);

//     AuthData {
//         email: player_email.to_string(),
//         token: player_token.to_string(),
//     }
// }

pub fn encode_user_token(email: &str, token: &str, userdata_auth: &str) -> String {
    let mut user_token = Hmac::new(Sha1::new(), userdata_auth.as_bytes());
    user_token.input(email.as_bytes());
    user_token.input(token.as_bytes());

    user_token
        .result()
        .code()
        .iter()
        .map(|byte| format!("{:02x?}", byte))
        .collect::<Vec<String>>()
        .join("")
}
