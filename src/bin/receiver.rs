use anyhow::Context as _;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::reply::WithStatus;
use warp::Filter;

use wk::db;
use wk::iterext::IterExt;
use wk::model::HeartBeat;

fn env(key: &str) -> anyhow::Result<String> {
    std::env::var(key).with_context(|| format!("failed to get \"{}\" environment variable", key))
}

fn inject<T>(t: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone
where
    T: Send + Sync + Clone,
{
    warp::any().map(move || t.clone())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let use_ansi = env("NO_COLOR").is_err();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(use_ansi)
        .init();

    let key = format!("Basic {}", base64::encode(env("KEY")?.as_bytes()));
    let key: &'static str = Box::leak(key.into_boxed_str()); // :/

    let mongodb_uri = env("MONGODB_URI")?;
    let db = db::mongo::MongoDb::new(&mongodb_uri)
        .await
        .context("failed to get mongodb instance")?;

    let route = warp::post()
        .and(warp::filters::body::json())
        .and(warp::filters::header::optional("x-machine-name"))
        .and(warp::filters::header::optional("Authorization"))
        .and(inject(db))
        .and(inject(key))
        .and_then(on_heartbeat)
        .with(warp::filters::trace::request());

    warp::serve(route).bind(([0, 0, 0, 0], 3000)).await;

    Ok(())
}

async fn on_heartbeat(
    msg: Vec<HeartBeatJson>,
    machine: Option<String>,
    auth: Option<String>,
    mut db: impl db::HeartBeatRepository,
    key: &str,
) -> Result<WithStatus<&'static str>, Infallible> {
    use warp::reply::with_status;

    if !matches!(auth, Some(k) if k == key) {
        return Ok(with_status("Unauthorized", StatusCode::UNAUTHORIZED));
    }

    let iter = msg.into_iter().map(|x| x.into());

    let result = {
        if let Some(m) = machine {
            let iter = iter.edit(move |x: &mut HeartBeat| {
                x.machine_name = Some(m.clone());
            });

            db.save(iter).await
        } else {
            db.save(iter).await
        }
    };

    match result {
        Ok(_) => Ok(with_status("Ok", StatusCode::OK)),
        Err(e) => {
            tracing::error!("DB Error: {:#?}", e);
            Ok(with_status(
                "Internal Server Error",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct HeartBeatJson {
    branch: Option<String>,
    category: Option<String>,
    entity: Option<String>,
    is_write: Option<bool>,
    language: Option<String>,
    lineno: Option<i32>,
    lines: Option<i32>,
    project: Option<String>,
    time: Option<f64>,
    user_agent: Option<String>,
    machine_name: Option<String>,
}

impl From<HeartBeatJson> for HeartBeat {
    fn from(h: HeartBeatJson) -> Self {
        use chrono::TimeZone;
        HeartBeat {
            branch: h.branch,
            category: h.category,
            entity: h.entity,
            is_write: h.is_write,
            language: h.language,
            lineno: h.lineno,
            lines: h.lines,
            project: h.project,
            user_agent: h.user_agent,
            machine_name: h.machine_name,
            time: h.time.map(|x| chrono::Utc.timestamp(x as _, 0)),
        }
    }
}
