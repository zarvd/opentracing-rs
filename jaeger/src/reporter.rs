use crate::Span;

pub trait Reporter {
    fn report(&mut self, span: &Span);
    fn close(&mut self);
}

#[derive(Default)]
pub struct NullReporter {}

impl NullReporter {}

impl Reporter for NullReporter {
    fn report(&mut self, _span: &Span) {}
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
    fn report(&mut self, span: &Span) {
        println!("reporting span {:?}", span);
    }

    fn close(&mut self) {}
}

#[derive(Default)]
pub struct RemoteReporter {}

impl RemoteReporter {}

impl Reporter for RemoteReporter {
    fn report(&mut self, span: &Span) {}

    fn close(&mut self) {}
}

#[derive(Default)]
pub struct CompositeReporter {
    reporters: Vec<Box<Reporter>>,
}

impl CompositeReporter {
    pub fn new() -> Self {
        Self {
            reporters: Vec::new(),
        }
    }

    pub fn add_reporter(&mut self, reporter: Box<Reporter>) {
        self.reporters.push(reporter);
    }
}

impl Reporter for CompositeReporter {
    fn report(&mut self, span: &Span) {
        for reporter in &mut self.reporters {
            reporter.report(span);
        }
    }

    fn close(&mut self) {
        for reporter in &mut self.reporters {
            reporter.close();
        }
    }
}
