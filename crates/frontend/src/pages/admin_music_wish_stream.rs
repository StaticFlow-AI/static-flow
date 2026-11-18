use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Event, EventSource, MessageEvent};
use yew::prelude::*;

use crate::{
    api::{
        build_admin_music_wish_ai_stream_url, fetch_admin_music_wish_ai_output,
        MusicWishAiRunChunk, MusicWishAiRunRecord,
    },
    components::stream_chunk_batcher::ChunkBatcher,
    pages::{
        admin::shell::AdminNav,
        admin_ai_stream::{log_entries, RunEntry, RunReader},
    },
};

type NamedEventClosure = Closure<dyn FnMut(MessageEvent)>;
type ErrorClosure = Closure<dyn FnMut(Event)>;

struct StreamHandle {
    source: EventSource,
    _on_chunk: NamedEventClosure,
    _on_done: NamedEventClosure,
    _on_stream_error: NamedEventClosure,
    _on_error: ErrorClosure,
    batcher: ChunkBatcher<MusicWishAiRunChunk, (i32, String)>,
}

impl Drop for StreamHandle {
    fn drop(&mut self) {
        self.source.close();
        self.batcher.cancel();
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub wish_id: String,
}

#[function_component(AdminMusicWishRunsPage)]
pub fn admin_music_wish_runs_page(props: &Props) -> Html {
    let load_error = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let runs = use_state(Vec::<MusicWishAiRunRecord>::new);
    let stream_chunks = use_state(Vec::<MusicWishAiRunChunk>::new);
    let stream_status = use_state(|| "idle".to_string());
    let stream_error = use_state(|| None::<String>);
    let stream_ref = use_mut_ref(|| None::<StreamHandle>);

    let wish_id = props.wish_id.clone();

    // Load AI output on mount
    {
        let wish_id = wish_id.clone();
        let load_error = load_error.clone();
        let loading = loading.clone();
        let runs = runs.clone();
        let stream_chunks = stream_chunks.clone();
        let stream_status = stream_status.clone();
        use_effect_with(wish_id.clone(), move |id| {
            let id = id.clone();
            let load_error = load_error;
            let loading = loading;
            let runs = runs;
            let stream_chunks = stream_chunks;
            let stream_status = stream_status;
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_admin_music_wish_ai_output(&id).await {
                    Ok(data) => {
                        if let Some(latest) = data.runs.last() {
                            stream_status.set(latest.status.clone());
                        }
                        stream_chunks.set(data.chunks);
                        runs.set(data.runs);
                        load_error.set(None);
                    },
                    Err(err) => {
                        load_error.set(Some(format!("Failed to load AI runs: {}", err)));
                    },
                }
                loading.set(false);
            });
            || ()
        });
    }

    // SSE connection for live streaming (named events: chunk, done, error)
    {
        let wish_id = wish_id.clone();
        let stream_chunks = stream_chunks.clone();
        let stream_status = stream_status.clone();
        let stream_error = stream_error.clone();
        let stream_ref = stream_ref.clone();
        use_effect_with(wish_id.clone(), move |wid| {
            if let Some(handle) = stream_ref.borrow_mut().take() {
                drop(handle);
            }

            let stream_url = build_admin_music_wish_ai_stream_url(wid);
            if let Ok(source) = EventSource::new(&stream_url) {
                stream_status.set("streaming".to_string());

                let batcher = ChunkBatcher::new(
                    stream_chunks.clone(),
                    |c: &MusicWishAiRunChunk| (c.batch_index, c.stream.clone()),
                    |c: &MusicWishAiRunChunk| c.batch_index,
                );
                let batcher_for_msg = batcher.clone();

                let on_chunk =
                    Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
                        let Some(text) = event.data().as_string() else { return };
                        let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) else {
                            return;
                        };
                        let stream = val["stream"].as_str().unwrap_or("stdout").to_string();
                        let batch_index = val["batch_index"].as_i64().unwrap_or(0) as i32;
                        let content = val["content"].as_str().unwrap_or("").to_string();
                        batcher_for_msg.push(MusicWishAiRunChunk {
                            chunk_id: format!("live-{}-{}", stream, batch_index),
                            run_id: String::new(),
                            wish_id: String::new(),
                            stream,
                            batch_index,
                            content,
                            created_at: 0,
                        });
                    });

                let on_done = {
                    let stream_status = stream_status.clone();
                    Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
                        let status = event
                            .data()
                            .as_string()
                            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
                            .and_then(|v| v["status"].as_str().map(String::from))
                            .unwrap_or_else(|| "done".to_string());
                        stream_status.set(status);
                    })
                };

                let on_stream_error = {
                    let stream_status = stream_status.clone();
                    let stream_error = stream_error.clone();
                    Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
                        let reason = event
                            .data()
                            .as_string()
                            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
                            .and_then(|v| v["failure_reason"].as_str().map(String::from))
                            .unwrap_or_else(|| "Unknown error".to_string());
                        stream_status.set("error".to_string());
                        stream_error.set(Some(reason));
                    })
                };

                let on_error = {
                    let stream_status = stream_status.clone();
                    let stream_error = stream_error.clone();
                    let source_ref = source.clone();
                    Closure::<dyn FnMut(Event)>::new(move |_| {
                        // Guard: ignore onerror if we already reached a terminal state
                        let current = (*stream_status).clone();
                        if current == "success" || current == "done" || current == "error" {
                            return;
                        }
                        // readyState: 0=CONNECTING (browser auto-reconnecting), 2=CLOSED
                        if source_ref.ready_state() == 2 {
                            stream_status.set("disconnected".to_string());
                            stream_error.set(Some("Stream connection closed".to_string()));
                        }
                        // CONNECTING (0) → browser is retrying, don't overwrite
                        // status
                    })
                };

                let _ = source
                    .add_event_listener_with_callback("chunk", on_chunk.as_ref().unchecked_ref());
                let _ = source
                    .add_event_listener_with_callback("done", on_done.as_ref().unchecked_ref());
                let _ = source.add_event_listener_with_callback(
                    "error",
                    on_stream_error.as_ref().unchecked_ref(),
                );
                source.set_onerror(Some(on_error.as_ref().unchecked_ref()));

                *stream_ref.borrow_mut() = Some(StreamHandle {
                    source,
                    _on_chunk: on_chunk,
                    _on_done: on_done,
                    _on_stream_error: on_stream_error,
                    _on_error: on_error,
                    batcher,
                });
            } else {
                stream_error.set(Some("Failed to open EventSource".to_string()));
            }

            let stream_ref = stream_ref.clone();
            move || {
                stream_ref.borrow_mut().take();
            }
        });
    }

    let run_rows = runs
        .iter()
        .map(|run| RunEntry {
            run_id: run.run_id.clone(),
            status: run.status.clone(),
            runner: run.runner_program.clone(),
            exit_code: run.exit_code,
            started_at: run.started_at,
            completed_at: run.completed_at,
            failure_reason: run.failure_reason.clone(),
        })
        .collect::<Vec<_>>();
    let logs = log_entries(stream_chunks.iter().map(|chunk| {
        (chunk.stream.clone(), chunk.batch_index, chunk.created_at, chunk.content.clone())
    }));

    html! {
        <RunReader
            nav={AdminNav::MusicWishes}
            eyebrow="共建队列 / 点歌心愿"
            title="点歌心愿 AI 运行记录"
            description="实时跟随该心愿最新一次 AI 运行的输出，并保留历史运行概览。"
            id_label="wish_id"
            id_value={wish_id}
            stream_status={(*stream_status).clone()}
            loading={*loading}
            load_error={(*load_error).clone()}
            stream_error={(*stream_error).clone()}
            runs={run_rows}
            selected_run_id={None::<String>}
            empty_runs_hint="该心愿还没有触发过 AI 运行。"
            logs={logs}
        />
    }
}
