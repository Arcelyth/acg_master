use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{Message, futures::WebSocket};
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::time::Duration;
use stylance::import_crate_style;
use web_sys::window;

import_crate_style!(styles, "./src/pages/styles/multi.module.scss");

use crate::bangumi::anime::*;
use crate::components::{back_btn::BackBtn, card::Card};
use crate::config::{Config, Language};
use crate::ws::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GameState {
    Lobby, // before matching
    Matching,
    Loading,
    Playing,
    Win,
    Lose,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ChatSide {
    I,
    O,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatEntry {
    side: ChatSide,
    content: String,
}

#[component]
pub fn Multi() -> impl IntoView {
    let (game_state, set_game_state) = signal(GameState::Lobby);
    // for chat
    let (chat_log, set_chat_log) = signal::<Vec<ChatEntry>>(vec![]);
    let (username, set_username) = signal("".to_string());
    let (text, set_text) = signal("".to_string());
    let config = use_context::<ReadSignal<Config>>().expect("reader");

    let ws_sender = StoredValue::new(None::<futures::channel::mpsc::UnboundedSender<Message>>);

    // timer
    let (elapsed_seconds, set_elapsed_seconds) = signal(0u64);
    let (is_timer_running, set_is_timer_running) = signal(false);
    let (current_config, set_current_config) = signal(config.get_untracked());
    let (p2_name, set_p2) = signal::<String>("".to_string());

    let (user_input, set_user_input) = signal("".to_string());
    let (debounced_input, set_debounced_input) = signal("".to_string());
    let (input_version, set_input_version) = signal(0);

    let (input_focused, set_input_focused) = signal(false);
    let (selected_dropdown_index, set_selected_dropdown_index) = signal(0usize);

    let (guess_time, set_guess_time) = signal(0usize);

    let (cards, set_cards) = signal::<Vec<(BangumiSubject, CompareResult)>>(vec![]);
    let (_refresh_trigger, set_refresh_trigger) = signal(0);
    let (answer, set_answer) = signal(None);

    let search_results = LocalResource::new(move || bangumi_search(debounced_input.get()));

    Effect::new(move |_| {
        let state = game_state.get();
        if state == GameState::Win || state == GameState::Lose {
            set_is_timer_running.set(false);
        }
    });

    // Loading -> Playing
    Effect::new(move |_| {
        if game_state.get() == GameState::Loading {
            spawn_local(async move {
                let success = anime_start_game(current_config.get_untracked()).await;
                if success {
                    set_game_state.set(GameState::Playing);
                }
            });
        }
    });

    // debounce
    Effect::new(move |_| {
        let current_text = user_input.get();
        set_input_version.update(|v| *v += 1);
        let current_version = input_version.get_untracked();

        spawn_local(async move {
            TimeoutFuture::new(500).await;
            if input_version.get_untracked() == current_version {
                set_debounced_input.set(current_text);
                set_selected_dropdown_index.set(0);
            }
        });
    });

    let formatted_time = move || {
        let s = elapsed_seconds.get();
        format!("{:02}:{:02}", s / 60, s % 60)
    };

    use_interval(1000, move || {
        if is_timer_running.get() {
            set_elapsed_seconds.update(|sec| *sec += 1);
        }
    });

    // TODO:
    let send_text = move |_| {
        let current_msg = text.get_untracked();
        if current_msg.is_empty() {
            return;
        }

        set_chat_log.update(|v| {
            v.push(ChatEntry {
                side: ChatSide::I,
                content: current_msg.clone(),
            });
        });

        if let Some(tx) = ws_sender.get_value() {
            let msg = ClientMsg::Message(current_msg.clone());

            if let Ok(text) = serde_json::to_string(&msg) {
                let _ = tx.unbounded_send(Message::Text(text));
            }

            set_text.set("".to_string());
        }
    };

    let connect = move |_| {
        let sender = connect_ws(move |msg| {
            println!("recv: {}", msg);

            if let Ok(server_msg) = serde_json::from_str::<ServerMsg>(&msg) {
                match server_msg {
                    ServerMsg::JoinSucc(name1, name2) => {
                        set_game_state.set(GameState::Playing);
                        if name1 == username.get() {
                            set_p2.set(name2);
                        } else {
                            set_p2.set(name1);
                        }
                    }
                    ServerMsg::Response(m) => {
                        set_chat_log.update(|v| {
                            v.push(ChatEntry {
                                side: ChatSide::O,
                                content: m,
                            });
                        });
                    }
                    ServerMsg::GuessResp(WsGuessResponse { guess, comparison }) => {
                        set_cards.update(|c| c.push((guess, comparison)));
                    }
                    ServerMsg::Over(win, answer) => {
                        if win {
                            set_game_state.set(GameState::Win);
                        } else {
                            set_game_state.set(GameState::Win);
                        }
                        set_answer.set(Some(answer));
                    }
                    _ => {}
                }
            }
        });
        let join_msg = ClientMsg::Join(username.get());
        if let Ok(text) = serde_json::to_string(&join_msg) {
            let _ = sender.unbounded_send(Message::Text(text));
        }

        ws_sender.set_value(Some(sender));
    };

    let start_match = move |_| {
        let name_len = username.get().trim().len();
        if name_len < 1 || name_len > 20 {
            return;
        }

        set_game_state.set(GameState::Matching);

        connect(());
    };

    let texts = move || match config.get().lang {
        Language::Chinese => (
            "输入名称",
            "开始匹配",
            "匹配中......",
            "输入动漫名称",
            "你赢了",
            "你输了",
        ),
        Language::English => (
            "Input your name",
            "Start matching",
            "Matching...",
            "Input anime's name",
            "You Win",
            "You Lose",
        ),
    };

    let unique_search_results = move || {
        let res = search_results.get().flatten().unwrap_or_default();
        let mut seen = HashSet::new();
        res.into_iter()
            .filter(|item| !item.name_cn.is_empty())
            .filter(|item| seen.insert(item.name_cn.clone()))
            .collect::<Vec<_>>()
    };

    let send_guess = move || {
        let items = unique_search_results();
        if items.is_empty() {
            return;
        }

        let current_idx = selected_dropdown_index.get_untracked();
        let target = items.get(current_idx).or(items.first()).cloned();

        if let Some(subject) = target {
            let exists = cards
                .get_untracked()
                .iter()
                .any(|(c, _)| c.id == subject.id);
            if exists {
                return;
            }

            if cards.get_untracked().is_empty() {
                set_is_timer_running.set(true);
            }

            set_user_input.set("".to_string());
            set_selected_dropdown_index.set(0);

            spawn_local(async move {
                if let Some(tx) = ws_sender.get_value() {
                    let msg = ClientMsg::Guess(subject.clone());

                    if let Ok(text) = serde_json::to_string(&msg) {
                        let _ = tx.unbounded_send(Message::Text(text));
                    }

                    set_text.set("".to_string());
                }
            });
        }
    };

    let on_keydown = move |ev: leptos::web_sys::KeyboardEvent| {
        let items = unique_search_results();
        if items.is_empty() {
            return;
        }

        let max_idx = items.len().saturating_sub(1);
        let current = selected_dropdown_index.get_untracked();

        match ev.key().as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                let next = if current >= max_idx { 0 } else { current + 1 };
                set_selected_dropdown_index.set(next);
            }
            "ArrowUp" => {
                ev.prevent_default();
                let prev = if current == 0 { max_idx } else { current - 1 };
                set_selected_dropdown_index.set(prev);
            }
            "Enter" => {
                ev.prevent_default();
                if let Some(item) = items.get(current) {
                    set_user_input.set(item.name_cn.clone());
                }
            }
            _ => {}
        }
    };

    let is_interaction_disabled = move || game_state.get() != GameState::Playing;

    let reset_game = move |_| {
        set_cards.set(vec![]);
        set_guess_time.set(0);
        set_user_input.set("".to_string());

        set_game_state.set(GameState::Loading);

        set_current_config.set(config.get_untracked());

        set_is_timer_running.set(false);
        set_elapsed_seconds.set(0);

        set_refresh_trigger.update(|n| *n += 1);
    };

    let reset_icon = move || {
        view! {
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
            </svg>
        }
    };

    view! {
        <ErrorBoundary fallback=|errors| {
            view! {
                <h1>"Uh oh! Something went wrong!"</h1>
                <ul>
                    {move || errors.get().into_iter().map(|(_, e)| view! { <li>{e.to_string()}</li> }).collect_view()}
                </ul>
            }
        }>
            <main>
                <div class=styles::top_section>
                    <BackBtn />
                </div>

                <Show when=move || game_state.get() == GameState::Lobby || game_state.get() == GameState::Matching>
                    <div class=styles::lobby_section>
                        <input
                            class=styles::username_input
                            placeholder=texts().0
                            bind:value=(username, set_username)
                        />
                        <button class=styles::match_btn on:click=start_match disabled=move || game_state.get() == GameState::Matching>
                            {move || if game_state.get() == GameState::Matching { texts().2 } else { texts().1 }}
                        </button>
                    </div>
                </Show>
                <Show when=move || game_state.get() != GameState::Lobby && game_state.get() != GameState::Matching>
                      // show names
                      <div class=styles::player_panel>
                          <div class=styles::player_me>
                              {move || username.get()}
                          </div>
                          <div class=styles::player_other>
                              {move || p2_name.get()}
                          </div>
                      </div>
                  // interact section
                   <div class=styles::interact_section>
                      <div class=styles::search_wrapper>
                         <div class=styles::input_section>
                              <span> {move || texts().1}: </span>

                              <div class=styles::input_container>
                                  <input
                                      placeholder={move || texts().3}
                                      type="text"
                                      disabled=is_interaction_disabled
                                      bind:value=(user_input, set_user_input)
                                      on:focus=move |_| set_input_focused.set(true)
                                      on:blur=move |_| set_input_focused.set(false)
                                      on:keydown=on_keydown
                                  />

                                  {move || {
                                      let items = unique_search_results();
                                      let focused = input_focused.get();
                                      let input_val = user_input.get();

                                      if focused && !items.is_empty() && !input_val.is_empty() {
                                          view! {
                                              <div>
                                              <ul class=styles::dropdown_list>
                                                  <For
                                                      each=move || items.clone().into_iter().enumerate()
                                                      key=|(_, item)| item.id.clone()
                                                      children=move |(i, item)| {
                                                          let is_selected = move || selected_dropdown_index.get() == i;
                                                          let name_clone = item.name_cn.clone();
                                                          view! {
                                                              <li
                                                                  class=move || if is_selected() { styles::dropdown_item_active } else { styles::dropdown_item }
                                                                  on:mousedown=move |ev| ev.prevent_default()
                                                                  on:click=move |_| {
                                                                      set_user_input.set(name_clone.clone());
                                                                      set_selected_dropdown_index.set(i);
                                                                      send_guess();
                                                                  }
                                                              >
                                                                  {item.name_cn}
                                                              </li>
                                                          }
                                                      }
                                                  />
                                              </ul>
                                              </div>
                                          }
                                      } else {
                                          view! { <div><ul style="display:none"></ul></div> }
                                      }
                                  }}
                              </div>
                          </div>
                      </div>

                      <div class=styles::button_section>
                          // send buttons
                          <button
                              disabled=is_interaction_disabled
                              on:click=move |_| send_guess()
                          >
                              {move || texts().2}
                          </button>
                          // reset button
                          <button
                              class=styles::reset_btn
                              on:click=reset_game
                          >
                              {reset_icon}
                          </button>
                          </div>
                      <div class=styles::guess_number>
                          <span> {guess_time}/{current_config.get().max_guess} </span>
                      </div>
                      <div class=styles::timer>
                          <span class=styles::timer_text> {formatted_time} </span>
                      </div>
                  </div>

                    // all the answers
                  <div class=styles::display_section>
                  <For
                      each=move || cards.get()
                      key=|(item, _)| item.id.clone()
                      children=move |(item, comp_res)| {
                          view! {
                              <div>
                                  <Card info=item comparison=comp_res />
                              </div>
                              }
                          }
                  />
                  </div>

                  // the final answer
                   <div class=styles::answer_reveal_section>
                      {move || {
                          let state = game_state.get();
                          if state == GameState::Win || state == GameState::Lose {
                              let (status_text, status_class) = match state {
                                  GameState::Win => (texts().4, styles::status_win),
                                  GameState::Lose => (texts().5, styles::status_lose),
                                  _ => ("", ""),
                              };

                              view! {
                                  <div>
                                      <div class=styles::reveal_container>
                                          <h2 class=status_class>{move || status_text}</h2>
                                          <h4 class=status_class>{guess_time}/{current_config.get().max_guess}</h4>
                                          <h4 class=status_class>Time: {formatted_time}</h4>
                                          <button
                                              class=styles::reset_btn
                                              on:click=reset_game
                                          >
                                              {reset_icon}
                                          </button>
                                          <hr class=styles::divider />
                                          <p class=styles::reveal_text> {move || texts().5} </p>

                                          <Suspense fallback=|| view! { "..." }>
                                              {move || Suspend::new(async move {
                                                  match answer.get() {
                                                      Some(a) => view! {<div> <Card info=a.0.clone() comparison=a.1/> </div>},
                                                      None => view! { <div><span></span> </div> }
                                                  }
                                              })}
                                          </Suspense>
                                      </div>
                                  </div>
                              }
                          } else {
                              view! { <div><div style="display:none"></div></div> }
                          }
                      }}
                  </div>

                  </Show>

            </main>

                // chat
              <Show when=move || game_state.get() != GameState::Lobby && game_state.get() != GameState::Matching>
                    <div class=styles::chat_panel>

                        <div class=styles::chat_messages>
                            {move || {
                            chat_log.get()
                                .iter()
                                .enumerate()
                                .map(|(_, item)| {
                                    let bubble_class = match item.side {
                                        ChatSide::I => styles::chat_item_me,
                                        ChatSide::O => styles::chat_item_other,
                                    };
                                    view! {
                                        <div class=bubble_class>
                                            {format!("{}", item.content.clone())}
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>()
                            }}

                        </div>
                        <div class=styles::chat_input_row>
                            <input
                                class=styles::chat_input
                                placeholder="message..."
                                bind:value=(text, set_text)
                                disabled=move || game_state.get() == GameState::Matching
                            />
                            <button
                                class=styles::chat_send
                                on:click=send_text
                                disabled=move || game_state.get() == GameState::Matching
                            >
                                "Send"
                            </button>
                        </div>
                    </div>
                </Show>

        </ErrorBoundary>
    }
}

pub fn use_interval<T, F>(interval_millis: T, f: F)
where
    F: Fn() + Clone + 'static,
    T: Into<Signal<u64>> + 'static,
{
    let interval_millis = interval_millis.into();
    Effect::new(move |prev_handle: Option<IntervalHandle>| {
        if let Some(prev_handle) = prev_handle {
            prev_handle.clear();
        };

        set_interval_with_handle(f.clone(), Duration::from_millis(interval_millis.get()))
            .expect("could not create interval")
    });
}
