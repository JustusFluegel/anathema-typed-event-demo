pub mod traits {
    pub trait Publishable<T> {
        fn publish_typed(&mut self, event: T);
    }

    pub trait AsTypedEvent<T> {
        fn as_typed_event(&self) -> Option<&T>;
    }

    // import the macro into the same scope so they always get imported together.
    pub use macro_package::Event;
    pub trait Event {
        fn event_ident(&self) -> &'static str;
    }
}
