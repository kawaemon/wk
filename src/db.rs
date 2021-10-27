use crate::model::HeartBeat;
use anyhow::Result;
use std::future::Future;

pub trait HeartBeatRepository {
    type SaveFuture<'a, I: 'a>: Future<Output = Result<()>> + 'a;

    fn save<'a, I>(&'a mut self, heartbeats: I) -> Self::SaveFuture<'a, I>
    where
        I: Iterator<Item = HeartBeat> + 'a;
}

pub mod mongo {
    use super::*;
    use anyhow::{Context as _, Result};
    use mongodb::options::ClientOptions;
    use mongodb::{Client, Collection};

    #[derive(Clone)]
    pub struct MongoDb {
        inner: Collection<MongoHeartBeat>,
    }

    impl MongoDb {
        const DATABASE_NAME: &'static str = "wk";
        const COLLECTION_NAME: &'static str = "heartbeat";

        pub async fn new(url: &str) -> Result<Self> {
            let opt = ClientOptions::parse(url)
                .await
                .context("failed to parse mongodb url")?;

            let collection = Client::with_options(opt)
                .context("failed to create mongodb client")?
                .database(Self::DATABASE_NAME)
                .collection(Self::COLLECTION_NAME);

            Ok(Self { inner: collection })
        }
    }

    impl HeartBeatRepository for MongoDb {
        type SaveFuture<'a, I: 'a> = impl Future<Output = Result<()>> + 'a;

        fn save<'a, I>(&'a mut self, heartbeats: I) -> Self::SaveFuture<'a, I>
        where
            I: Iterator<Item = HeartBeat> + 'a,
        {
            async move {
                let heartbeats = heartbeats.map(MongoHeartBeat::from_heartbeat);

                self.inner
                    .insert_many(heartbeats, None)
                    .await
                    .context("failed to put heartbeats to mongodb")?;

                Ok(())
            }
        }
    }

    #[derive(Debug, Clone, serde::Serialize)]
    struct MongoHeartBeat {
        branch: Option<String>,
        category: Option<String>,
        entity: Option<String>,
        is_write: Option<bool>,
        language: Option<String>,
        lineno: Option<i32>,
        lines: Option<i32>,
        project: Option<String>,
        time: Option<bson::DateTime>,
        user_agent: Option<String>,
        machine_name: Option<String>,
    }

    impl MongoHeartBeat {
        fn from_heartbeat(origin: HeartBeat) -> MongoHeartBeat {
            MongoHeartBeat {
                branch: origin.branch,
                category: origin.category,
                entity: origin.entity,
                is_write: origin.is_write,
                language: origin.language,
                lineno: origin.lineno,
                lines: origin.lines,
                project: origin.project,
                user_agent: origin.user_agent,
                machine_name: origin.machine_name,
                time: origin.time.map(|x| x.into()),
            }
        }
    }
}
