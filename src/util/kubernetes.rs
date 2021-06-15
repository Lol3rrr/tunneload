//! This module provides a couple of Utilities for interacting and using Kubernetes

pub mod watcher {
    //! This provides a Watcher for any Kubernetes-Ressource

    use futures::StreamExt;
    use kube::{api::ListParams, Api};
    use kube_runtime::watcher;
    use serde::de::DeserializeOwned;
    use std::pin::Pin;

    /// The Events returned by the Watcher
    #[derive(Debug)]
    pub enum Event<T> {
        /// The provided Entity was either newly created
        /// or Updated and the given State is the newest
        /// known State that should be used
        Updated(T),
        /// A given Entity was removed from the Cluster
        Removed(T),
        /// The initial List of entities that were already
        /// registered
        Started(Vec<T>),
        /// This is emited every time the Watcher needs to
        /// be restarted and can safely be ignored by the
        /// User
        Restarted,
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
        api: Api<T>,
        list_params: ListParams,
        watcher: Pin<
            Box<
                dyn futures::Stream<
                        Item = Result<
                            kube_runtime::watcher::Event<T>,
                            kube_runtime::watcher::Error,
                        >,
                    > + Send,
            >,
        >,
        started: bool,
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
                        Item = Result<
                            kube_runtime::watcher::Event<T>,
                            kube_runtime::watcher::Error,
                        >,
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

            let watcher = Self::create_watcher(tmp_api.clone(), lp.clone());

            Ok(Self {
                watcher,
                list_params: lp,
                started: false,
                api: tmp_api,
            })
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
                Err(e) => match e {
                    watcher::Error::WatchError {
                        source,
                        backtrace: _,
                    } => {
                        return if source.reason == "Expired" {
                            self.watcher =
                                Self::create_watcher(self.api.clone(), self.list_params.clone());
                            Some(Event::Restarted)
                        } else {
                            log::error!("Received Watcher-Error: {}", source);
                            None
                        };
                    }
                    _ => {
                        log::error!("Getting Stream-Data: {}", e);
                        return None;
                    }
                },
            };

            match event_data {
                kube_runtime::watcher::Event::Applied(tmp) => Some(Event::Updated(tmp)),
                kube_runtime::watcher::Event::Deleted(tmp) => Some(Event::Removed(tmp)),
                kube_runtime::watcher::Event::Restarted(all) if !self.started => {
                    self.started = true;
                    Some(Event::Started(all))
                }
                kube_runtime::watcher::Event::Restarted(_) => Some(Event::Other),
            }
        }
    }
}

pub mod secret {
    //! This module provides convience functions to interact with Secrets
    //! in Kubernetes

    use k8s_openapi::{api::core::v1::Secret, ByteString};
    use kube::Api;

    /// Attempts to load the Secret from the connected Kubernetes
    /// Cluster
    pub async fn load_secret(
        client: kube::Client,
        namespace: &str,
        secret_name: &str,
    ) -> Option<std::collections::BTreeMap<String, ByteString>> {
        let secrets: Api<Secret> = Api::namespaced(client, namespace);

        let secret = match secrets.get(secret_name).await {
            Ok(r) => r,
            Err(e) => {
                log::error!("Loading Secret: {}", e);
                return None;
            }
        };

        secret.data
    }

    /// The Type of Secrets in Kubernetes for TLS-Certificates
    pub const TLS_TYPE: &str = "kubernetes.io/tls";
    /// The Annotation-Name for the Domain used by Cert-Manager
    pub const TLS_DOMAIN_KEY_CERT_MANAGER: &str = "cert-manager.io/common-name";
    /// The Annotation-Name for the Domain used by Tunneload
    pub const TLS_DOMAIN_KEY_TUNNELOAD: &str = "tunneload/common-name";

    /// Loads the Domain from the given TLS-Secret
    pub fn tls_domain(secret: &Secret) -> Option<String> {
        if secret.type_.as_ref()? != TLS_TYPE {
            return None;
        }

        let annotations = secret.metadata.annotations.as_ref()?;

        if let Some(domain) = annotations.get(TLS_DOMAIN_KEY_CERT_MANAGER) {
            return Some(domain.clone());
        }
        if let Some(domain) = annotations.get(TLS_DOMAIN_KEY_TUNNELOAD) {
            return Some(domain.clone());
        }
        None
    }
}
