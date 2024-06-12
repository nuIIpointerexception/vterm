use ::anyhow::Result;

use crate::{
    gen_id,
    ui::{
        id_hash,
        primitives::{Justify, SpaceBetween},
        widgets::{
            Col, ComposedMessage, Composite, CompositeWidget,
            Container, Element, Label, Row, WithContainer,
        },
        Font, Id,
    },
    vec4,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WindowState {
    Hidden,
    Visible,
}

impl Default for WindowState {
    fn default() -> Self {
        WindowState::Hidden
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WindowEvent {
    ShowWindow,
    HideWindow,
}

pub struct Window<Message> {
    id: Id,
    font: Font,
    title: String,
    contents: Option<Element<Message>>,
}

impl<Message> Window<Message>
    where
        Message: 'static + std::fmt::Debug + Copy + Clone,
{
    pub fn new(font: Font, title: impl Into<String>) -> Self {
        let owned_title = title.into();
        Self {
            id: gen_id!(&owned_title),
            font,
            title: owned_title,
            contents: None,
        }
    }

    pub fn contents(self, contents: impl Into<Element<Message>>) -> Self {
        Self {
            contents: Some(contents.into()),
            ..self
        }
    }
}

impl<Message> CompositeWidget<WindowEvent, Message> for Window<Message>
    where
        Message: 'static + std::fmt::Debug + Copy + Clone,
{
    type State = WindowState;

    fn id(&self) -> &Id {
        &self.id
    }

    fn view(
        &mut self,
        state: &Self::State,
    ) -> Element<ComposedMessage<WindowEvent, Message>> {
         match state {
            WindowState::Hidden => {
                let top_bar = Row::new()
                    .child(Label::new(&self.font, &self.title), Justify::Center)
                    .space_between(SpaceBetween::EvenSpaceBetween);

                Col::new().child(top_bar, Justify::End).into()
            }
            WindowState::Visible => {
                let top_bar = Row::new()
                    .child(Label::new(&self.font, &self.title), Justify::Center)
                    .space_between(SpaceBetween::EvenSpaceBetween);

                let contents: Element<Message> =
                    self.contents.take().unwrap().into();

                Col::new()
                    .child(top_bar, Justify::End)
                    .child(contents, Justify::Center)
                    .into()
            }
        }
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: WindowEvent,
    ) -> Result<()> {
        match event {
            WindowEvent::HideWindow => {
                *state = WindowState::Hidden;
            }
            WindowEvent::ShowWindow => {
                *state = WindowState::Visible;
            }
        }
        Ok(())
    }
}

impl<Message> WithContainer<Message, Element<Message>> for Window<Message>
    where
        Message: 'static + std::fmt::Debug + Copy + Clone,
{
    fn container(self) -> Container<Message, Element<Message>> {
        let result: Element<Message> = self.into();
        result.container()
    }
}

impl<Message> Into<Element<Message>> for Window<Message>
    where
        Message: 'static + std::fmt::Debug + Copy + Clone,
{
    fn into(self) -> Element<Message> {
        Element::new(Composite::new(self))
    }
}

impl<Message> Into<Element<Message>>
for Composite<WindowEvent, Message, Window<Message>>
    where
        Message: 'static + std::fmt::Debug + Copy + Clone,
{
    fn into(self) -> Element<Message> {
        Element::new(self)
    }
}
