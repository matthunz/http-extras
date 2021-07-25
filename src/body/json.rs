use super::{aggregate::AggregateError, AggregateFuture};
use bytes::Buf;
use http_body::Body;
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

pub enum JsonError<E> {
    Aggregate(AggregateError<E>),
    Deserialize(serde_json::Error),
}

pin_project! {
    pub struct JsonFuture<'b, B: Body, T> {
        #[pin]
        aggregate: AggregateFuture<'b, B>,
        _marker: PhantomData<T>
    }
}

impl<'b, B, T> JsonFuture<'b, B, T>
where
    B: Body + Unpin,
    T: DeserializeOwned,
{
    pub(crate) fn new(body: &'b mut B, content_len: usize) -> Self {
        Self {
            aggregate: AggregateFuture::new(body, content_len),
            _marker: PhantomData,
        }
    }
}

impl<B, T> Future for JsonFuture<'_, B, T>
where
    B: Body + Unpin,
    T: DeserializeOwned,
{
    type Output = Result<T, JsonError<B::Error>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.project().aggregate.poll(cx).map(|result| {
            result.map_err(JsonError::Aggregate).and_then(|bufs| {
                serde_json::from_reader(bufs.reader()).map_err(JsonError::Deserialize)
            })
        })
    }
}
