use crate::HeartBeat;
use anyhow::Result;
use std::future::Future;

pub(crate) trait HeartBeatRepository {
    type SaveFuture<'a, I: 'a>: Future<Output = Result<()>> + 'a;

    fn save<'a, I>(&'a mut self, heartbeats: I) -> Self::SaveFuture<'a, I>
    where
        I: Iterator<Item = HeartBeat> + 'a;
}

pub(crate) mod mongo {
    use super::*;
    use anyhow::{Context as _, Result};
    use mongodb::options::ClientOptions;
    use mongodb::{Client, Collection};

    #[derive(Clone)]
    pub(crate) struct MongoDb {
        inner: Collection<MongoHeartBeat>,
    }

    impl MongoDb {
        const DATABASE_NAME: &'static str = "wk";
        const COLLECTION_NAME: &'static str = "heartbeat";

        pub(crate) async fn new(url: &str) -> Result<Self> {
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
    pub(crate) struct MongoHeartBeat {
        pub(crate) branch: Option<String>,
        pub(crate) category: Option<String>,
        pub(crate) entity: Option<String>,
        pub(crate) is_write: Option<bool>,
        pub(crate) language: Option<String>,
        pub(crate) lineno: Option<i32>,
        pub(crate) lines: Option<i32>,
        pub(crate) project: Option<String>,
        pub(crate) time: Option<bson::DateTime>,
        pub(crate) user_agent: Option<String>,
        pub(crate) machine_name: Option<String>,
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
