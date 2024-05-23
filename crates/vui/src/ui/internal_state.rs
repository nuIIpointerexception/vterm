use ::std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::ui::Id;

pub struct InternalState {
    widget_states: HashMap<Id, Box<dyn Any>>,
}

impl InternalState {
    pub fn new() -> Self {
        Self {
            widget_states: HashMap::new(),
        }
    }

    pub fn get_state<S>(&mut self, id: &Id) -> &S
        where
            S: Default + 'static,
    {
        self.check_missing::<S>(id);

        self.widget_states.get(id).unwrap().downcast_ref().unwrap()
    }

    pub fn get_state_mut<S>(&mut self, id: &Id) -> &mut S
        where
            S: Default + 'static,
    {
        self.check_missing::<S>(id);

        self.widget_states
            .get_mut(id)
            .unwrap()
            .downcast_mut()
            .unwrap()
    }

    fn check_missing<S>(&mut self, id: &Id)
        where
            S: Default + 'static,
    {
        let needs_insert = if let Some(state) = self.widget_states.get(id) {
            let wrong_type = state.as_ref().type_id() != TypeId::of::<S>();
            if wrong_type {
                log::error!(
                    "Unable to downcast existing Widget state for {:?}! \
                    Are your UI IDs unique? Expected {:?} but found {:?}",
                    id,
                    TypeId::of::<S>(),
                    state.type_id(),
                );
            }
            wrong_type
        } else {
            true
        };

        if needs_insert {
            log::trace!("Creating new state for {:?}", id);
            self.widget_states.insert(*id, Box::new(S::default()));
        }
    }
}
