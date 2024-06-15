use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};


#[derive(Debug)]
struct Delay {
    when: Instant
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready("done") // represent that a value is immediately ready
        }
        else {
            cx.waker().wake_by_ref();
            Poll::Pending // represent that a value is not ready yet
        }
    }
}

#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_millis(10);
    let future = Delay {when};

    let out = future.await;
    assert_eq!(out, "done");
}
