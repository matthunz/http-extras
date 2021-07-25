use http_body::Body;

mod aggregate;
pub use aggregate::{Aggregate, AggregateFuture};

mod json;
pub use json::JsonFuture;
use serde::de::DeserializeOwned;

pub trait BodyExt: Body + Unpin + Sized {
    #[inline]
    fn aggregate(&mut self, content_len: usize) -> AggregateFuture<'_, Self> {
        AggregateFuture::new(self, content_len)
    }

    fn json<T>(&mut self, content_len: usize) -> JsonFuture<'_, Self, T>
    where
        T: DeserializeOwned,
    {
        JsonFuture::new(self, content_len)
    }
}

impl<B> BodyExt for B where B: Body + Unpin {}
