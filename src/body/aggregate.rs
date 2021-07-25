use bytes::{Buf, BufMut, Bytes, BytesMut};
use http_body::Body;
use std::io::IoSlice;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{collections::VecDeque, future::Future};

pub enum AggregateError<E> {
    Body(E),
    ContentLength,
}

pub struct AggregateFuture<'b, B: Body> {
    body: &'b mut B,
    bufs: Option<Aggregate<B::Data>>,
    content_len: usize,
    used: usize,
}

impl<'b, B> AggregateFuture<'b, B>
where
    B: Body + Unpin,
    B::Data: Unpin,
{
    pub(crate) fn new(body: &'b mut B, content_len: usize) -> Self {
        Self {
            body,
            bufs: Some(Aggregate::new()),
            content_len,
            used: 0,
        }
    }
}

impl<B> Future for AggregateFuture<'_, B>
where
    B: Body + Unpin,
    B::Data: Unpin,
{
    type Output = Result<Aggregate<B::Data>, AggregateError<B::Error>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let me = self.get_mut();
        loop {
            match Pin::new(&mut me.body).poll_data(cx) {
                Poll::Ready(Some(Ok(data))) => {
                    me.used += data.remaining();
                    if me.content_len > me.used {
                        break Poll::Ready(Err(AggregateError::ContentLength));
                    }

                    if let Some(ref mut buf) = me.bufs {
                        buf.push(data);
                    }
                }
                Poll::Ready(Some(Err(error))) => {
                    break Poll::Ready(Err(AggregateError::Body(error)))
                }
                Poll::Ready(None) => break Poll::Ready(Ok(me.bufs.take().unwrap())),
                Poll::Pending => break Poll::Pending,
            }
        }
    }
}

pub struct Aggregate<T> {
    bufs: VecDeque<T>,
}

impl<T: Buf> Aggregate<T> {
    pub(crate) fn new() -> Aggregate<T> {
        Aggregate {
            bufs: VecDeque::new(),
        }
    }

    #[inline]
    pub fn push(&mut self, buf: T) {
        debug_assert!(buf.has_remaining());
        self.bufs.push_back(buf);
    }

    #[inline]
    #[cfg(feature = "http1")]
    pub(crate) fn bufs_cnt(&self) -> usize {
        self.bufs.len()
    }
}

impl<T: Buf> Buf for Aggregate<T> {
    #[inline]
    fn remaining(&self) -> usize {
        self.bufs.iter().map(|buf| buf.remaining()).sum()
    }

    #[inline]
    fn chunk(&self) -> &[u8] {
        self.bufs.front().map(Buf::chunk).unwrap_or_default()
    }

    #[inline]
    fn advance(&mut self, mut cnt: usize) {
        while cnt > 0 {
            {
                let front = &mut self.bufs[0];
                let rem = front.remaining();
                if rem > cnt {
                    front.advance(cnt);
                    return;
                } else {
                    front.advance(rem);
                    cnt -= rem;
                }
            }
            self.bufs.pop_front();
        }
    }

    #[inline]
    fn chunks_vectored<'t>(&'t self, dst: &mut [IoSlice<'t>]) -> usize {
        if dst.is_empty() {
            return 0;
        }
        let mut vecs = 0;
        for buf in &self.bufs {
            vecs += buf.chunks_vectored(&mut dst[vecs..]);
            if vecs == dst.len() {
                break;
            }
        }
        vecs
    }

    #[inline]
    fn copy_to_bytes(&mut self, len: usize) -> Bytes {
        match self.bufs.front_mut() {
            Some(front) if front.remaining() == len => {
                let b = front.copy_to_bytes(len);
                self.bufs.pop_front();
                b
            }
            Some(front) if front.remaining() > len => front.copy_to_bytes(len),
            _ => {
                assert!(len <= self.remaining(), "`len` greater than remaining");
                let mut bm = BytesMut::with_capacity(len);
                bm.put(self.take(len));
                bm.freeze()
            }
        }
    }
}
