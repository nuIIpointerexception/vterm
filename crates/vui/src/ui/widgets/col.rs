use ::anyhow::Result;

use crate::{
    graphics::triangles::Frame,
    ui::{
        Input,
        InternalState,
        primitives::{DimensionList, Dimensions, Justify, SpaceBetween}, widgets::{Element, Widget},
    },
    Vec2,
};

use super::CompositeStyle;

pub struct Col<Message> {
    children: Vec<Element<Message>>,
    child_dimensions: DimensionList,
    justify: Justify,
}

impl<Message> Col<Message> {
    pub fn new() -> Self {
        Self {
            children: vec![],
            child_dimensions: DimensionList::vertical(),
            justify: Justify::Begin,
        }
    }

    pub fn space_between(mut self, space_between: SpaceBetween) -> Self {
        self.child_dimensions = self.child_dimensions.space_between(space_between);
        self
    }

    pub fn child<W>(mut self, child: W) -> Self
    where
        W: Into<Element<Message>>,
    {
        self.children.push(child.into());
        self
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }
}

impl<Message: 'static> Widget<Message> for Col<Message> {
    fn handle_event(
        &mut self,
        internal_state: &mut InternalState,
        input: &Input,
        event: &winit::event::WindowEvent,
    ) -> Result<Option<Message>> {
        for child in &mut self.children {
            if let Some(message) = child.handle_event(internal_state, input, event)? {
                return Ok(Some(message));
            }
        }
        Ok(None)
    }

    fn draw_frame(
        &mut self,
        internal_state: &mut InternalState,
        frame: &mut Frame,
    ) -> Result<()> {
        for child in &mut self.children {
            child.draw_frame(internal_state, frame)?;
        }
        Ok(())
    }

    fn dimensions(
        &mut self,
        internal_state: &mut InternalState,
        max_size: &Dimensions,
    ) -> Dimensions {
        if self.children.is_empty() {
            return Dimensions::new(0.0, 0.0);
        }

        self.child_dimensions.set_max_size(max_size);

        let mut remaining_size = *max_size;
        for child in &mut self.children {
            let child_bounds = child.dimensions(internal_state, &remaining_size);
            remaining_size = self.child_dimensions.add_child_dimensions(child_bounds, self.justify);
        }

        self.child_dimensions.dimensions()
    }

    fn set_top_left_position(
        &mut self,
        internal_state: &mut InternalState,
        position: Vec2,
    ) {
        let positions = self.child_dimensions.compute_child_positions();
        for (child, child_pos) in self.children.iter_mut().zip(positions.iter()) {
            child.set_top_left_position(internal_state, position + *child_pos);
        }
    }
}

impl<Message> Into<Element<Message>> for Col<Message>
where
    Message: 'static,
{
    fn into(self) -> Element<Message> {
        Element::new(self)
    }
}
