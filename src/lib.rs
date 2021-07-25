use http_body::Body;

mod aggregate;
pub use aggregate::{Aggregate, AggregateFuture};


pub trait BodyExt: Body {
    fn aggregate(&mut self) -> AggregateFuture<'_, Self>
    where
        Self: Unpin + Sized,
        Self::Data: Unpin
    {
       AggregateFuture::new(self)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
