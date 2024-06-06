use std::collections::VecDeque;

use futures_util::StreamExt;
use leptos::{ev::KeyboardEvent, *};

use crate::{
    markdown::Markdown,
    server_functions::{
        chat,
        models::{AppData, ProjectData},
    },
    wasm::{
        components::icons::*,
        types::{ChatMessage, ChatUser},
    },
};

#[component]
pub fn ChatPanel(project: ProjectData) -> impl IntoView {
    let Some(app_data) = use_context::<AppData>() else {
        return view! { <div></div> };
    };

    let (open, set_open) = create_signal(false);
    let (expanded, set_expanded) = create_signal(false);
    let (input, set_input) = create_signal(String::new());
    let (messages, set_messages) = create_signal::<VecDeque<ChatMessage>>(VecDeque::new());

    let on_submit = move || {
        let user = app_data.user.clone();
        let prompt = input.get().to_owned();
        set_input.set(String::new());

        spawn_local(async move {
            let mut stream = chat(project.id, project.version, prompt.to_owned())
                .await
                .expect("Failed to start chat stream")
                .into_inner();

            let user_message = ChatMessage {
                user: ChatUser::User(user.given_name.clone()),
                content: create_rw_signal(prompt),
                key: uuid::Uuid::new_v4().as_u128(),
            };

            let bot_message = ChatMessage {
                user: ChatUser::Assistant,
                content: create_rw_signal(String::new()),
                key: uuid::Uuid::new_v4().as_u128(),
            };

            set_messages.update(|messages| {
                messages.push_front(user_message);
                messages.push_front(bot_message);
            });

            while let Some(Ok(chunk)) = stream.next().await {
                let mut message = String::new();

                let parts: Vec<&str> = chunk.split("¤¤").collect();
                for part in parts {
                    message.push_str(part.trim_start_matches("data: "));
                }

                set_messages.update(|messages| {
                    if let Some(m) = messages.front_mut() {
                        m.content
                            .update(|content| content.push_str(message.as_str()));
                    }
                });
            }
        });
    };

    let textarea_ref = create_node_ref::<html::Textarea>();
    create_effect(move |_| {
        let textarea = textarea_ref.get().expect("Failed to get textarea");
        if open.get() || expanded.get() {
            let _ = textarea.focus();
        } else {
            let _ = textarea.blur();
        }
    });

    let on_input = move |e: KeyboardEvent| {
        if e.key().eq("Enter") && !e.shift_key() {
            e.prevent_default();
            on_submit();
        }
    };

    view! {
        <div
            id="chat-panel"
            class=move || if expanded.get() {"expanded"} else if open.get() {"open"} else {"-bottom-[calc(24rem-3.5rem)]"}
        >
            <div id="chat-header" class="flex justify-between items-center w-full h-14 px-4 bg-[#101010] rounded-t-lg">
                <button
                    id="chat-expand"
                    class="w-6 h-6 rotate-90"
                    on:click=move |_| set_expanded.update(|x| *x = !*x)
                >
                    <ExpandIcon />
                </button>
                <button
                    id="chat-open"
                    class="h-full"
                    on:click=move |_| set_open.update(|x| *x = !*x)
                >
                    <h4>"Chat about "<b>{project.name}</b></h4>
                </button>
                <div class="w-6 h-6">
                    <button id="chat-options" class="w-full hidden">
                        <TriangleDownIcon />
                    </button>
                </div>
            </div>
            <div class="flex justify-center w-full h-[calc(100%-3.5rem)] bg-[#212121] border-1 border-base">
                <div class="flex flex-col w-full h-full pb-2">
                    <div id="chat-messages" class="flex flex-col-reverse flex-auto overflow-y-auto">
                        <For
                            each=move || messages.get()
                            key=|state| state.key
                            let:child
                        >
                            <div class="message">
                                {match child.user {
                                    ChatUser::User(name) => view! {
                                        <>
                                            <div class="message-header">
                                                <PersonCircleIcon size="20px" />
                                                <span class="message-user">{name}</span>
                                            </div>
                                            <div class="message-body">{child.content}</div>
                                        </>
                                    },
                                    ChatUser::Assistant => view! {
                                        <>
                                            <div class="message-header">
                                                <BotIcon size="20px" />
                                                <span class="message-user">"Magic Docs"</span>
                                            </div>
                                            <div
                                                class="message-body"
                                                inner_html=move || Markdown::to_html(&child.content.get())
                                            ></div>
                                        </>
                                    },
                                }}
                            </div>
                        </For>
                    </div>
                    <div class="input-area flex-initial mx-auto w-[calc(100%-16px)] max-w-[50rem]">
                        <textarea
                            on:keydown=on_input
                            on:input=move |e| set_input.set(event_target_value(&e))
                            prop:value=input
                            id="chat-input"
                            name="message"
                            class="w-full px-4 py-2 bg-[#343434] text-white resize-none rounded-md"
                            placeholder="Type a message..."
                            rows="1"
                            ref=textarea_ref
                        ></textarea>
                    </div>
                </div>
            </div>
        </div>
    }
}
