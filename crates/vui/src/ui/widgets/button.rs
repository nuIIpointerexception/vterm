use ::anyhow::Result;

use crate::{
    builder_field, builder_field_some,
    graphics::triangles::Frame,
    ui::{
        Id,
        Input,
        InternalState, primitives::{Dimensions, Rect, Tile}, widgets::{Element, Widget},
    },
    vec2, Vec2, vec4, Vec4,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ButtonState {
    Inactive,
    Hover,
    Pressed,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self::Inactive
    }
}

pub struct Button<Message> {
    id: Id,

    child: Element<Message>,

    background: Rect,

    color: Vec4,

    hover_color: Vec4,

    pressed_color: Vec4,

    on_click: Option<Message>,
}

impl<Message> Button<Message> {
    pub fn new<W>(id: Id, child: W) -> Self
    where
        W: Into<Element<Message>>,
    {
        Self {
            id,
            child: child.into(),
            background: Rect::new(0.0, 0.0, 0.0, 0.0),
            color: vec4(0.1, 0.1, 0.1, 1.0),
            hover_color: vec4(0.3, 0.3, 0.3, 1.0),
            pressed_color: vec4(0.5, 0.5, 0.5, 1.0),
            on_click: None,
        }
    }

    builder_field!(id, Id);
    builder_field!(color, Vec4);
    builder_field!(hover_color, Vec4);
    builder_field!(pressed_color, Vec4);
    builder_field_some!(on_click, Message);
}

impl<Message> Widget<Message> for Button<Message>
where
    Message: Copy + Clone + std::fmt::Debug,
{
    fn handle_event(
        &mut self,
        internal_state: &mut InternalState,
        input: &Input,
        event: &glfw::WindowEvent,
    ) -> Result<Option<Message>> {
        use glfw::{Action, MouseButton, WindowEvent};

        let state = internal_state.get_state_mut::<ButtonState>(&self.id);
        let message = match *event {
            WindowEvent::CursorPos(x, y) => {
                if self.background.contains(vec2(x as f32, y as f32)) {
                    if *state == ButtonState::Inactive {
                        *state = ButtonState::Hover;
                    }
                } else {
                    *state = ButtonState::Inactive;
                }
                None
            }
            WindowEvent::MouseButton(
                MouseButton::Button1,
                Action::Press,
                _,
            ) => {
                if *state == ButtonState::Hover {
                    *state = ButtonState::Pressed;
                }
                None
            }
            WindowEvent::MouseButton(
                MouseButton::Button1,
                Action::Release,
                _,
            ) => {
                if *state == ButtonState::Pressed {
                    if self.background.contains(input.mouse_position) {
                        *state = ButtonState::Hover;
                    } else {
                        *state = ButtonState::Inactive;
                    }
                    self.on_click
                } else {
                    None
                }
            }
            _ => None,
        };
        Ok(message)
    }

    fn draw_frame(
        &self,
        internal_state: &mut InternalState,
        frame: &mut Frame,
    ) -> Result<()> {
        let state = internal_state.get_state::<ButtonState>(&self.id);
        let color = match *state {
            ButtonState::Inactive => self.color,
            ButtonState::Hover => self.hover_color,
            ButtonState::Pressed => self.pressed_color,
        };
        Tile {
            model: self.background,
            color,
            ..Default::default()
        }
        .fill(frame)?;

        self.child.draw_frame(internal_state, frame)
    }

    fn dimensions(
        &mut self,
        internal_state: &mut InternalState,
        max_size: &Dimensions,
    ) -> Dimensions {
        let child_dimensions = self.child.dimensions(internal_state, max_size);
        self.background = child_dimensions.as_rect();
        child_dimensions
    }

    fn set_top_left_position(
        &mut self,
        internal_state: &mut InternalState,
        position: Vec2,
    ) {
        let offset = position - self.background.top_left;
        self.background = self.background.translate(offset);
        self.child.set_top_left_position(internal_state, position);
    }
}

impl<Message> Into<Element<Message>> for Button<Message>
where
    Message: 'static + Copy + Clone + std::fmt::Debug,
{
    fn into(self) -> Element<Message> {
        Element::new(self)
    }
}
