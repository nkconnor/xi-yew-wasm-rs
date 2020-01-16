extern crate zn;
extern crate web_view;
extern crate simple_logger;

use std::error::Error;
use web_view::WVResult;
use zn::start_websocket_server;
use async_std::task;
use log::info;

fn main() -> Result<(), web_view::Error> { //
    simple_logger::init().unwrap();

    info!("Booting up");
    // -- 0) Auto-reload for client/server in dev mode
    // -- 0,1) Use native app instead of web tab
    // -- 1) wire in Yew
    // 2) attach WebSocketService
    // 3) spawn server in separate thread w/ Socket reader
    // 4) port message handling and XI-Server over to here
    // 5) display lines in Yew using Agents
    let html_content = r#"<!doctype html>
<html lang="en">

<head>
    <meta charset="utf-8" />
    <title>Yew</title>
    <script src="http://localhost:8085/pkg/bundle.js" defer></script>
</head>

<body>
</body>

</html>"#;

    std::thread::spawn(|| task::block_on(start_websocket_server()));

    web_view::builder()
        .title("Zinc")
        .content(web_view::Content::Html(html_content))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| {
            // _webview.set_fullscreen(true);
            Ok(())
        })
        .run()


    //yew::start_app::<zn_client::Model>();
}

