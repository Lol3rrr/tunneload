use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
    time::Duration,
};

use acme2::OrderStatus;
use async_raft::raft::ClientWriteResponse;
use lazy_static::lazy_static;
use prometheus::Registry;
use tokio::sync::OnceCell;
use tokio_stream::StreamExt;
use tracing::Level;

use crate::{
    configurator::{RuleList, ServiceList},
    tls::ConfigManager,
};

use super::{
    Account, AutoDiscover, CertificateQueue, CertificateRequest, ChallengeList, Environment,
};

use tls::TLSStorage;

mod cluster;
pub use cluster::addr_to_id;
use cluster::{Cluster, ClusterResponse, WriteError};

lazy_static! {
    static ref RAFT_ACME_NODES: prometheus::IntGauge = prometheus::IntGauge::new(
        "acme_raft_nodes",
        "The Number of Nodes in the Raft-Cluster responsible for the ACME-Auto-TLS"
    )
    .expect("Creating a Metric should always work");
    static ref RAFT_LEADER: prometheus::IntGauge = prometheus::IntGauge::new(
        "acme_raft_leader",
        "If the current Node is the Cluster Leader"
    )
    .expect("Creating a Metric should always work");
    static ref RAFT_TERM: prometheus::IntGauge =
        prometheus::IntGauge::new("acme_raft_term", "The current Term of the Raft-Cluster")
            .expect("Creating a Metric should always work");
}

/// Manages all the Auto-TLS-Session stuff
pub struct AutoSession<D> {
    env: Environment,
    contacts: Vec<String>,
    acme_acc: OnceCell<Account>,
    cluster: Arc<Cluster<D>>,
    tls_config: ConfigManager,
    tx: CertificateQueue,
    rx: tokio::sync::mpsc::UnboundedReceiver<CertificateRequest>,
}

impl<D> Debug for AutoSession<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AutoSession ()")
    }
}

/// Registers all the related Metrics
pub fn register_metrics(registry: &Registry) {
    if let Err(e) = registry.register(Box::new(RAFT_ACME_NODES.clone())) {
        tracing::error!("Registering RAFT_ACME_NODE metric: {:?}", e);
    }
    if let Err(e) = registry.register(Box::new(RAFT_LEADER.clone())) {
        tracing::error!("Registering RAFT_ACME_NODE metric: {:?}", e);
    }
    if let Err(e) = registry.register(Box::new(RAFT_TERM.clone())) {
        tracing::error!("Registering RAFT_ACME_NODE metric: {:?}", e);
    }
}

impl<D> AutoSession<D>
where
    D: AutoDiscover + Send + Sync + 'static,
{
    /// Creates a new AutoSession, that can be used to issue new Certificates
    /// when needed
    pub async fn new(
        env: Environment,
        contacts: Vec<String>,
        rules: RuleList,
        services: ServiceList,
        tls_config: ConfigManager,
        challenges: ChallengeList,
        discover: D,
        listen_port: u16,
    ) -> Self {
        let (tx, rx) = CertificateQueue::new();

        let cluster = Cluster::new(
            discover,
            listen_port,
            challenges,
            rules,
            services,
            tx.clone(),
        )
        .await;

        Self {
            env,
            contacts,
            acme_acc: OnceCell::new(),
            cluster,
            tls_config,
            tx,
            rx,
        }
    }

    async fn get_acme_account<S>(&self, storage: &S) -> Option<&Account>
    where
        S: TLSStorage,
    {
        if self.acme_acc.initialized() {
            return self.acme_acc.get();
        }

        let key = storage.load_acc_key().await;

        match Account::new(&self.env, self.contacts.clone(), key.clone()).await {
            Some(acc) => {
                if key.is_none() {
                    let priv_key = acc.private_key();
                    storage.store_acc_key(&priv_key).await;
                }

                if self.acme_acc.set(acc).is_err() {
                    return None;
                }

                self.acme_acc.get()
            }
            None => None,
        }
    }

    /// Notifies the Cluster about the missing Certificate for the given
    /// Domain
    async fn nofity_missing(&self, domain: String) {
        match self
            .cluster
            .write(domain.clone(), cluster::ClusterAction::MissingCert)
            .await
        {
            Ok(_) => {
                tracing::info!("Notified Cluster about missing Domain-Cert: {:?}", &domain);
            }
            Err(e) => {
                tracing::error!(
                    "Error notifying Cluster about the Missing Domain-Cert: {:?}",
                    e
                );
            }
        };
    }

    async fn write_verifying_data(
        &self,
        domain: String,
        parts: Vec<(String, String)>,
    ) -> Result<ClientWriteResponse<ClusterResponse>, WriteError> {
        self.cluster
            .write(domain, cluster::ClusterAction::AddVerifyingData(parts))
            .await
    }

    async fn write_failed_cert(
        &self,
        domain: String,
    ) -> Result<ClientWriteResponse<ClusterResponse>, WriteError> {
        self.cluster
            .write(domain, cluster::ClusterAction::RemoveVerifyingData)
            .await
    }

    // # Procedure
    // 1.
    // * Check if the current Node is leader
    //
    // 1. => False
    // * Send out message to the Raft-Cluster, that a new TLS-Certificate is desired
    // * return
    // 1. => True
    // * Continue
    //
    // 2.
    // * Generate the Verify-Messages
    // * Commit the Verify-Messages to the Raft-Cluster
    //
    // 3.
    // * Actually start the Verify Phase of the individual ACME-Challenges
    /// This starts the Generation of a new Certificate for a given Domain
    #[tracing::instrument]
    async fn generate_domain<S>(&self, request: CertificateRequest, storage: &S)
    where
        S: TLSStorage + std::fmt::Debug + Sync + Send + 'static,
    {
        let domain = request.domain().to_owned();
        if !self.cluster.is_leader().await {
            // If the Domain-Request should not be propagated, exit
            // early and dont notify the Cluster about it
            if !request.propagate() {
                return;
            }

            // Notify the Rest of the cluster about the missing
            // cerficate and then exit
            self.nofity_missing(domain).await;

            return;
        }

        tracing::info!("Starting Certificate Generation for {:?}", domain);

        let acme_acc = match self.get_acme_account(storage).await {
            Some(acc) => acc,
            None => {
                tracing::error!("Could not get ACME-Account to generate Certificate");
                // Notify the Cluster about the failure to generate a Certificate or in
                // this case even just to obtain an account to use for generation
                if let Err(e) = self.write_failed_cert(domain).await {
                    tracing::error!("Writing to Cluster: {:?}", e);
                }
                return;
            }
        };

        tracing::debug!("Generating Order and Verification");
        let (order, verify_messages) = match acme_acc.generate_verify(domain.to_owned()).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Generating Order: {:?}", e);
                // Notify the Cluster about the failure to generate a Certificate or in
                // this case to generate the Order for the Certificate
                if let Err(e) = self.write_failed_cert(domain).await {
                    tracing::error!("Writing to Cluster: {:?}", e);
                }
                return;
            }
        };

        // Store all the Parts in a List
        let mut verify_parts = Vec::new();
        for (pending, _) in verify_messages.iter() {
            verify_parts.push((pending.token().to_owned(), pending.key().to_owned()));
        }

        // Commit all the Pending-Parts for the Domain in one go
        if let Err(e) = self
            .write_verifying_data(domain.clone(), verify_parts)
            .await
        {
            tracing::error!("Error Sending VerifyingData: {:?}", e);
            return;
        }

        tracing::debug!("Starting Validation for Challenges");
        // Start the verification for the Domain
        for (_, challenge) in verify_messages.iter() {
            if let Err(e) = challenge.validate().await {
                tracing::error!("Starting Validation: {:?}", e);
            }
        }

        tracing::debug!("Waiting for Order to become Ready");
        let order = order
            .wait_ready(Duration::from_secs(5), 3)
            .await
            .expect("Failed to wait for the Order");
        if order.status != OrderStatus::Ready {
            tracing::error!("Order did not become ready: {:?}", order.status);
            // Notify the Cluster about the failure to validate the Certificate
            if let Err(e) = self.write_failed_cert(domain).await {
                tracing::error!("Writing to Cluster: {:?}", e);
            }
            return;
        }

        let private_key = acme2::gen_rsa_private_key(4096).expect("Creating the Private Key");
        let order = order
            .finalize(acme2::Csr::Automatic(private_key.clone()))
            .await
            .expect("Could not finalize Order");

        tracing::debug!("Waiting for Order to become Done");
        let order = order
            .wait_done(Duration::from_secs(5), 3)
            .await
            .expect("Order failed to become done");
        if order.status != OrderStatus::Valid {
            tracing::error!("Order did not become Valid: {:?}", order.status);
            // Notify the Cluster about the failure to validate the Certificate
            if let Err(e) = self.write_failed_cert(domain).await {
                tracing::error!("Writing to Cluster: {:?}", e);
            }
            return;
        }

        if let Err(e) = self
            .cluster
            .write(domain.clone(), cluster::ClusterAction::RemoveVerifyingData)
            .await
        {
            tracing::error!("Notifying Cluster about finished Status: {:?}", e);
            return;
        }

        // These are the final certificates
        let mut certs = order.certificate().await.expect("").expect("");

        // Store the generated Certificate
        if !certs.is_empty() {
            let cert = certs.remove(0);
            // Store the newly generated Certificate
            storage.store(domain.clone(), private_key, cert).await;
        }

        tracing::info!("Generated Certificate for {:?}", domain);
    }

    #[tracing::instrument]
    async fn handle_request<S>(&mut self, storage: &S) -> Result<(), ()>
    where
        S: TLSStorage + std::fmt::Debug + Send + Sync + 'static,
    {
        let request = match self.rx.recv().await {
            Some(r) => r,
            None => {
                tracing::event!(Level::ERROR, "Certificate Queue has been stopped");
                return Err(());
            }
        };

        let domain = request.domain();
        if self.tls_config.contains_cert(domain) {
            return Ok(());
        }

        self.generate_domain(request, storage).await;

        Ok(())
    }

    #[tracing::instrument]
    async fn listen<S>(mut self, storage: Arc<S>)
    where
        S: TLSStorage + std::fmt::Debug + Send + Sync + 'static,
    {
        // Waiting 20s before actually doing anything to allow the system to fully
        // get up and running with everything
        tokio::time::sleep(Duration::from_secs(20)).await;

        self.cluster.clone().initialize().await;

        tokio::time::sleep(Duration::from_secs(10)).await;

        loop {
            if self.handle_request(storage.as_ref()).await.is_err() {
                return;
            }
        }
    }

    async fn run_metrics(raw_recv: tokio::sync::watch::Receiver<async_raft::RaftMetrics>) {
        let mut stream = tokio_stream::wrappers::WatchStream::new(raw_recv);

        loop {
            let value = match stream.next().await {
                Some(v) => v,
                None => return,
            };

            let member_count = value.membership_config.members.len();
            RAFT_ACME_NODES.set(member_count as i64);

            let current_id = value.id;
            let is_leader = match value.current_leader {
                Some(leader) if leader == current_id => 1,
                _ => 0,
            };
            RAFT_LEADER.set(is_leader);

            let current_term = value.current_term;
            RAFT_TERM.set(current_term as i64);
        }
    }

    /// Starts a background-task, which will try to obtain Certificates for all the
    /// Domains it receives over the given Channel
    pub fn start<S>(self, stores: Arc<S>) -> CertificateQueue
    where
        S: TLSStorage + std::fmt::Debug + Sync + Send + 'static,
    {
        self.cluster.clone().start();

        let metrics = self.cluster.metrics();
        tokio::task::spawn(Self::run_metrics(metrics));

        let tx = self.tx.clone();
        tokio::task::spawn(self.listen(stores));

        tx
    }
}
