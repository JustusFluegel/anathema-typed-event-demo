use library_package::traits::{AsTypedEvent, Event, Publishable};

#[derive(Event)]
#[event(prefix = "abc")]
pub enum MyEvents {
    #[event(rename = "event_a")]
    EventA(String),
    EventB,
    EventC {
        foo: usize,
    },
}

fn main() {
    assert_eq!(MyEvents::EventA(String::new()).event_ident(), "event_a");
    assert_eq!(MyEvents::EventB.event_ident(), "abcEventB");
    assert_eq!(MyEvents::EventC { foo: 1 }.event_ident(), "abcEventC");

    let my_event = MyEvents::EventC { foo: 1 };

    let context: anathema::component::Context<()> = todo!();
    context.publish_typed(my_event);

    let untyped_event: anathema::component::UserEvent = todo!();
    let my_event: Option<&MyEvents> = untyped_event.as_typed_event();

    let pos = anathema::geometry::Pos { x: 1, y: 2 };
}
