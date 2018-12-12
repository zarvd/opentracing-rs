use crate::{Span, Transport};

pub trait Reporter {
    fn report(&mut self, span: Span);
    fn close(&mut self);
}

#[derive(Default)]
pub struct NullReporter {}

impl NullReporter {}

impl Reporter for NullReporter {
    fn report(&mut self, span: Span) {}
    fn close(&mut self) {}
}

pub struct LoggingReporter {
    // TODO logger
}

impl LoggingReporter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Reporter for LoggingReporter {
    fn report(&mut self, span: Span) {
        println!("reporting span {:?}", span);
    }

    fn close(&mut self) {}
}

#[derive(Default)]
pub struct RemoteReporter<T> {
    sender: T,
}

impl<T> RemoteReporter<T>
where
    T: Transport,
{
    pub fn new(sender: T) -> Self {
        Self { sender }
    }
}

impl<T> Reporter for RemoteReporter<T>
where
    T: Transport,
{
    fn report(&mut self, span: Span) {
        self.sender.append(span);
    }

    fn close(&mut self) {
        self.sender.flush();
    }
}
