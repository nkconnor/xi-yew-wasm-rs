use serde::{Deserialize, Serialize};
use yew::format::Json;
use yew::services::websocket::{WebSocketStatus, WebSocketTask};
use yew::services::{ConsoleService, WebSocketService};
use yew::worker::*;
use zn_core::messages::{ClientMessage, ClientStartedParams, Method, ServerMessage, ViewId};

pub struct ViewSubscriber {
    handler_id: HandlerId,
    view_id: ViewId,
}

#[derive(Serialize, Deserialize)]
pub enum Send {
    Subscribe,
    SubscribeToView(ViewId),
    Forward(ClientMessage),
}

#[derive(Serialize, Deserialize)]
pub enum Receive {
    Forward(ServerMessage),
}

pub struct Mediary {
    link: AgentLink<Mediary>,
    task: WebSocketTask,
    console: ConsoleService,
    subscriber: Option<HandlerId>,
    view_subscribers: Vec<ViewSubscriber>,
}

pub enum Callback {
    Receive(ServerMessage),
    Status(WebSocketStatus),
}

impl Agent for Mediary {
    // Available:
    // - `Job` (one per bridge on the main thread)
    // - `Context` (shared in the main thread)
    // - `Private` (one per bridge in a separate thread)
    // - `Public` (shared in a separate thread)
    type Reach = Context;
    type Message = Callback;
    // Spawn only one instance on the main thread (all components can share this agent)
    type Input = Send;
    type Output = Receive;

    //// Called when a new bridge connects
    //fn connected(&mut self, _id: HandlerId) {
    //    self.handlers.push(_id)
    //}

    // Create an instance with a link to the agent.
    fn create(link: AgentLink<Self>) -> Self {
        let receive = link.callback(|Json::<Result<ServerMessage, failure::Error>>(data)| {
            //self.console.log(format!("Received cbout {:?}", &data).as_str());
            //let msg_txt = data.expect("Message parses as json, then ServerMessage");
            Callback::Receive(data.unwrap())
        });

        let send = link.callback(|input| Callback::Status(input));

        let mut socket_service = WebSocketService::new();

        if let Ok(task) = socket_service.connect("ws://127.0.0.1:8080/ws/", receive, send.into()) {
            Mediary {
                link,
                task,
                console: ConsoleService::new(),
                subscriber: None,
                view_subscribers: Vec::new(),
            }
        } else {
            panic!("Socket service couldn't connect to the server!")
        }
    }

    // Handle inner messages (from callbacks)
    fn update(&mut self, msg: Self::Message) {
        match msg {
            Callback::Receive(server_message) => {
                // self.views.get(id) orElse ..
                self.console
                    .log(&format!("Socket service received: {:?}", server_message));
                //self.handlers.iter().for_each(|h| self.link.respond(*h, server_message.clone()));
                match server_message {
                    ServerMessage::EditorMethod(Method::Update { update, view_id }) => {
                        if let Some(subscriber) =
                            self.view_subscribers.iter().find(|s| s.view_id == view_id)
                        {
                            self.link.respond(
                                subscriber.handler_id,
                                Receive::Forward(ServerMessage::EditorMethod(Method::Update {
                                    update,
                                    view_id,
                                })),
                            );
                        }
                    }
                    _ => {
                        if let Some(subscriber) = self.subscriber {
                            self.link
                                .respond(subscriber, Receive::Forward(server_message))
                        }
                    }
                }
            }
            Callback::Status(WebSocketStatus::Opened) => {
                self.task.send(Json(&ClientMessage::ClientStarted {
                    params: ClientStartedParams {
                        client_extras_dir: None,
                        config_dir: None,
                    },
                }));
            }
            Callback::Status(status) => {
                self.console.log(&format!("Socket status: {:?}", status));
            }
        }
    }

    // Handle incoming messages from components of other agents.
    fn handle_input(&mut self, msg: Self::Input, who: HandlerId) {
        match msg {
            Send::Forward(server_message) => self.task.send(Json(&server_message)),
            Send::SubscribeToView(view_id) => self.view_subscribers.push(ViewSubscriber {
                handler_id: who,
                view_id,
            }),
            Send::Subscribe => self.subscriber = Some(who),
        }
    }
}
