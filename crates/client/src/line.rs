use serde::{Deserialize, Serialize};
use yew::{
    prelude::*, services::ConsoleService, virtual_dom::VNode, Bridge, Callback, Component,
    ComponentLink,
};

use bus;
use bus::LineBus;
use stdweb::{js, unstable::TryInto, Value};

pub struct Line {
    id: u64,

    link: ComponentLink<Self>,
    linebus: Box<Bridge<LineBus>>,
    console: ConsoleService,
    text: String,
    cursor: Option<Vec<u64>>,
    pub on_custom_fn: Callback<(u64, u64)>,
}

impl Line {}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub id: u64,
    pub text: String,
    pub cursor: Option<Vec<u64>>,
    #[props(required)]
    pub on_custom_fn: Callback<(u64, u64)>,
}

#[derive(Deserialize, Serialize)]
pub enum Message {
    Event(bus::Event),
    Test(u64),
}

impl Component for Line {
    type Message = Message;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|bus::Output::Event(sender_id, event)| Message::Event(event));

        let mut agent = bus::LineBus::bridge(callback);
        agent.send(bus::Input::Subscribe(format!("{}", props.id.clone())));

        Self {
            id: props.id,
            link,
            linebus: agent,
            console: ConsoleService::new(),
            text: props.text,
            cursor: props.cursor,
            on_custom_fn: props.on_custom_fn,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Message::Test(_0) => {
                self.on_custom_fn.emit((_0, self.id));
            }
            _ => {}
        }
        true
    }

    fn view(&self) -> VNode {
        let cursor_text = self.text.clone();
        let text_node = match &self.cursor {
            Some(positions) => {
                let (start, end) = cursor_text.split_at((*positions.first().unwrap()) as usize);
                html! {
                    <span><span>{start}</span><span class="cursor"/><span>{end}</span></span>
                }
            }
            None => html! {
                <span>{cursor_text}</span>
            },
        };

        let on_click = self.link.callback(|e: ClickEvent| {
            let page_x = e.client_x();
            let page_y = e.client_y();

            let offset: Value = js! {
                let pageX = @{page_x};
                let pageY = @{page_y};
                var range;
                var textNode;
                var offset;

                if (document.caretPositionFromPoint) {    // standardhttps://uploads-ssl.webflow.com/5929dd19f82ea71d6234fa2d/59efc0274a0dc400019b36cd_shaw-0617-p-500.jpeg
                    range = document.caretPositionFromPoint(pageX, pageY);
                    textNode = range.offsetNode;
                    offset = range.offset;

                } else if (document.caretRangeFromPoint) {    // WebKit
                    range = document.caretRangeFromPoint(pageX, pageY);
                    textNode = range.startContainer;
                    offset = range.startOffset;
                }

                return offset;
            };

            let offset: u64 = offset.try_into().unwrap();
            Message::Test(offset)
        });

        html! {
            <div class="line">
               <div class="gutter">{self.id.clone()}</div>
               <div class="cursors">{text_node.clone()}</div>
               <div class="code" onclick={on_click}>{text_node}</div>
            </div>
        }
    }
}
