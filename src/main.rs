#![feature(min_type_alias_impl_trait)]
#![feature(generic_associated_types)]

mod db;
mod iterext;
mod model;

use anyhow::Context as _;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::reply::WithStatus;
use warp::Filter;

use crate::iterext::IterExt;
use crate::model::HeartBeat;

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

    match auth {
        Some(k) if k == key => {}
        _ => return Ok(with_status("Unauthorized", StatusCode::UNAUTHORIZED)),
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
pub(crate) struct HeartBeatJson {
    pub(crate) branch: Option<String>,
    pub(crate) category: Option<String>,
    pub(crate) entity: Option<String>,
    pub(crate) is_write: Option<bool>,
    pub(crate) language: Option<String>,
    pub(crate) lineno: Option<i32>,
    pub(crate) lines: Option<i32>,
    pub(crate) project: Option<String>,
    pub(crate) time: Option<f64>,
    pub(crate) user_agent: Option<String>,
    pub(crate) machine_name: Option<String>,
}

impl Into<HeartBeat> for HeartBeatJson {
    fn into(self) -> HeartBeat {
        use chrono::TimeZone;
        HeartBeat {
            branch: self.branch,
            category: self.category,
            entity: self.entity,
            is_write: self.is_write,
            language: self.language,
            lineno: self.lineno,
            lines: self.lines,
            project: self.project,
            user_agent: self.user_agent,
            machine_name: self.machine_name,
            time: self.time.map(|x| chrono::Utc.timestamp(x as _, 0)),
        }
    }
}
