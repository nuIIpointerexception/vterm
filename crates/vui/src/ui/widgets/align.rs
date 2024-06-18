use ::anyhow::Result;

use crate::{
    builder_field,
    graphics::triangles::Frame,
    ui::{
        Input,
        InternalState,
        primitives::Dimensions, widgets::{Element, Widget},
    },
    vec2, Vec2,
};

use super::CompositeStyle;

#[derive(Debug, Copy, Clone)]
pub enum HAlignment {
    Center,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone)]
pub enum VAlignment {
    Center,
    Bottom,
    Top,
}

#[derive(Debug, Copy, Clone)]
pub struct Align<Message, W: Widget<Message>> {
    horizontal_alignment: HAlignment,
    vertical_alignment: VAlignment,
    child: W,
    child_offset: Vec2,
    _phantom_data: std::marker::PhantomData<Message>,
}

impl<Message, W: Widget<Message>> Align<Message, W> {
    pub fn new(child: W) -> Self {
        Self {
            horizontal_alignment: HAlignment::Center,
            vertical_alignment: VAlignment::Center,
            child,
            child_offset: vec2(0.0, 0.0),
            _phantom_data: Default::default(),
        }
    }

    builder_field!(horizontal_alignment, HAlignment);
    builder_field!(vertical_alignment, VAlignment);

    pub fn alignment(
        self,
        horizontal: HAlignment,
        vertical: VAlignment,
    ) -> Self {
        self.horizontal_alignment(horizontal)
            .vertical_alignment(vertical)
    }
}

impl<Message: 'static, W: Widget<Message>> Widget<Message> for Align<Message, W> {
    fn handle_event(
        &mut self,
        internal_state: &mut InternalState,
        input: &Input,
        event: &winit::event::WindowEvent,
    ) -> Result<Option<Message>> {
        self.child.handle_event(internal_state, input, event)
    }

    fn draw_frame(
        &mut self,
        internal_state: &mut InternalState,
        frame: &mut Frame,
    ) -> Result<()> {
        self.child.draw_frame(internal_state, frame)
    }

    fn dimensions(
        &mut self,
        internal_state: &mut InternalState,
        max_size: &Dimensions,
    ) -> Dimensions {
        let child_dimensions = self.child.dimensions(internal_state, max_size);
        let remaining_width = max_size.width - child_dimensions.width;
        let remaining_height = max_size.height - child_dimensions.height;

        self.child_offset = vec2(
            match self.horizontal_alignment {
                HAlignment::Left => 0.0,
                HAlignment::Center => 0.5 * remaining_width,
                HAlignment::Right => remaining_width,
            },
            match self.vertical_alignment {
                VAlignment::Top => 0.0,
                VAlignment::Center => 0.5 * remaining_height,
                VAlignment::Bottom => remaining_height,
            },
        );
        self.child_offset.x = self.child_offset.x.round();
        self.child_offset.y = self.child_offset.y.round();

        *max_size
    }

    fn set_top_left_position(
        &mut self,
        internal_state: &mut InternalState,
        position: Vec2,
    ) {
        self.child.set_top_left_position(
            internal_state,
            position + self.child_offset,
        );
    }
}

impl<Message, W> Into<Element<Message>> for Align<Message, W>
where
    Message: 'static + std::fmt::Debug + Copy + Clone,
    W: Widget<Message> + 'static,
{
    fn into(self) -> Element<Message> {
        Element::new(self)
    }
}
