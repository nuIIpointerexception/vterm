use ::anyhow::Result;

pub use self::composed_message::{ComposedElement, ComposedMessage};
use crate::{
    graphics::triangles::Frame,
    ui::{
        primitives::Dimensions,
        widgets::{Element, Widget},
        Id, Input, InternalState,
    },
    Vec2,
};

mod composed_message;

pub trait CompositeWidget<IMessage, EMessage> {
    type State;

    fn id(&self) -> &Id;

    fn view(&mut self, state: &Self::State) -> Element<ComposedMessage<IMessage, EMessage>>;

    fn update(&self, state: &mut Self::State, event: IMessage) -> Result<()>;
}

pub struct Composite<IMessage, EMessage, CW>
where
    CW: CompositeWidget<IMessage, EMessage>,
{
    composite: CW,
    current_view: Option<Element<ComposedMessage<IMessage, EMessage>>>,
}

impl<IMessage, EMessage, CW> Composite<IMessage, EMessage, CW>
where
    CW: CompositeWidget<IMessage, EMessage>,
{
    pub fn new(composite: CW) -> Self {
        Self { composite, current_view: None }
    }
}

impl<IMessage: 'static, EMessage: 'static, CW> Widget<EMessage>
    for Composite<IMessage, EMessage, CW>
where
    CW: CompositeWidget<IMessage, EMessage> + 'static,
    CW::State: 'static + Default,
{
    fn handle_event(
        &mut self,
        internal_state: &mut InternalState,
        input: &Input,
        event: &winit::event::WindowEvent,
    ) -> Result<Option<EMessage>> {
        if self.current_view.is_none() {
            let current_state = internal_state.get_state::<CW::State>(self.composite.id());
            self.current_view = Some(self.composite.view(current_state));
        }
        let result =
            self.current_view.as_mut().unwrap().handle_event(internal_state, input, event)?;
        match result {
            Some(ComposedMessage::Internal(internal)) => {
                let state = internal_state.get_state_mut::<CW::State>(self.composite.id());
                self.composite.update(state, internal)?;
                return Ok(None);
            }
            Some(ComposedMessage::External(ext)) => {
                return Ok(Some(ext));
            }
            None => {
                return Ok(None);
            }
        }
    }

    fn draw_frame(&mut self, internal_state: &mut InternalState, frame: &mut Frame) -> Result<()> {
        self.current_view.as_mut().unwrap().draw_frame(internal_state, frame)
    }

    fn dimensions(
        &mut self,
        internal_state: &mut InternalState,
        max_size: &Dimensions,
    ) -> Dimensions {
        if self.current_view.is_none() {
            let current_state = internal_state.get_state::<CW::State>(self.composite.id());
            self.current_view = Some(self.composite.view(current_state));
        }
        self.current_view.as_mut().unwrap().dimensions(internal_state, max_size)
    }

    fn set_top_left_position(&mut self, internal_state: &mut InternalState, position: Vec2) {
        if self.current_view.is_none() {
            let current_state = internal_state.get_state::<CW::State>(self.composite.id());
            self.current_view = Some(self.composite.view(current_state));
        }
        self.current_view.as_mut().unwrap().set_top_left_position(internal_state, position);
    }
}
