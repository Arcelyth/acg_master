use gloo_net::websocket::Message;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use stylance::import_crate_style;

import_crate_style!(styles, "./src/pages/styles/multi.module.scss");

use crate::bangumi::anime::*;
use crate::components::{back_btn::BackBtn, card2::Card2};
use crate::config::{Config, Language};
use crate::ws::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GameState {
    Lobby, // before matching
    Matching,
    Waiting, // waiting in room
    Loading,
    Playing,
    Exhausted,
    Win,
    Lose,
    Draw,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ChatSide {
    I,
    O,
    Sys,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatEntry {
    name: String,
    content: String,
    is_sys: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerEntry {
    is_prepared: bool,
    guess_time: usize,
}

#[component]
pub fn Multi() -> impl IntoView {
    let (game_state, set_game_state) = signal(GameState::Lobby);
    // for chat
    let (chat_log, set_chat_log) = signal::<Vec<ChatEntry>>(vec![]);
    let (username, set_username) = signal("".to_string());
    let (room_name, set_room_name) = signal("".to_string());
    let (text, set_text) = signal("".to_string());
    let config = use_context::<ReadSignal<Config>>().expect("reader");

    let ws_sender = StoredValue::new(None::<futures::channel::mpsc::UnboundedSender<Message>>);

    // timer
    let (elapsed_seconds, set_elapsed_seconds) = signal(0u64);
    let (is_timer_running, set_is_timer_running) = signal(false);

    let (dup, set_dup) = signal(false);
    let (user_input, set_user_input) = signal("".to_string());
    let (debounced_input, set_debounced_input) = signal("".to_string());
    let (input_version, set_input_version) = signal(0);

    let (input_focused, set_input_focused) = signal(false);
    let (selected_dropdown_index, set_selected_dropdown_index) = signal(0usize);

    let (guess_time, set_guess_time) = signal(0usize);

    let (cards, set_cards) = signal::<Vec<(BangumiSubject, CompareResult)>>(vec![]);
    let (hide_cards, set_hide_cards) =
        signal::<HashMap<String, Vec<BangumiSubjectHide>>>(HashMap::new());
    let (_refresh_trigger, set_refresh_trigger) = signal(0);
    let (answer, set_answer) = signal(None);
    let (send_reset, set_send_reset) = signal(false);

    let search_results = LocalResource::new(move || bangumi_search(debounced_input.get()));

    let (multi_config, set_multi_config) = signal::<MultiConfig>(MultiConfig {
        max_guess: 10,
        start_year: 1960,
        end_year: 2026,
    });

    let (rooms, set_rooms) = signal::<Vec<RoomInfo>>(vec![]);
    let (players, set_players) = signal::<HashMap<String, PlayerEntry>>(HashMap::new());
    let (join_trigger, set_join_trigger) = signal::<Option<String>>(None);
    let (create_trigger, set_create_trigger) = signal::<Option<String>>(None);
    let (is_host, set_is_host) = signal::<bool>(false);
    let (winner, set_winner) = signal::<Option<String>>(None);
    let (all_guesses, set_all_guesses) = signal::<HashMap<String, Vec<String>>>(HashMap::new());
    let (username_err, set_username_err) = create_signal(false);
    let (room_name_err, set_room_name_err) = create_signal(false);
    let (is_modal_open, set_is_modal_open) = signal::<bool>(false);

    // disconnect
    on_cleanup(move || {
        if let Some(tx) = ws_sender.get_value() {
            let _ = tx.unbounded_send(Message::Text(
                serde_json::to_string(&ClientMsg::ILeave).unwrap(),
            ));
        }
    });

    Effect::new(move |_| {
        let state = game_state.get();
        if state == GameState::Win || state == GameState::Lose {
            set_is_timer_running.set(false);
        }
    });

    Effect::new(move |_| {
        spawn_local(async move {
            let rooms = get_rooms().await;
            set_rooms.set(rooms);
        });
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

    let send_text = move |_| {
        let current_msg = text.get_untracked();
        if current_msg.is_empty() {
            return;
        }

        set_chat_log.update(|v| {
            v.push(ChatEntry {
                name: username.get(),
                content: current_msg.clone(),
                is_sys: false,
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

    let texts = move || match config.get().lang {
        Language::Chinese => (
            "输入名称",
            "创建房间",
            "匹配中......",
            "输入动漫名称",
            "胜者：",
            "没人猜对",
            "发送",
            "输入你的答案",
            "等待对方中......",
            "输入消息",
            "发送",
            "有人离开了房间",
            "已在列表中",
            "答案",
            "次数用尽",
            ("封面", "标题与集数", "评分", "放送日期", "标签"),
            "输入房间名称",
            ("玩家", "状态", "猜测次数"),
            ("已准备", "未准备"),
            ("玩家名称不能为空", "房间名称不能为空"),
            ("当前房间数", "等待中", "游戏中", "加入"),
            ("准备就绪", "开始"),
            ("猜测次数", "重置以生效", "年份范围", "至"),
        ),
        Language::English => (
            "Input your name",
            "Create a room",
            "Matching...",
            "Input anime's name",
            "Winner: ",
            "No one guessed it right.",
            "Send",
            "Input your answer",
            "Waiting for the opponent...",
            "Input message",
            "Send",
            "Someone has left the room",
            "Already in the list",
            "ANSWER",
            "Run out of guess times",
            ("Cover", "Title & Eps", "Rating", "Air Date", "Tags"),
            "Input room name",
            ("Player", "State", "Guess time"),
            ("Ready", "Not Ready"),
            ("Player name cannot be empty", "Room name cannot be empty"),
            ("Current Room Count", "Waiting", "In Game", "Join"),
            ("Ready", "Start"),
            ("Guess Times", "Reset to apply", "Year Range", "to"),
        ),
    };

    let connect = move |_| {
        let sender = connect_ws(move |msg| {
            println!("recv: {}", msg);

            if let Ok(server_msg) = serde_json::from_str::<ServerMsg>(&msg) {
                match server_msg {
                    ServerMsg::JoinSucc(ps) => {
                        set_game_state.set(GameState::Waiting);
                        set_players.update(|p| {
                            p.insert(
                                username.get(),
                                PlayerEntry {
                                    is_prepared: false,
                                    guess_time: 0,
                                },
                            );
                            for (name, data) in ps {
                                p.insert(
                                    name,
                                    PlayerEntry {
                                        is_prepared: data.is_prepared,
                                        guess_time: 0,
                                    },
                                );
                            }
                        });
                    }
                    ServerMsg::OJoinSucc(name) => {
                        set_game_state.set(GameState::Waiting);

                        set_players.update(|p| {
                            let mut new = p.clone();
                            new.insert(
                                name,
                                PlayerEntry {
                                    is_prepared: false,
                                    guess_time: 0,
                                },
                            );
                            *p = new;
                        });
                    }

                    ServerMsg::Start(conf) => {
                        set_game_state.set(GameState::Playing);
                        set_cards.set(Vec::new());
                        set_winner.set(None);
                        set_all_guesses.set(HashMap::new());
                        set_elapsed_seconds.set(0u64);
                        set_multi_config.set(conf);
                    }
                    ServerMsg::CreateRoomOk => {
                        set_game_state.set(GameState::Waiting);
                        set_is_host.set(true);
                        set_players.update(|p| {
                            p.insert(
                                username.get(),
                                PlayerEntry {
                                    is_prepared: false,
                                    guess_time: 0,
                                },
                            );
                        });
                    }

                    ServerMsg::Prepare(name) => {
                        set_players.update(|p| {
                            if let Some(entry) = p.get_mut(&name) {
                                entry.is_prepared = true;
                            }
                        });
                    }
                    ServerMsg::Response(name, m) => {
                        set_chat_log.update(|v| {
                            v.push(ChatEntry {
                                name,
                                content: m,
                                is_sys: false,
                            });
                        });
                    }
                    ServerMsg::GuessResp(WsGuessResponse { guess, comparison }, gt) => {
                        set_cards.update(|c| c.push((guess, comparison)));
                        set_guess_time.set(gt);
                        set_players.update(|p| {
                            if let Some(entry) = p.get_mut(&username.get()) {
                                entry.guess_time = gt;
                            }
                        });

                        if gt >= multi_config.get().max_guess {
                            set_game_state.set(GameState::Exhausted);
                        }
                    }
                    ServerMsg::OGuessResp(name, gt) => {
                        if hide_cards.get_untracked().is_empty() {
                            set_is_timer_running.set(true);
                        }

                        set_players.update(|p| {
                            if let Some(entry) = p.get_mut(&name) {
                                entry.guess_time = gt;
                            }
                        });
                        //                        set_hide_cards.update(|c| {
                        //                            c.entry(name).or_insert_with(Vec::new).push(hide);
                        //                        });
                    }
                    ServerMsg::Over(winner, all_guesses, answer) => {
                        if let Some(winner) = winner.clone() {
                            if winner == username.get() {
                                set_game_state.set(GameState::Win);
                            } else {
                                set_game_state.set(GameState::Lose);
                            }
                        } else {
                            set_game_state.set(GameState::Draw);
                        }
                        set_all_guesses.set(all_guesses);
                        set_winner.set(winner);
                        set_answer.set(Some(answer));
                        set_players.update(|p| {
                            for e in p.values_mut() {
                                *e = PlayerEntry {
                                    is_prepared: false,
                                    guess_time: 0,
                                }
                            }
                        });
                    }
                    ServerMsg::Reset => {
                        set_cards.set(vec![]);
                        set_hide_cards.set(HashMap::new());
                        set_guess_time.set(0);
                        set_user_input.set("".to_string());

                        set_dup.set(false);
                        set_game_state.set(GameState::Playing);

                        set_send_reset.set(false);
                        set_is_timer_running.set(false);
                        set_elapsed_seconds.set(0);

                        set_refresh_trigger.update(|n| *n += 1);
                    }
                    ServerMsg::ResetOk => {
                        set_send_reset.set(true);
                    }
                    ServerMsg::Leave(name) => {
                        set_chat_log.update(|v| {
                            v.push(ChatEntry {
                                name: String::new(),
                                content: texts().11.to_string(),
                                is_sys: true,
                            });
                        });
                        set_players.update(|p| {
                            p.remove(&name);
                        });
                    }
                }
            }
        });

        ws_sender.set_value(Some(sender));
    };

    let create_room = move || {
        let name_len = username.get().len();
        if name_len < 1 || name_len > 20 {
            return;
        }

        set_create_trigger.set(Some(room_name.get()));
        connect(());
    };

    Effect::new(move |_| {
        if let Some(name) = create_trigger.get() {
            if let Some(tx) = ws_sender.get_value() {
                let msg = ClientMsg::CreateRoom(name, username.get_untracked());
                if let Ok(text) = serde_json::to_string(&msg) {
                    let _ = tx.unbounded_send(Message::Text(text));
                    set_create_trigger.set(None);
                }
            }
        }
    });

    let unique_search_results = move || {
        let res = search_results.get().flatten().unwrap_or_default();
        let mut seen = HashSet::new();
        res.into_iter()
            .filter(|item| !item.name_cn.is_empty())
            .filter(|item| seen.insert(item.name_cn.clone()))
            .collect::<Vec<_>>()
    };

    let send_guess = move || {
        set_dup.set(false);
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
                set_dup.set(true);
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

    let join_room = move |id: String| {
        let name_len = username.get().len();
        if name_len < 1 || name_len > 20 {
            return;
        }

        set_join_trigger.set(Some(id));
        connect(());
    };

    Effect::new(move |_| {
        if let Some(id) = join_trigger.get() {
            if let Some(tx) = ws_sender.get_value() {
                let msg = ClientMsg::Join(id, username.get_untracked());
                if let Ok(text) = serde_json::to_string(&msg) {
                    let _ = tx.unbounded_send(Message::Text(text));
                    set_join_trigger.set(None);
                }
            }
        }
    });

    let prepare = move |_| {
        if let Some(tx) = ws_sender.get_value() {
            let msg = ClientMsg::Prepare;

            if let Ok(text) = serde_json::to_string(&msg) {
                let _ = tx.unbounded_send(Message::Text(text));
            }
        }
    };

    let start_game = move |_| {
        let all_prepared = players.get_untracked().values().all(|p| p.is_prepared);
        if !all_prepared {
            return;
        }

        if let Some(tx) = ws_sender.get_value() {
            let msg = ClientMsg::Start(multi_config.get());
            set_game_state.set(GameState::Loading);

            if let Ok(text) = serde_json::to_string(&msg) {
                let _ = tx.unbounded_send(Message::Text(text));
            }
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

                <Show when=move || !players.get().is_empty()>

                    <div class=styles::players_table_container>
                        <table class=styles::players_table>
                            <thead>
                                <tr>
                                    <th class=styles::th_cell>{texts().17.0}</th>
                                    <th class=styles::th_cell>{texts().17.1}</th>
                                    <th class=styles::th_cell>{texts().17.2}</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || {
                                    let mut p_list: Vec<_> = players.get().into_iter().collect();
                                    p_list.sort_by(|a, b| a.0.cmp(&b.0));
                                    p_list.into_iter().map(|(name, entry)| {
                                        let status = if entry.is_prepared { texts().18.0 } else { texts().18.1 };
                                        let st_class = if entry.is_prepared { styles::status_ready } else { styles::status_unready };
                                        view! {
                                            <tr class=styles::tr_row>
                                                <td class=styles::td_cell>{name}</td>
                                                <td class=styles::td_cell><span class=st_class>{status}</span></td>
                                                <td class=styles::td_cell><span>{entry.guess_time}</span></td>
                                            </tr>
                                        }
                                    }).collect_view()
                                }}
                            </tbody>
                        </table>
                    </div>
                </Show>
                <Show when=move || game_state.get() == GameState::Lobby || game_state.get() == GameState::Matching>
                    <div class=styles::lobby_container>
                        <div class=styles::lobby_section>
                            <div class=styles::input_wrapper>
                                <input
                                    class=styles::username_input
                                    placeholder=texts().0
                                    bind:value=(username, set_username)
                                    on:input=move |_| set_username_err.set(false)
                                />
                                <Show when=move || username_err.get() fallback=|| ()>
                                    <span class=styles::error_msg>{texts().19.0}</span>
                                </Show>
                            </div>
                            <div class=styles::input_wrapper>
                                <input
                                    class=styles::username_input
                                    placeholder=texts().16
                                    bind:value=(room_name, set_room_name)
                                    on:input=move |_| set_room_name_err.set(false)
                                />
                                <Show when=move || room_name_err.get() fallback=|| ()>
                                    <span class=styles::error_msg>{texts().19.1}</span>
                                </Show>
                            </div>
                            <button
                                class=styles::match_btn
                                on:click=move |_| {
                                    let mut valid = true;
                                    if username.get().trim().is_empty() {
                                        set_username_err.set(true);
                                        valid = false;
                                    }
                                    if room_name.get().trim().is_empty() {
                                        set_room_name_err.set(true);
                                        valid = false;
                                    }
                                    if valid {
                                        create_room();
                                    }
                                }
                                disabled=move || game_state.get() == GameState::Matching
                            >
                                {move || if game_state.get() == GameState::Matching { texts().2 } else { texts().1 }}
                            </button>
                        </div>

                        <div class=styles::room_list_section>
                            <div class=styles::room_header>
                                {move || format!("{}：{}/100", texts().20.0,rooms.get().len())}
                            </div>
                            <div class=styles::room_grid>
                                <For
                                    each=move || rooms.get()
                                    key=|room| room.id.clone()
                                    children=move |room| {
                                        let state_text = match room.state {
                                            RoomState::Waiting => texts().20.1,
                                            RoomState::Playing => texts().20.2,
                                        };
                                        let state_class = match room.state {
                                            RoomState::Waiting => styles::state_waiting,
                                            RoomState::Playing => styles::state_playing,
                                        };
                                        view! {
                                            <div class=styles::room_item>
                                                <div class=styles::room_details>
                                                    <span class=styles::room_name>{room.name.clone()}</span>
                                                    <span class=styles::room_players>{room.player_num}"/10"</span>
                                                    <span class=state_class>{state_text}</span>
                                                </div>
                                                <button
                                                    class=styles::join_btn
                                                    on:click=move |_| {
                                                        if username.get().trim().is_empty() {
                                                            set_username_err.set(true);
                                                        } else {
                                                            join_room(room.id.clone());
                                                        }
                                                    }
                                                    disabled=move || room.state != RoomState::Waiting || game_state.get() == GameState::Matching
                                                >
                                                    {texts().20.3}
                                                </button>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        </div>
                    </div>
                </Show>

                <Show when=move || game_state.get() != GameState::Lobby && game_state.get() != GameState::Matching>

                  // interact section
                   <div class=styles::interact_section>
                      <div class=styles::search_wrapper>
                         <div class=styles::input_section>
                              <span> {move || texts().0}: </span>

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
                                      if dup.get() {
                                          view! { <div><span class=styles::dup_message>{move || texts().12}</span></div> }
                                      } else {
                                          view! { <div><div style="display:none"></div></div> }
                                      }
                                  }}

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
                              {move || texts().6}
                          </button>

                          </div>
                      <div class=styles::guess_number>
                          <span> {guess_time}/{multi_config.get().max_guess} </span>
                      </div>
                      <div class=styles::timer>
                          <span class=styles::timer_text> {formatted_time} </span>
                      </div>
                  </div>

                    // all the answers
                  <div class=styles::display_section>
                    // the table header
                    <div class=styles::table_header>
                        <div class=styles::header_image_placeholder>
                            {move || texts().15.0}
                        </div>

                        <div class=styles::header_content_grid>
                            <div class=styles::col_header_text>{move || texts().15.1}</div>
                            <div class=styles::center_text>{move || texts().15.2}</div>
                            <div class=styles::center_text>{move || texts().15.3}</div>
                            <div class=styles::col_header_text>{move || texts().15.4}</div>
                        </div>
                    </div>


                 <div class=styles::your_answers>
                   <For
                       each=move || cards.get()
                       key=|(item, _)| item.id.clone()
                       children=move |(item, comp_res)| {
                           view! {
                                   <Card2 info=item comparison=comp_res />
                               }
                           }
                   />
                 </div>

                <Show when=move || game_state.get() == GameState::Exhausted>
                    <span class=styles::exhausted>
                        {move || texts().14}
                    </span>
                </Show>

                <Show
                    when=move || game_state.get() == GameState::Loading
                    fallback=move || view! { <div /> }
                >
                    <div class=styles::loader_wrapper>
                        <div class=styles::spinner></div>
                    </div>
                </Show>

                   <Show when=move || matches!(game_state.get(), GameState::Waiting | GameState::Win)>
                        <div class=styles::prepare_section>
                        <button
                            class=move || if players.get().get(&username.get()).map(|p| p.is_prepared).unwrap_or(false) {
                                styles::prepare_btn_active
                            } else {
                                styles::prepare_btn
                            }
                            on:click=prepare>
                            { texts().21.0 }
                        </button>
                        {
                            move || {
                                if is_host.get() {
                                  view!(
                                  <div>
                                      <button
                                            class=styles::prepare_btn
                                            on:click=start_game
                                            disabled=move || {
                                                !players
                                                    .get()
                                                    .values()
                                                    .all(|p| p.is_prepared)
                                            }>
                                          {texts().21.1}
                                        </button>
                                                                         </div>
                                    )
                                } else {
                                    view!(<div><div></div></div>)
                                }
                            }
                        }
                        </div>
                    </Show>

                  // the final answer
                   <div class=styles::answer_reveal_section>
                      {move || {
                          let state = game_state.get();
                          if state == GameState::Win || state == GameState::Lose || state == GameState::Draw {
                                let w = winner.get();
                              let (status_text, status_class) = match state {
                                  GameState::Win => (format!("{} {}", texts().4, w.unwrap()), styles::status_win),
                                  GameState::Lose => (format!("{} {}", texts().4, w.unwrap()), styles::status_lose),
                                  GameState::Draw => (texts().5.to_string(), styles::status_draw),
                                  _ => ("".to_string(), ""),
                              };

                              view! {
                                  <div>
                                      <div class=styles::reveal_container>
                                          <h2 class=status_class>{status_text}</h2>
                                          <h4 class=status_class>{guess_time}/{multi_config.get().max_guess}</h4>
                                          <h4 class=status_class>Time: {formatted_time}</h4>
                                        <div class=styles::btn_wrapper>

                                            <Show
                                                when=move || send_reset.get()
                                                fallback=|| ()
                                            >
                                                <span class=styles::reset_hint>
                                                    {move || texts().8}
                                                </span>
                                            </Show>
                                        </div>
                                          <hr class=styles::divider />
                                          <p class=styles::reveal_text> {move || texts().13} </p>

                                          <Suspense fallback=|| view! { "..." }>
                                              {move || Suspend::new(async move {
                                                  match answer.get() {
                                                      Some(a) => view! {<div> <Card2 info=a.0.clone() comparison=a.1/> </div>},
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
                     </div>


                    <Show
                        when=move || {
                            let s = game_state.get();
                            s == GameState::Win || s == GameState::Lose || s == GameState::Draw
                        }
                        fallback=|| ()
                    >
                        <div class=styles::guess_history_wrapper>
                            <table class=styles::guess_history_table>
                                <thead>
                                    <tr>
                                        <For
                                            each=move || {
                                                let mut p: Vec<String> = all_guesses.get().keys().cloned().collect();
                                                p.sort();
                                                p
                                            }
                                            key=|name| name.clone()
                                            children=move |name| {
                                                view! { <th class=styles::guess_th>{name}</th> }
                                            }
                                        />
                                    </tr>
                                </thead>
                                <tbody>
                                    <For
                                        each=move || {
                                            let max_rows = all_guesses.get().values().map(|v| v.len()).max().unwrap_or(0);
                                            (0..max_rows).collect::<Vec<_>>()
                                        }
                                        key=|i| *i
                                        children=move |row_idx| {
                                            view! {
                                                <tr class=styles::guess_tr>
                                                    <For
                                                        each=move || {
                                                            let mut p: Vec<String> = all_guesses.get().keys().cloned().collect();
                                                            p.sort();
                                                            p
                                                        }
                                                        key=|name| name.clone()
                                                        children=move |name| {
                                                            let guess = move || {
                                                                all_guesses.get()
                                                                    .get(&name)
                                                                    .and_then(|guesses| guesses.get(row_idx))
                                                                    .cloned()
                                                                    .unwrap_or_default()
                                                            };
                                                            view! {
                                                                <td class=styles::guess_td>{guess}</td>
                                                            }
                                                        }
                                                    />
                                                </tr>
                                            }
                                        }
                                    />
                                </tbody>
                            </table>
                        </div>
                    </Show>

                  </Show>

            </main>

            // setting
            <Show when=move || game_state.get() == GameState::Waiting>
                <button
                    class=styles::settings_btn
                    on:click=move |_| set_is_modal_open.set(true)
                >
                    <svg viewBox="0 0 24 24">
                        <circle cx="12" cy="12" r="3"></circle>
                        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
                    </svg>
                </button>

                <Show when=move || is_modal_open.get()>
                    <div class=styles::modal_overlay on:click=move |_| set_is_modal_open.set(false)>
                        <div class=styles::modal_content on:click=move |e| e.stop_propagation()>
                            <button class=styles::close_btn on:click=move |_| set_is_modal_open.set(false)>
                                <svg viewBox="0 0 24 24">
                                    <line x1="18" y1="6" x2="6" y2="18"></line>
                                    <line x1="6" y1="6" x2="18" y2="18"></line>
                                </svg>
                            </button>

                            <div class=styles::setting_item>
                                <label>{move || texts().22.0}</label>
                                <div class=styles::slider_wrapper>
                                    <input
                                        type="range"
                                        min="1"
                                        max="100"
                                        prop:value=move || multi_config.get().max_guess
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<usize>() {
                                                set_multi_config.update(|v| v.max_guess = val);
                                            }
                                        }
                                    />
                                    <span>{move || multi_config.get().max_guess}</span>
                                </div>
                            </div>

                            <div class=styles::setting_item>
                                <label>{move || texts().22.2}</label>
                                <div class=styles::year_range_wrapper>
                                    <input
                                        type="number"
                                        class=styles::year_input
                                        min="1960"
                                        max=move || multi_config.get().end_year.to_string()
                                        prop:value=move || multi_config.get().start_year
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<usize>() {
                                                set_multi_config.update(|v| v.start_year = val);
                                            }
                                        }
                                    />
                                    <span>{move || texts().22.3}</span>
                                    <input
                                        type="number"
                                        class=styles::year_input
                                        min=move || multi_config.get().start_year.to_string()
                                        max="2026"
                                        prop:value=move || multi_config.get().end_year
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<usize>() {
                                                set_multi_config.update(|v| v.end_year = val);
                                            }
                                        }
                                    />
                                </div>
                            </div>

                            <div class=styles::note_text>{move || texts().22.1}</div>
                        </div>
                    </div>
                </Show>
            </Show>

                // chat
              <Show when=move || game_state.get() != GameState::Lobby && game_state.get() != GameState::Matching>
                    <div class=styles::chat_panel>
                        <div class=styles::chat_messages>
                            {move || {
                                chat_log.get()
                                    .iter()
                                    .enumerate()
                                    .map(|(_, item)| {
                                        if item.is_sys {
                                            view! {
                                                <div class=styles::chat_item_sys>
                                                    {item.content.clone()}
                                                </div>
                                            }.into_any()
                                        } else if item.name == username.get() {
                                            view! {
                                                <div class=styles::chat_wrapper_me>
                                                    <div class=styles::chat_item_me>
                                                        {item.content.clone()}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class=styles::chat_wrapper_other>
                                                    <span class=styles::chat_name>{item.name.clone()}</span>
                                                    <div class=styles::chat_item_other>
                                                        {item.content.clone()}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            }}
                        </div>
                        <div class=styles::chat_input_row>
                            <input
                                class=styles::chat_input
                                placeholder=texts().9
                                bind:value=(text, set_text)
                                disabled=move || game_state.get() == GameState::Matching
                            />
                            <button
                                class=styles::chat_send
                                on:click=send_text
                                disabled=move || game_state.get() == GameState::Matching
                            >
                                {texts().10}
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

    Effect::new(move |_| {
        let handle =
            set_interval_with_handle(f.clone(), Duration::from_millis(interval_millis.get()))
                .expect("could not create interval");

        on_cleanup(move || {
            handle.clear();
        });
    });
}
