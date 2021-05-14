use futures::{Stream, StreamExt};
use kube::{
    api::{ListParams, WatchEvent},
    Api,
};
use kube_runtime::watcher;
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
    T: Clone + kube::api::Meta + DeserializeOwned + 'static + Send,
{
    watcher: Pin<
        Box<
            dyn futures::Stream<
                    Item = Result<kube_runtime::watcher::Event<T>, kube_runtime::watcher::Error>,
                > + Send,
        >,
    >,
}

impl<T> Watcher<T>
where
    T: Clone + kube::api::Meta + DeserializeOwned + 'static + Send,
{
    fn create_watcher(
        api: Api<T>,
        params: ListParams,
    ) -> Pin<
        Box<
            dyn futures::Stream<
                    Item = Result<kube_runtime::watcher::Event<T>, kube_runtime::watcher::Error>,
                > + Send,
        >,
    > {
        watcher(api, params).boxed()
    }

    async fn create_stream(
        api: &mut Api<T>,
        params: &ListParams,
        latest_version: &str,
    ) -> kube::Result<WatchEventStream<T>> {
        Ok(api.watch(&params, &latest_version).await?.boxed())
    }

    pub async fn from_api(
        tmp_api: Api<T>,
        params: Option<ListParams>,
    ) -> Result<Self, kube::Error> {
        let lp: ListParams = params.unwrap_or_default();

        let watcher = Self::create_watcher(tmp_api, lp);

        Ok(Self { watcher })
    }

    pub async fn next_event(&mut self) -> Option<Event<T>> {
        let raw_next = match self.watcher.next().await {
            Some(n) => n,
            None => {
                log::info!("Received None from Event");
                return None;
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
            kube_runtime::watcher::Event::Applied(tmp) => Some(Event::Updated(tmp)),
            kube_runtime::watcher::Event::Deleted(tmp) => Some(Event::Removed(tmp)),
            kube_runtime::watcher::Event::Restarted(all_applied) => {
                // TODO
                log::info!("Restarted Watcher");
                Some(Event::Other)
            }
        }
    }
}
