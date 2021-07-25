use http::{header, HeaderValue, Request};

#[derive(Clone, Copy, Debug)]
pub enum ToStrError {
    MissingHeader,
    InvalidStr,
}

#[derive(Clone, Copy, Debug)]
pub enum TokenError {
    Token,
    ToStr(ToStrError),
}

pub struct Authorization<'a> {
    header: Option<&'a HeaderValue>,
}

impl<'a> Authorization<'a> {
    pub fn new<'r, B>(req: &'r Request<B>) -> Self
    where
        'r: 'a,
    {
        Self {
            header: req.headers().get(header::AUTHORIZATION),
        }
    }

    #[inline]
    pub fn bearer(&self) -> Result<&str, TokenError> {
        self.token("Bearer ")
    }

    #[inline]
    pub fn token(&self, prefix: &str) -> Result<&str, TokenError> {
        self.to_str()
            .map_err(|error| TokenError::ToStr(error))
            .and_then(|s| s.strip_prefix(prefix).ok_or(TokenError::Token))
    }

    #[inline]
    pub fn to_str(&self) -> Result<&str, ToStrError> {
        self.header
            .ok_or(ToStrError::MissingHeader)
            .and_then(|header| header.to_str().map_err(|_| ToStrError::InvalidStr))
    }
}
