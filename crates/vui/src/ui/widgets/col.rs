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

pub struct Col<Message> {
    children: Vec<(Element<Message>, Justify)>,
    child_dimensions: DimensionList,
}

impl<Message> Col<Message> {
    pub fn new() -> Self {
        Self {
            children: vec![],
            child_dimensions: DimensionList::vertical(),
        }
    }

    pub fn space_between(self, space_between: SpaceBetween) -> Self {
        Self {
            child_dimensions: self
                .child_dimensions
                .space_between(space_between),
            ..self
        }
    }

    pub fn child<W>(mut self, child: W, justify: Justify) -> Self
    where
        W: Into<Element<Message>>,
    {
        self.children.push((child.into(), justify));
        self
    }
}

impl<Message> Widget<Message> for Col<Message> {
    fn handle_event(
        &mut self,
        internal_state: &mut InternalState,
        input: &Input,
        event: &winit::event::WindowEvent,
    ) -> Result<Option<Message>> {
        for (child, _) in &mut self.children {
            if let Some(message) =
                child.handle_event(internal_state, input, event)?
            {
                return Ok(Some(message));
            }
        }
        Ok(None)
    }

    fn draw_frame(
        &self,
        internal_state: &mut InternalState,
        frame: &mut Frame,
    ) -> Result<()> {
        for (child, _) in &self.children {
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

        for (child, justify) in &mut self.children {
            let child_bounds =
                child.dimensions(internal_state, &remaining_size);

            remaining_size = self
                .child_dimensions
                .add_child_dimensions(child_bounds, *justify);
        }

        self.child_dimensions.dimensions()
    }

    fn set_top_left_position(
        &mut self,
        internal_state: &mut InternalState,
        position: Vec2,
    ) {
        let positions = self.child_dimensions.compute_child_positions();
        for ((child, _), child_pos) in
            self.children.iter_mut().zip(positions.iter())
        {
            child.set_top_left_position(internal_state, position + child_pos);
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
