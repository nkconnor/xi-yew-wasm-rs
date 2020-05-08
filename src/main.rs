extern crate simple_logger;
extern crate web_view;
extern crate zn;

use async_std::task;
use log::info;
use std::error::Error;
use web_view::WVResult;
use zn::start_websocket_server;

fn main() -> Result<(), web_view::Error> {
    //
    simple_logger::init().unwrap();

    info!("Booting up");
    // -- 0) Auto-reload for client/server in dev mode
    // -- 0,1) Use native app instead of web tab
    // -- 1) wire in Yew
    // -- 2) attach WebSocketService
    // -- 3) spawn server in separate thread w/ Socket reader
    // -- 4) port message handling and XI-Server over to here
    // 5a) send view message
    // 5b) display lines
    // 6) send edit, e.g. delete on backspace
    let html_content = r#"<!doctype html>
<html lang="en">

<head>
    <style>

        html, body {
            background-color: #1a1d21;
            margin:0;
            padding:0;
        }

        .gutter {
            background-color: #19171d;
        }

        .line {
           font-family: monospace, monospace;
        }

        .line .gutter {
            padding: 20px;
            margin-right:10px;
            font-size:0.8rem;
            border-right: 1px black;
            display: inline-block;
            -webkit-touch-callout: none;
            -webkit-user-select: none;
            -khtml-user-select: none;
            -moz-user-select: none;
            -ms-user-select: none;
            user-select: none;
        }

        .line .code {
            display: inline-block;
        }
        
        .cursors {
            position: absolute;
            z-index:10;
            color: transparent;
        }
        
        .cursor {
            display:inline-block;
            width: 4px;
            height: 1rem;
            background:white;
        }
    </style>
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
