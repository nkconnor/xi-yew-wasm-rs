use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use yew::agent::HandlerId;
use yew::worker::*;

// .. grr couldn't get generics to work here
// this should be refactorable to take generic event messages
// LineBus<T>

pub struct LineBus {
    link: AgentLink<Self>,
    subscribers: HashMap<ChannelId, Vec<HandlerId>>,
}

pub type Subscribers = HashMap<ChannelId, Vec<HandlerId>>;
pub type ChannelId = String;
pub type SenderId = HandlerId;

#[derive(Serialize, Deserialize)]
pub enum Input {
    Subscribe(ChannelId),
    Event(ChannelId, Event),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Event {
    Insert {
        cursor: Option<Vec<u64>>,
        text: String,
    },
}

#[derive(Serialize, Deserialize)]
pub enum Output {
    Event(SenderId, Event),
}

impl Agent for LineBus {
    type Reach = Context;
    type Message = ();
    type Input = Input;
    type Output = Output;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: Subscribers::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        ()
    }

    /// Since the parent has a ref to the child's private
    /// agent, this simply passes the message forward
    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        let sender_id = id;
        match msg {
            Input::Subscribe(channel_id) => {
                self.subscribers
                    .entry(channel_id)
                    .or_insert_with(Vec::new)
                    .push(sender_id);
            }
            Input::Event(channel_id, event) => {
                if let Some(subscribers) = self.subscribers.get(&channel_id) {
                    subscribers.iter().for_each(|s| {
                        self.link
                            .respond(*s, Output::Event(sender_id, event.clone()))
                    })
                }
            }
        }
    }
}
