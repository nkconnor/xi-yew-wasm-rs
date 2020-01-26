use socket::*;
use yew::{
    prelude::*, services::ConsoleService, virtual_dom::VNode, Component, ComponentLink, Properties,
};
use zn_core::{
    messages,
    messages::{
        ClientMessage, Edit, GestureParams, GestureType, Method, ServerMessage, UpdateUpdateParams,
    },
};
use {bus, socket};

use bus::Output;
use line::Line;
use view::Message::Nothing;

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub id: String,
}

pub enum Message {
    Apply(Method),
    Click(u64, u64),
    Nothing,
}

pub struct View {
    id: String,
    link: ComponentLink<Self>,
    console: ConsoleService,
    socket: Box<dyn Bridge<socket::Mediary>>,
    linebus: Box<dyn Bridge<bus::LineBus>>,
    lines: Vec<messages::Line>,
}

impl Component for View {
    type Message = Message;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|Receive::Forward(server_message)| {
            if let ServerMessage::EditorMethod(Method::Update { update, view_id }) = server_message
            {
                Message::Apply(Method::Update { update, view_id })
            } else {
                Message::Nothing
            }
        });
        // `Worker::bridge` spawns an instance if no one is available
        let mut socket = socket::Mediary::bridge(callback); // Connected! :tada:
        socket.send(socket::Send::SubscribeToView(props.id.clone()));

        let linebus = bus::LineBus::bridge(link.callback(|Output::Event(_, _)| Nothing));

        View {
            id: props.id,
            link,
            console: ConsoleService::new(),
            socket,
            linebus,
            lines: Vec::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        let should_render: bool = match msg {
            Message::Nothing => false,
            Message::Click(line, pos) => {
                self.socket.send(Send::Forward(ClientMessage::Edit {
                    params: Edit::Gesture {
                        params: GestureParams {
                            line: line,
                            col: pos as u64,
                            ty: GestureType::PointSelect,
                        },
                        view_id: "view-id-1".to_string(),
                    },
                }));

                false
            }
            Message::Apply(Method::Update {
                update:
                    UpdateUpdateParams {
                        annotations,
                        ops,
                        pristine,
                    },
                view_id,
            }) => {
                // alrighty.. current issue is if we publish to the event bus
                // originally, and the line hasn't even mounted yet,
                // the line won't get the message.
                // if   !self.lines_rendered { self.lines = such and such }
                // else linebus.send(ops)

                self.console
                    .info(&format!("Got lines of length {}", self.lines.len()));

                if self.lines.len() == 0 {
                    self.lines = ops
                        .into_iter()
                        .flat_map(|op| op.lines)
                        .flatten()
                        .collect::<Vec<messages::Line>>();

                    true
                } else {
                    ops.into_iter()
                        .flat_map(|op| op.lines)
                        .flatten()
                        .for_each(|line| {
                            self.linebus.send(bus::Input::Event(
                                String::from(format!("{}", line.ln)),
                                bus::Event::Insert {
                                    cursor: None,
                                    text: line.text.clone(),
                                },
                            ))
                        });

                    false
                }

                // so we either need to seed the line with some text
                // or mount it before we get this far

                // also how do we avoid rendering in general..

                //let text = update.ops.into_iter()
                //    .flat_map(|op| {
                //        op.lines
                //    }).flatten().collect::<Vec<Line>>();
                //
                //self.lines = text;
            }
            _ => false,
        };

        should_render
    }

    fn view(&self) -> VNode {
        html! {
            <div>
                {
                    for self.lines.iter().map(|line| {

                        html! {
                            <Line
                                id={line.ln.clone()}
                                on_custom_fn={self.link.callback(|(ln, pos)| Message::Click(ln, pos))}
                                text={line.text.clone()}
                                cursor={line.cursor.clone()}
                            />
                        }
                    })
                }
            </div>
        }
    }
}
