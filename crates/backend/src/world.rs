//! Shared state for the driving world's four public interactions.
use std::{path::PathBuf, sync::Arc};

use axum::{
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use parking_lot::Mutex;
use rusqlite::{params, Connection, TransactionBehavior};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::public_submit_guard::{
    build_client_fingerprint, build_submit_rate_limit_key, enforce_public_submit_rate_limit,
    PublicSubmitGuard,
};

const DAY_MS: i64 = 86_400_000;

struct World {
    path: PathBuf,
    connection: Mutex<Option<Connection>>,
    guards: [PublicSubmitGuard; 4],
}

pub(crate) fn router(path: PathBuf) -> Router {
    Router::new()
        .route("/api/world", get(read).post(update))
        .layer(DefaultBodyLimit::max(2048))
        .with_state(Arc::new(World {
            path,
            connection: Mutex::default(),
            guards: Default::default(),
        }))
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
enum Action {
    #[serde(rename = "cookiesInsert")]
    Cookies { amount: u16 },
    #[serde(rename = "cataclysmInsert")]
    Sacrifice,
    #[serde(rename = "circuitInsert")]
    Score { tag: String, country_code: String, duration: u32, checkpoint_timings: Vec<u32> },
    #[serde(rename = "whispersInsert")]
    Whisper { visitor: String, message: String, country_code: String, x: f64, y: f64, z: f64 },
}

fn valid_flag(s: &str) -> bool {
    s.is_empty() || (s.len() == 2 && s.bytes().all(|c| c.is_ascii_alphabetic()))
}

impl Action {
    /// Guard index and minimum time between submissions of this kind.
    fn policy(&self) -> Result<(usize, u64), StatusCode> {
        match self {
            Self::Cookies {
                amount: 1..=60,
            } => Ok((0, 1)),
            Self::Sacrifice => Ok((1, 3)),
            Self::Score {
                tag,
                country_code,
                duration,
                checkpoint_timings,
            } if tag.len() == 3
                && tag.bytes().all(|c| c.is_ascii_uppercase())
                && valid_flag(country_code)
                && (1_000..=3_600_000).contains(duration)
                && (2..=64).contains(&checkpoint_timings.len())
                && checkpoint_timings.windows(2).all(|w| w[0] < w[1])
                && checkpoint_timings
                    .last()
                    .is_some_and(|last| *last <= *duration && duration - last <= 2000) =>
            {
                Ok((2, 5))
            },
            Self::Whisper {
                visitor,
                message,
                country_code,
                x,
                y,
                z,
            } if visitor.len() == 36
                && uuid::Uuid::try_parse(visitor).is_ok()
                && !message.trim().is_empty()
                && message.chars().count() <= 30
                && !message.chars().any(char::is_control)
                && valid_flag(country_code)
                && [x, y, z].into_iter().all(|v| v.abs() <= 1000.) =>
            {
                Ok((3, 60))
            },
            _ => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }
}

async fn read(State(world): State<Arc<World>>) -> Result<Json<Value>, StatusCode> {
    query(world, None).await
}

async fn update(
    State(world): State<Arc<World>>,
    headers: HeaderMap,
    Json(action): Json<Action>,
) -> Result<Json<Value>, StatusCode> {
    let (kind, seconds) = action.policy()?;
    let key = build_submit_rate_limit_key(&headers, &build_client_fingerprint(&headers));
    enforce_public_submit_rate_limit(
        &world.guards[kind],
        &key,
        chrono::Utc::now().timestamp_millis(),
        seconds,
        "World interaction",
    )
    .map_err(|(status, _)| status)?;
    query(world, Some(action)).await
}

async fn query(world: Arc<World>, action: Option<Action>) -> Result<Json<Value>, StatusCode> {
    tokio::task::spawn_blocking(move || {
        let mut guard = world.connection.lock();
        let conn = match guard.as_mut() {
            Some(conn) => conn,
            None => {
                let conn = Connection::open(&world.path)?;
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS counters (id INTEGER PRIMARY KEY, cookies INTEGER \
                     NOT NULL, sacrifices INTEGER NOT NULL, storm_until INTEGER NOT NULL);
                    INSERT OR IGNORE INTO counters VALUES (1, 0, 0, 0);
                    CREATE TABLE IF NOT EXISTS whispers (id INTEGER PRIMARY KEY AUTOINCREMENT, \
                     visitor TEXT UNIQUE NOT NULL, message TEXT, flag TEXT, x REAL, y REAL, z \
                     REAL);
                    CREATE TABLE IF NOT EXISTS scores (id INTEGER PRIMARY KEY, day INTEGER, tag \
                     TEXT, flag TEXT, duration INTEGER);",
                )?;
                guard.insert(conn)
            },
        };
        snapshot(conn, action, chrono::Utc::now().timestamp_millis())
    })
    .await
    .map_err(|e| {
        tracing::error!(%e, "world database task failed");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .map(Json)
    .map_err(|e| {
        tracing::warn!(%e, "world database unavailable");
        StatusCode::SERVICE_UNAVAILABLE
    })
}

fn snapshot(conn: &mut Connection, action: Option<Action>, now: i64) -> rusqlite::Result<Value> {
    let day = now - now.rem_euclid(DAY_MS);
    let behavior = if action.is_some() {
        TransactionBehavior::Immediate
    } else {
        TransactionBehavior::Deferred
    };
    let tx = conn.transaction_with_behavior(behavior)?;
    match action {
        Some(Action::Cookies {
            amount,
        }) => {
            tx.execute("UPDATE counters SET cookies = cookies + ?1 WHERE id=1", [amount])?;
        },
        Some(Action::Sacrifice) => {
            // Every tenth sacrifice summons a one-minute storm for all visitors.
            tx.execute(
                "UPDATE counters SET sacrifices = sacrifices + 1, storm_until = CASE WHEN \
                 (sacrifices + 1) % 10 = 0 THEN ?1 ELSE storm_until END WHERE id=1",
                [now + 60_000],
            )?;
        },
        Some(Action::Whisper {
            visitor,
            message,
            country_code,
            x,
            y,
            z,
        }) => {
            tx.execute(
                "INSERT OR REPLACE INTO whispers(visitor,message,flag,x,y,z) VALUES(?,?,?,?,?,?)",
                params![visitor, message.trim(), country_code, x, y, z],
            )?;
            tx.execute(
                "DELETE FROM whispers WHERE id NOT IN (SELECT id FROM whispers ORDER BY id DESC \
                 LIMIT 30)",
                [],
            )?;
        },
        Some(Action::Score {
            tag,
            country_code,
            duration,
            ..
        }) => {
            tx.execute("INSERT INTO scores(day,tag,flag,duration) VALUES(?,?,?,?)", params![
                day,
                tag,
                country_code,
                duration
            ])?;
            tx.execute(
                "DELETE FROM scores WHERE id NOT IN (SELECT id FROM scores WHERE day=? ORDER BY \
                 duration,id LIMIT 10)",
                [day],
            )?;
        },
        None => {},
    }
    let (cookies, count, storm): (i64, i64, i64) =
        tx.query_row("SELECT cookies,sacrifices,storm_until FROM counters WHERE id=1", [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?;
    let whispers = tx
        .prepare("SELECT id,message,flag,x,y,z FROM whispers ORDER BY id")?
        .query_map([], |r| {
            Ok(json!({
                "id": r.get::<_,i64>(0)?,
                "message": r.get::<_,String>(1)?,
                "countrycode": r.get::<_,String>(2)?,
                "x": r.get::<_,f64>(3)?,
                "y": r.get::<_,f64>(4)?,
                "z": r.get::<_,f64>(5)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let scores = tx
        .prepare("SELECT tag,flag,duration FROM scores WHERE day=? ORDER BY duration,id")?
        .query_map([day], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, u32>(2)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    tx.commit()?;
    Ok(
        json!({"type":"init", "cookiesCount":cookies, "cataclysmCount":count, "cataclysmProgress":(count % 10) as f64 / 10., "cataclysmRunning":storm > now, "whispers":whispers, "circuitResetTime":day, "circuitLeaderboard":scores, "easterEggs":{}}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn two_slots_share_persistent_scores_messages_and_world_events() -> anyhow::Result<()> {
        let tmp = tempfile::tempdir()?;
        let db = tmp.path().join("world.sqlite3");
        let connect = || {
            Arc::new(World {
                path: db.clone(),
                connection: Mutex::default(),
                guards: Default::default(),
            })
        };
        let first = connect();
        let second = connect();
        let post = |body: Value| {
            let first = first.clone();
            async move {
                let action = serde_json::from_value(body).expect("game request");
                update(State(first), HeaderMap::new(), Json(action))
                    .await
                    .map(|response| response.0)
            }
        };
        let note = json!({"type":"whispersInsert","visitor":uuid::Uuid::new_v4().to_string(),"message":"图书馆见","countryCode":"","x":2,"y":1,"z":3});
        assert_eq!(
            post(note.clone()).await.expect("first message")["whispers"][0]["message"],
            "图书馆见"
        );
        assert_eq!(
            post(note.clone()).await.expect_err("repeat whisper"),
            StatusCode::TOO_MANY_REQUESTS
        );
        let mut moved = note;
        moved["message"] = json!("在赛道等你");
        let replaced = query(second.clone(), Some(serde_json::from_value(moved)?))
            .await
            .expect("move whisper");
        assert_eq!(replaced["whispers"].as_array().expect("message list").len(), 1);
        assert!(replaced["whispers"][0].get("visitor").is_none());
        let score = json!({"type":"circuitInsert","tag":"RUS","countryCode":"CN","duration":27000,"checkpointTimings":[230,11632,26990]});
        post(score).await.expect("race score");
        assert_eq!(
            post(json!({"type":"cookiesInsert","amount":0}))
                .await
                .expect_err("invalid cookie count"),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        post(json!({"type":"cookiesInsert","amount":3}))
            .await
            .expect("cookies");
        for _ in 0..10 {
            let _state = query(first.clone(), Some(Action::Sacrifice))
                .await
                .expect("sacrifice");
        }
        let state = query(second.clone(), None).await.expect("second slot");
        assert_eq!(state["cookiesCount"], 3);
        assert_eq!(state["circuitLeaderboard"], json!([["RUS", "CN", 27000]]));
        assert_eq!(state["whispers"][0]["message"], "在赛道等你");
        assert_eq!(state["cataclysmRunning"], true);
        drop(first);
        drop(second);
        assert_eq!(query(connect(), None).await.expect("reopen")["cataclysmCount"], 10);
        Ok(())
    }
}
