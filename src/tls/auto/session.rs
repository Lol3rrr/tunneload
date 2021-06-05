use std::{sync::Arc, time::Duration};

use acme2::OrderStatus;
use async_raft::raft::ClientWriteResponse;
use lazy_static::lazy_static;
use prometheus::Registry;
use tokio::sync::OnceCell;
use tokio_stream::StreamExt;

use crate::{
    configurator::{RuleList, ServiceList},
    tls::ConfigManager,
};

use super::{
    cluster::{self, Cluster, ClusterResponse},
    Account, AutoDiscover, CertificateQueue, CertificateRequest, ChallengeList, Environment,
    StoreTLS,
};

lazy_static! {
    static ref RAFT_ACME_NODES: prometheus::IntGauge = prometheus::IntGauge::new(
        "acme_raft_nodes",
        "The Number of Nodes in the Raft-Cluster responsible for the ACME-Auto-TLS"
    )
    .unwrap();
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

/// Registers all the related Metrics
pub fn register_metrics(registry: &Registry) {
    registry
        .register(Box::new(RAFT_ACME_NODES.clone()))
        .unwrap();
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
        S: StoreTLS,
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
                    log::error!("Could not set the ACME-Account");
                    return None;
                }

                self.acme_acc.get()
            }
            None => None,
        }
    }

    /// Checks if the current Node is the Leader, if not notify others about
    /// the missing Certificate and then return
    async fn generate_domain_1(&self, domain: String, propagate: bool) -> Result<(), ()> {
        // Check if the Node is NOT the leader
        //
        // If this Node is not the leader:
        // * it should notify the rest of the Nodes about the Missing-Cert
        if self.cluster.is_leader().await {
            if !propagate {
                return Err(());
            }

            match self
                .cluster
                .write(domain.clone(), cluster::ClusterAction::MissingCert)
                .await
            {
                Ok(_) => {
                    log::info!("Notified Cluster about missing Domain-Cert: {:?}", &domain);
                }
                Err(e) => {
                    log::error!(
                        "Error notifying Cluster about the Missing Domain-Cert: {:?}",
                        e
                    );
                }
            };

            return Err(());
        }

        Ok(())
    }

    async fn write_verifying_data(
        &self,
        domain: String,
        parts: Vec<(String, String)>,
    ) -> Result<ClientWriteResponse<ClusterResponse>, ()> {
        self.cluster
            .write(domain, cluster::ClusterAction::AddVerifyingData(parts))
            .await
    }

    async fn write_failed_cert(
        &self,
        domain: String,
    ) -> Result<ClientWriteResponse<ClusterResponse>, ()> {
        self.cluster
            .write(domain, cluster::ClusterAction::RemoveVerifyingData)
            .await
    }

    // TODO
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
    async fn generate_domain<S>(&self, request: CertificateRequest, storage: &S)
    where
        S: StoreTLS + Sync + Send + 'static,
    {
        let domain = request.domain().to_owned();
        if self
            .generate_domain_1(domain.clone(), request.propagate())
            .await
            .is_err()
        {
            return;
        }

        let acme_acc = match self.get_acme_account(storage).await {
            Some(acc) => acc,
            None => {
                log::error!("Could not get ACME-Account to generate Certificate");
                return;
            }
        };

        let (order, verify_messages) = match acme_acc.generate_verify(domain.to_owned()).await {
            Some(v) => v,
            None => {
                log::error!("Could not generate Order");
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
            log::error!("Error Sending VerifyingData: {:?}", e);
        }

        // Start the verification for the Domain
        for (_, challenge) in verify_messages.iter() {
            match challenge.validate().await {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Error starting Validation: {:?}", e);
                }
            };
        }

        let order = order.wait_ready(Duration::from_secs(5), 3).await.unwrap();
        if order.status != OrderStatus::Ready {
            log::error!("Order did not become ready: {:?}", order.status);
            self.write_failed_cert(domain.clone()).await.unwrap();
            return;
        }

        let private_key = acme2::gen_rsa_private_key(4096).unwrap();
        let order = order
            .finalize(acme2::Csr::Automatic(private_key.clone()))
            .await
            .unwrap();

        let order = order.wait_done(Duration::from_secs(5), 3).await.unwrap();
        if order.status != OrderStatus::Valid {
            log::error!("Order did not become Valid: {:?}", order.status);
            self.write_failed_cert(domain.clone()).await.unwrap();
            return;
        }

        if let Err(e) = self
            .cluster
            .write(domain.clone(), cluster::ClusterAction::RemoveVerifyingData)
            .await
        {
            log::error!("Notifying Cluster about finished Status");
            return;
        }

        // These are the final certificates
        let certs = order.certificate().await.unwrap().unwrap();

        // Store the Certificate in all the needed Places
        for cert in certs.iter() {
            storage.store(domain.clone(), &private_key, cert).await;
        }
    }

    async fn listen<S>(mut self, storage: S)
    where
        S: StoreTLS + Send + Sync + 'static,
    {
        // Waiting 30s before actually doing anything to allow the system to fully
        // get up and running with everything
        tokio::time::sleep(Duration::from_secs(30)).await;

        loop {
            let request = match self.rx.recv().await {
                Some(d) => d,
                None => {
                    log::error!("Certificate Queue has been stopped");
                    return;
                }
            };

            let domain = request.domain();
            if self.tls_config.contains_cert(domain) {
                continue;
            }

            self.generate_domain(request, &storage).await;
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
        }
    }

    /// Starts a background-task, which will try to obtain Certificates for all the
    /// Domains it receives over the given Channel
    pub fn start<S>(self, stores: S) -> CertificateQueue
    where
        S: StoreTLS + Sync + Send + 'static,
    {
        let metrics = self.cluster.metrics();
        tokio::task::spawn(Self::run_metrics(metrics));

        tokio::task::spawn(self.cluster.clone().start());

        let tx = self.tx.clone();
        tokio::task::spawn(self.listen(stores));

        tx
    }
}
