use http_body::Body;

mod aggregate;
pub use aggregate::{Aggregate, AggregateFuture};

pub trait BodyExt: Body {
    #[inline]
    fn aggregate(&mut self, content_len: usize) -> AggregateFuture<'_, Self>
    where
        Self: Unpin + Sized,
        Self::Data: Unpin,
    {
        AggregateFuture::new(self, content_len)
    }
}
