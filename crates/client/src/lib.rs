#![recursion_limit = "5120"]
extern crate failure;
extern crate serde;
extern crate serde_json;
extern crate stdweb;
extern crate wasm_bindgen;
extern crate yew;
extern crate zn_core;

use stdweb::web::Date;
use yew::prelude::*;
use yew::{
    html, services::keyboard::KeyboardService, services::ConsoleService, Component, ComponentLink,
    Html, ShouldRender,
};

use socket::*;
use wasm_bindgen::prelude::*;
use zn_core::messages::*;

pub mod bus;
pub mod line;
pub mod socket;
pub mod view;

use crate::view::View;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::start_app::<Model>();
    Ok(())
}

pub struct Model {
    link: ComponentLink<Self>,
    console: ConsoleService,
    socket: Box<Bridge<socket::Mediary>>,
    keyboard: KeyboardService,
    views: Vec<ViewId>,
    value: Vec<Line>,
}

pub enum Msg {
    OpenFile,
    WSReceived(ServerMessage),
    Empty,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        //link.callback(|server_message| {

        //    match server_message {
        //        ServerMessage::Pong{v} => (),
        //        ServerMessage::EditorNotification(Notification::Result{id, result}) => {
        //            ()
        //        }
        //        // we need an agent to forward the message to the view
        //        ServerMessage::EditorMethod(Method::Update {update, view_id}) => {
        //            // view_id_bridge_send ( update )
        //            ()
        //        },
        //        _ => {}
        //    };

        //    Msg::WSReceived(server_message)
        //});
        let callback =
            link.callback(|Receive::Forward(server_message)| Msg::WSReceived(server_message));
        // `Worker::bridge` spawns an instance if no one is available
        let mut socket = socket::Mediary::bridge(callback); // Connected! :tada:
        socket.send(Send::Subscribe);

        Model {
            link,
            console: ConsoleService::new(),
            socket: socket,
            keyboard: KeyboardService {},
            views: Vec::new(),
            value: Vec::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::WSReceived(ServerMessage::EditorNotification(Notification::Result {
                id: _,
                result,
            })) => {
                // New View is ready, open an empty browser tab
                self.console.log("Adding new view");
                self.views.push(result);
            }
            Msg::OpenFile => {
                self.socket.send(Send::Forward(ClientMessage::NewView {
                    id: 0,
                    params: NewViewParams {
                        file_path: Some(String::from("/home/nconnor/p/zn/zn/build.rs")),
                    },
                }));
            }
            _ => {}
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <nav class="menu">
                    <button onclick=self.link.callback(|_| Msg::OpenFile)>
                        { "Send New View" }
                    </button>
                </nav>
                <div>
                    {
                        for self.views.iter().map(|id| {
                            html! {
                                <div><span></span>
                                <View id={id} />
                                </div>
                            }
                        })
                    }
                </div>
                <p>{ Date::new().to_string() }</p>
            </div>
        }
    }
}
