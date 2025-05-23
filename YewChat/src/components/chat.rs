use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        html! {
            <div class="flex w-screen h-screen bg-gradient-to-br from-purple-50 to-blue-50">
                // Sidebar untuk daftar pengguna
                <div class="flex-none w-72 h-screen bg-gradient-to-b from-indigo-600 to-purple-700 shadow-2xl">
                    <div class="text-2xl font-bold p-4 text-white border-b border-indigo-500/30">
                        <div class="flex items-center gap-3">
                            <div class="w-3 h-3 bg-green-400 rounded-full animate-pulse"></div>
                            {"👥 Pengguna Online"}
                        </div>
                    </div>
                    <div class="p-4 space-y-3">
                        {
                            self.users.clone().iter().map(|u| {
                                html!{
                                    <div class="flex items-center gap-3 bg-white/10 backdrop-blur-sm rounded-xl p-3 hover:bg-white/20 transition-all duration-200 cursor-pointer border border-white/20">
                                        <div class="relative">
                                            <img class="w-12 h-12 rounded-full border-2 border-white/50" src={u.avatar.clone()} alt="avatar"/>
                                            <div class="absolute -bottom-1 -right-1 w-4 h-4 bg-green-400 rounded-full border-2 border-white"></div>
                                        </div>
                                        <div class="flex-grow">
                                            <div class="text-white font-semibold text-sm">
                                                {u.name.clone()}
                                            </div>
                                            <div class="text-indigo-200 text-xs">
                                                {"Sedang aktif"}
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </div>
                
                // Area chat utama
                <div class="grow h-screen flex flex-col">
                    // Header chat
                    <div class="w-full h-16 bg-white shadow-lg border-b border-gray-200">
                        <div class="flex items-center h-full px-6">
                            <div class="text-2xl">{"💬"}</div>
                            <div class="ml-3">
                                <div class="text-xl font-bold text-gray-800">{"Ruang Obrolan"}</div>
                                <div class="text-sm text-gray-500">
                                    {format!("{} pengguna aktif", self.users.len())}
                                </div>
                            </div>
                        </div>
                    </div>
                    
                    // Area pesan
                    <div class="w-full grow overflow-auto p-4 space-y-4 bg-gradient-to-b from-gray-50 to-white">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                html!{
                                    <div class="flex items-start gap-3 max-w-4xl">
                                        <img class="w-10 h-10 rounded-full border-2 border-indigo-200 shadow-sm" src={user.avatar.clone()} alt="avatar"/>
                                        <div class="bg-white rounded-2xl rounded-tl-md shadow-md p-4 border border-gray-100 flex-grow">
                                            <div class="flex items-center gap-2 mb-2">
                                                <div class="text-sm font-semibold text-indigo-600">
                                                    {m.from.clone()}
                                                </div>
                                                <div class="text-xs text-gray-400">
                                                    {"baru saja"}
                                                </div>
                                            </div>
                                            <div class="text-gray-700">
                                                if m.message.ends_with(".gif") {
                                                    <img class="mt-2 rounded-lg max-w-sm shadow-sm" src={m.message.clone()} alt="GIF"/>
                                                } else {
                                                    <div class="break-words">
                                                        {m.message.clone()}
                                                    </div>
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    
                    // Input area
                    <div class="w-full bg-white border-t border-gray-200 shadow-lg">
                        <div class="flex items-center gap-4 p-4">
                            <input 
                                ref={self.chat_input.clone()} 
                                type="text" 
                                placeholder="Ketik pesan Anda di sini..." 
                                class="flex-grow py-3 px-4 bg-gray-50 border border-gray-200 rounded-full outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all duration-200 text-gray-700 placeholder-gray-400"
                                name="message" 
                                required=true 
                            />
                            <button 
                                onclick={submit} 
                                class="p-3 bg-gradient-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700 w-12 h-12 rounded-full flex justify-center items-center shadow-lg hover:shadow-xl transition-all duration-200 transform hover:scale-105"
                            >
                                <svg fill="#ffffff" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="w-5 h-5">
                                    <path d="M0 0h24v24H0z" fill="none"></path>
                                    <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}