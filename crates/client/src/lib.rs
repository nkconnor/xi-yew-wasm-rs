#![recursion_limit = "256"]
extern crate serde;
extern crate serde_json;
extern crate yew;
extern crate stdweb;
extern crate wasm_bindgen;
extern crate failure;
extern crate zn_core;

use failure::Error;
use yew::prelude::*;
use stdweb::web::Date;
use yew::{
    services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask},
    services::ConsoleService,
    services::keyboard::KeyboardService,
    html,
    Component,
    ComponentLink,
    Html,
    ShouldRender
};


use wasm_bindgen::prelude::*;
use yew::format::{Json, Text};
use zn_core::messages::ClientStartedParams;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::start_app::<Model>();
    Ok(())
}

pub struct Model {
    link: ComponentLink<Self>,
    console: ConsoleService,
    ws: Option<WebSocketTask>,
    wss: WebSocketService,
    keyboard: KeyboardService,
    value: i64,
}

pub enum Msg {
    Increment,
    Decrement,
    Bulk(Vec<Msg>),
    WSConnect,
    WSReceived(Result<String, failure::Error>), // data received from server
    WS(WebSocketStatus)
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model {
            link,
            console: ConsoleService::new(),
            ws: None,
            wss: WebSocketService::new(),
            keyboard: KeyboardService{},
            value: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Increment => {
                self.value = self.value + 25;
                self.console.log("plus 25");
            }
            Msg::Decrement => {
                self.value = self.value - 150;
                let msg = zn_core::messages::ClientMessage::ClientStarted { params: ClientStartedParams { client_extras_dir: None, config_dir: None}};
                let test_message = Json(&msg);
                match self.ws {
                    Some(ref mut task) => {
                        task.send(test_message);
                    },
                    _ => ()
                }

                self.console.log("minus one");
            }
            Msg::Bulk(list) => {
                for msg in list {
                    self.update(msg);
                    self.console.log("Bulk action");
                }
            },

            Msg::WSConnect => {
                self.console.log("Connecting");
                let cbout = self.link.callback(|Json(data)| Msg::WSReceived(data));

                let cbnot = self.link.callback(|input| {
                    Msg::WS(input)
                });
                if self.ws.is_none() {
                    let task = self.wss.connect("ws://127.0.0.1:8080/ws/", cbout, cbnot.into());
                    self.ws = Some(task.unwrap());
                }
                self.console.log("Done connecting");
            },
            Msg::WS(WebSocketStatus::Opened) => {
                self.console.log("Opened socket")
            },
            Msg::WSReceived(Ok(message)) => {
                self.console.log(format!("Client received message {}", message).as_str())
            },
            Msg::WSReceived(Err(e)) => {
                self.console.log(format!("Client received error {:?}", e).as_str())
            },
            Msg::WS(WebSocketStatus::Closed) => {

                self.console.log("Socket closed!")
            }
            Msg::WS(WebSocketStatus::Error) => {
                self.console.log("Socket error!")
            }
        }
        true
    }


    fn view(&self) -> Html {
        html! {
            <div>
                <nav class="menu">
                    <button onclick=self.link.callback(|_| Msg::Increment)>
                        { "Increment Me" }
                    </button>
                    <button onclick=self.link.callback(|_| Msg::Decrement)>
                        { "Decrement" }
                    </button>
                    <button onclick=self.link.batch_callback(|_| vec![Msg::Increment, Msg::Increment, Msg::WSConnect])>
                        { "Increment Twice" }
                    </button>
                </nav>
                <p>{ self.value }</p>
                <p>{ Date::new().to_string() }</p>
            </div>
        }
    }
}