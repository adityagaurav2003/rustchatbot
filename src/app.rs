use std::cell::RefCell;
use std::rc::Rc;

use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use leptos::*;
use leptos_meta::*;

mod components;
use components::chat_area::ChatArea;
use components::type_area::TypeArea;

use crate::model::conversation::{Conversation, Message};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let (dark_mode, _) = create_signal(true);
    provide_context(dark_mode);

    let (conversation, set_conversation) = create_signal(Conversation::new());

    let client: Rc<RefCell<Option<SplitSink<WebSocket, Message>>>> = Default::default();
    let client_clone = client.clone();

    create_effect(move |_| {
        let location = web_sys::window().unwrap().location();
        let hostname = location.hostname().expect("failed to retrieve origin hostname");
        let ws_url = format!("ws://{}:3000/ws", hostname);

        match WebSocket::open(&ws_url) {
            Ok(connection) => {
                let (sender, mut recv) = connection.split();
                spawn_local(async move {
                    while let Some(msg) = recv.next().await {
                        match msg {
                            Ok(Message::Text(msg)) => {
                                set_conversation.update(|c| {
                                    c.messages.last_mut().unwrap().text.push_str(&msg);
                                });
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                });

                *client_clone.borrow_mut() = Some(sender);
            }
            Err(err) => {
                console_error!("Failed to establish WebSocket connection: {}", err);
                // Handle error accordingly, e.g., notify the user or retry connection
            }
        }
    });

    let send = create_action(move |new_message: &String| {
        let user_message = Message {
            text: new_message.clone(),
            user: true,
        };
        set_conversation.update(|c| {
            c.messages.push(user_message);
        });

        let client_clone = client.clone();
        let msg = new_message.clone();
        async move {
            if let Some(sender) = client_clone.borrow_mut().as_mut() {
                if let Err(err) = sender.send(Message::Text(msg)).await {
                    console_error!("WebSocket send error: {}", err);
                    // Handle send error, e.g., retry or notify user
                    Err(ServerFnError::ServerError("WebSocket issue".to_string()))
                } else {
                    Ok(())
                }
            } else {
                Err(ServerFnError::ServerError("WebSocket sender not initialized".to_string()))
            }
        }
    });

    create_effect(move |_| {
        if let Some(_) = send.input().get() {
            let model_message = Message {
                text: String::new(),
                user: false,
            };

            set_conversation.update(|c| {
                c.messages.push(model_message);
            });
        }
    });

    view! {
        <Stylesheet id="leptos" href="/pkg/rusty_llama.css"/>
        <Title text="Rusty Llama"/>
        <ChatArea conversation/>
        <TypeArea send/>
    }
}
