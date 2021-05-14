use futures::StreamExt;
use kube::{api::ListParams, Api};
use kube_runtime::watcher;
use serde::de::DeserializeOwned;
use std::pin::Pin;

/// The Events returned by the Watcher
pub enum Event<T> {
    /// The provided Entity was either newly created
    /// or Updated and the given State is the newest
    /// known State that should be used
    Updated(T),
    /// A given Entity was removed from the Cluster
    Removed(T),
    /// Some other unknown Event occured, this can
    /// mostly be ignored as it mainly functions
    /// as a fallback in case of new or unwanted
    /// Events
    Other,
}

/// A single custom Watcher that watches for any events
/// related to the given Ressource-Type and ListParams
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
    /// Creates a new Watcher with the given Parameters
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

    /// Creates a new Watcher from the given Api and the
    /// given ListParams, if no Params were provided the
    /// default Params will be used
    pub async fn from_api(
        tmp_api: Api<T>,
        params: Option<ListParams>,
    ) -> Result<Self, kube::Error> {
        let lp: ListParams = params.unwrap_or_default();

        let watcher = Self::create_watcher(tmp_api, lp);

        Ok(Self { watcher })
    }

    /// Waits for the next Event from the Cluster regarding
    /// the Ressources
    ///
    /// # Returns:
    /// * Some(event): The given Event was successfully received
    /// and can be handled by the consumer
    /// * None: An unexpected Error occured and the Watcher could
    /// not be restarted properly
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
            kube_runtime::watcher::Event::Restarted(_all_applied) => {
                // TODO
                log::info!("Restarted Watcher");
                Some(Event::Other)
            }
        }
    }
}
