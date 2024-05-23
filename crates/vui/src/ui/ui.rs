use std::time::Duration;

use ::anyhow::Result;

use crate::{
    graphics::triangles::Frame,
    Mat4,
    ui::{
        Input,
        InternalState,
        primitives::{Dimensions, Rect},
        ui_screen_space_projection, widgets::{Element, Widget},
    }, vec2,
};

pub trait UIState {
    type Message;

    fn view(&self) -> Element<Self::Message>;

    fn update(&mut self, message: &Self::Message);
}

pub struct UI<C: UIState> {
    viewport: Rect,
    projection: Mat4,
    custom: C,
    current_view: Element<C::Message>,
    internal_state: InternalState,
    input: Input,

    fps: f32,
    frame_times: [Duration; 100],
    frame_time_index: usize,
    last_fps_draw_time: std::time::Instant,
}

impl<C: UIState> UI<C> {
    pub fn new(viewport: Dimensions, custom_ui: C) -> Self {
        let mut ui = Self {
            viewport: Rect::new(0.0, 0.0, viewport.height, viewport.width),
            projection: ui_screen_space_projection(viewport),
            current_view: custom_ui.view(),
            custom: custom_ui,
            internal_state: InternalState::new(),
            input: Input::new(),
            fps: 0.0,
            frame_times: [Duration::new(0, 0); 100],
            frame_time_index: 0,
            last_fps_draw_time: std::time::Instant::now(),
        };
        ui.layout();
        ui
    }

    pub fn handle_event(
        &mut self,
        event: &glfw::WindowEvent,
    ) -> Result<Option<C::Message>> {
        use glfw::WindowEvent;

        self.input.handle_event(event);
        match *event {
            WindowEvent::FramebufferSize(width, height) => {
                self.viewport =
                    Rect::new(0.0, 0.0, height as f32, width as f32);
                self.projection =
                    ui_screen_space_projection((width, height).into());
            }
            _ => (),
        }

        let message_opt = self.current_view.handle_event(
            &mut self.internal_state,
            &self.input,
            event,
        )?;

        if let Some(message) = &message_opt {
            self.custom.update(message);
            self.flush();
        } else {
            self.layout();
        }

        Ok(message_opt)
    }

    pub fn state(&self) -> &C {
        &self.custom
    }

    pub fn state_mut(&mut self) -> &mut C {
        &mut self.custom
    }

    pub fn draw_frame(&mut self, frame: &mut Frame) -> Result<()> {
        self.flush();

        let frame_start = std::time::Instant::now();

        frame.set_view_projection(self.projection);
        self.current_view
            .draw_frame(&mut self.internal_state, frame)?;

        let frame_time = frame_start.elapsed();
        self.frame_times[self.frame_time_index] = frame_time;
        self.frame_time_index =
            (self.frame_time_index + 1) % self.frame_times.len();

        let elapsed_time = frame_start - self.last_fps_draw_time;
        if elapsed_time >= Duration::from_secs(1) {
            let total_frame_time: Duration = self.frame_times.iter().sum();
            let average_frame_time =
                total_frame_time / self.frame_times.len() as u32;

            if average_frame_time.as_secs_f32() > 0.0 {
                self.fps = 1.0 / average_frame_time.as_secs_f32();
            }

            self.last_fps_draw_time = frame_start;
        }

        Ok(())
    }
}

impl<C: UIState> UI<C> {
    fn flush(&mut self) {
        self.current_view = self.custom.view();
        self.layout();
    }

    fn layout(&mut self) {
        let _root_widget_dimensions = self
            .current_view
            .dimensions(&mut self.internal_state, &self.viewport.dimensions());
        self.current_view
            .set_top_left_position(&mut self.internal_state, vec2(0.0, 0.0));
    }
}
