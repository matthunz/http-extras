use http::Request;

mod auth;
pub use auth::Authorization;

pub trait RequestExt {
    fn authorization(&self) -> Authorization<'_>;
}

impl<B> RequestExt for Request<B> {
    #[inline]
    fn authorization(&self) -> Authorization<'_> {
        Authorization::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::header;

    #[test]
    fn it_works() {
        let req = Request::builder()
            .header(header::AUTHORIZATION, "Bearer 1234")
            .body(())
            .unwrap();
        dbg!(req.authorization().bearer().unwrap());
    }
}
