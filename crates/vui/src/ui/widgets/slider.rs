use ::anyhow::Result;

use crate::{
    builder_field,
    graphics::triangles::Frame,
    ui::{
        Id,
        Input,
        InternalState, primitives::{Dimensions, Rect, Tile}, widgets::{Element, Widget},
    },
    vec2, Vec2, vec4,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SliderState {
    Inactive,
    Focused,
    Active,
}

impl Default for SliderState {
    fn default() -> Self {
        Self::Inactive
    }
}

pub struct Slider<Message> {
    id: Id,
    min: f32,
    max: f32,
    value: f32,
    cursor: Rect,
    bounds: Rect,
    value_line: Rect,
    height_ratio: f32,
    on_change: Option<Box<dyn Fn(f32) -> Message>>,
}

impl<Message> Slider<Message> {
    pub fn new(id: Id, min: f32, max: f32) -> Self {
        Self {
            id,
            min,
            max,
            value: min,
            cursor: Rect::new(0.0, 0.0, 0.0, 0.0),
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            value_line: Rect::new(0.0, 0.0, 0.0, 0.0),
            height_ratio: 1.0 / 10.0,
            on_change: None,
        }
    }

    builder_field!(height_ratio, f32);

    pub fn value(self, value: f32) -> Self {
        Self {
            value: value.clamp(self.min, self.max),
            ..self
        }
    }

    pub fn on_change<F>(self, on_change_fn: F) -> Self
    where
        F: 'static + Fn(f32) -> Message,
    {
        Self {
            on_change: Some(Box::new(on_change_fn)),
            ..self
        }
    }
}

impl<Message> Widget<Message> for Slider<Message> {
    fn handle_event(
        &mut self,
        internal_state: &mut InternalState,
        input: &Input,
        event: &glfw::WindowEvent,
    ) -> Result<Option<Message>> {
        use glfw::{Action, MouseButton, WindowEvent};

        let current_state =
            internal_state.get_state_mut::<SliderState>(&self.id);

        match *event {
            WindowEvent::CursorPos(x, y) => {
                if self.bounds.contains(vec2(x as f32, y as f32)) {
                    if current_state == &SliderState::Inactive {
                        *current_state = SliderState::Focused;
                    }
                } else {
                    if current_state == &SliderState::Focused {
                        *current_state = SliderState::Inactive;
                    }
                }
            }
            WindowEvent::MouseButton(
                MouseButton::Button1,
                Action::Press,
                _,
            ) => match current_state {
                SliderState::Inactive => (),
                SliderState::Focused => {
                    if self.bounds.contains(input.mouse_position) {
                        *current_state = SliderState::Active;
                    }
                }
                _ => (),
            },
            WindowEvent::MouseButton(
                MouseButton::Button1,
                Action::Release,
                _,
            ) => {
                if current_state == &SliderState::Active {
                    if self.bounds.contains(input.mouse_position) {
                        *current_state = SliderState::Focused;
                    } else {
                        *current_state = SliderState::Inactive;
                    }
                }
            }
            _ => (),
        }

        if current_state == &SliderState::Active {
            let cursor_pos = input
                .mouse_position
                .x
                .clamp(self.value_line.left(), self.value_line.right());
            let normalized_value =
                (cursor_pos - self.value_line.left()) / self.value_line.width();
            let new_value =
                self.min + normalized_value * (self.max - self.min).abs();

            if let Some(on_change) = &self.on_change {
                Ok(Some(on_change(new_value)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn draw_frame(
        &self,
        _internal_state: &mut InternalState,
        frame: &mut Frame,
    ) -> Result<()> {
        Tile {
            model: self.value_line,
            color: vec4(0.0, 0.0, 0.0, 0.5),
            ..Default::default()
        }
        .fill(frame)?;

        Tile {
            model: self.cursor,
            color: vec4(0.5, 0.5, 0.5, 0.9),
            ..Default::default()
        }
        .fill(frame)?;

        Ok(())
    }

    fn dimensions(
        &mut self,
        _internal_state: &mut InternalState,
        max_size: &Dimensions,
    ) -> Dimensions {
        self.bounds = Dimensions::new(
            max_size.width,
            max_size.height.min(max_size.width * self.height_ratio),
        )
        .as_rect();

        self.cursor = Dimensions::new(
            self.bounds.height() * 0.9,
            self.bounds.height() * 0.9,
        )
        .as_rect();

        self.value_line = Dimensions::new(
            self.bounds.width() - self.cursor.width(),
            self.bounds.height() * self.height_ratio,
        )
        .as_rect();

        self.bounds.dimensions()
    }

    fn set_top_left_position(
        &mut self,
        _internal_state: &mut InternalState,
        position: Vec2,
    ) {
        self.bounds = self.bounds.set_top_left_position(position);

        let value_line_offset = vec2(
            0.5 * (self.bounds.width() - self.value_line.width()),
            0.5 * (self.bounds.height() - self.value_line.height()),
        );
        self.value_line = self
            .value_line
            .set_top_left_position(position + value_line_offset);

        let normalized_cursor_pos = self.value / (self.max - self.min).abs();
        let cursor_pos_x = self.value_line.left()
            + (normalized_cursor_pos * self.value_line.width())
            - 0.5 * self.cursor.width();
        let cursor_pos_y = self.bounds.top()
            + 0.5 * (self.bounds.height() - self.cursor.height());

        self.cursor = self
            .cursor
            .set_top_left_position(vec2(cursor_pos_x, cursor_pos_y));
    }
}

impl<Message> Into<Element<Message>> for Slider<Message>
where
    Message: 'static,
{
    fn into(self) -> Element<Message> {
        Element::new(self)
    }
}
