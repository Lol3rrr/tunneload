use futures::{Stream, StreamExt};
use kube::{
    api::{ListParams, WatchEvent},
    Api,
};
use serde::de::DeserializeOwned;
use std::pin::Pin;

pub enum Event<T> {
    Updated(T),
    Removed(T),
    Other,
}

type WatchEventStream<T> = Pin<Box<dyn Stream<Item = Result<WatchEvent<T>, kube::Error>> + Send>>;

pub struct Watcher<T>
where
    T: Clone + kube::api::Meta + DeserializeOwned + 'static,
{
    api: Api<T>,
    list_params: ListParams,
    stream: WatchEventStream<T>,
    latest_version: String,
}

impl<T> Watcher<T>
where
    T: Clone + kube::api::Meta + DeserializeOwned + 'static,
{
    async fn create_stream(
        api: &mut Api<T>,
        params: &ListParams,
        latest_version: &str,
    ) -> kube::Result<WatchEventStream<T>> {
        Ok(api.watch(&params, &latest_version).await?.boxed())
    }

    pub async fn from_api(
        mut tmp_api: Api<T>,
        params: Option<ListParams>,
    ) -> Result<Self, kube::Error> {
        let lp: ListParams = params.unwrap_or_default();
        let latest_version = "0".to_owned();

        let stream = Self::create_stream(&mut tmp_api, &lp, &latest_version).await?;

        Ok(Self {
            api: tmp_api,
            list_params: lp,
            stream,
            latest_version,
        })
    }

    pub async fn next_event(&mut self) -> Option<Event<T>> {
        let raw_next = match self.stream.next().await {
            Some(n) => n,
            None => {
                self.stream = match Self::create_stream(
                    &mut self.api,
                    &self.list_params,
                    &self.latest_version,
                )
                .await
                {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("Could not create new Watch-Stream: {}", e);
                        return None;
                    }
                };

                match self.stream.next().await {
                    Some(n) => n,
                    None => {
                        log::error!("Could not get Data from new Stream");
                        return None;
                    }
                }
            }
        };

        let event_data = match raw_next {
            Ok(d) => d,
            Err(e) => {
                log::error!("Getting Stream-Data: {}", e);
                return None;
            }
        };

        match event_data {
            WatchEvent::Added(tmp) | WatchEvent::Modified(tmp) => Some(Event::Updated(tmp)),
            WatchEvent::Deleted(tmp) => Some(Event::Removed(tmp)),
            WatchEvent::Bookmark(_) | WatchEvent::Error(_) => Some(Event::Other),
        }
    }
}
