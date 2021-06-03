use std::{collections::HashSet, sync::Arc, time::Duration};

use acme2::OrderStatus;
use async_raft::{
    raft::{ClientWriteRequest, ClientWriteResponse},
    ClientWriteError, NodeId, Raft,
};
use tokio::sync::OnceCell;
use tokio_stream::StreamExt;

use crate::{
    configurator::{RuleList, ServiceList},
    tls::ConfigManager,
};

use super::{
    consensus::{Action, Network, Request, Response, Storage},
    Account, CertificateQueue, CertificateRequest, ChallengeList, Environment, StoreTLS,
};

/// Manages all the Auto-TLS-Session stuff
pub struct AutoSession {
    env: Environment,
    contacts: Vec<String>,
    acme_acc: OnceCell<Account>,
    raft: async_raft::Raft<Request, Response, Network, Storage>,
    tls_config: ConfigManager,
    rx: tokio::sync::mpsc::UnboundedReceiver<CertificateRequest>,
    tx: CertificateQueue,
}

impl AutoSession {
    /// Creates a new AutoSession, that can be used to issue new Certificates
    /// when needed
    pub async fn new(
        env: Environment,
        contacts: Vec<String>,
        rules: RuleList,
        services: ServiceList,
        tls_config: ConfigManager,
        challenges: ChallengeList,
    ) -> Self {
        let id: u64 = rand::random();

        let (tx, rx) = CertificateQueue::new();

        let raw_config = async_raft::Config::build("Tunneload-TLS".to_owned())
            .validate()
            .expect("Failed to build Raft-Config");
        let config = Arc::new(raw_config);

        let network = Arc::new(Network::new());
        let storage = Arc::new(Storage::new(
            id,
            challenges,
            rules.clone(),
            services.clone(),
            tx.clone(),
        ));

        let raft = Raft::new(id, config, network, storage);

        let mut cluster_nodes: HashSet<NodeId> = HashSet::new();
        cluster_nodes.insert(id);

        if let Err(e) = raft.initialize(cluster_nodes).await {
            log::error!("Could not initialize the Cluster: {:?}", e);
        }

        Self {
            env,
            contacts,
            acme_acc: OnceCell::new(),
            raft,
            tls_config,
            rx,
            tx,
        }
    }

    async fn get_acme_account(&self) -> Option<&Account> {
        if self.acme_acc.initialized() {
            return self.acme_acc.get();
        }

        match Account::new(&self.env, self.contacts.clone()).await {
            Some(acc) => {
                if let Err(_) = self.acme_acc.set(acc) {
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
        if self.raft.client_read().await.is_err() {
            if !propagate {
                return Err(());
            }

            match self
                .raft
                .client_write(ClientWriteRequest::new(Request {
                    domain_name: domain.clone(),
                    action: Action::MissingCert,
                }))
                .await
            {
                Ok(_) => {
                    log::info!("Notified cluster about missing Domain-Cert: {:?}", &domain);
                }
                Err(e) => {
                    log::error!(
                        "Error notifying the Cluster about missing Domain-Cert:  {:?}",
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
    ) -> Result<ClientWriteResponse<Response>, ClientWriteError<Request>> {
        self.raft
            .client_write(ClientWriteRequest::new(Request {
                domain_name: domain,
                action: Action::VerifyingData(parts),
            }))
            .await
    }

    async fn write_failed_cert(
        &self,
        domain: String,
    ) -> Result<ClientWriteResponse<Response>, ClientWriteError<Request>> {
        self.raft
            .client_write(ClientWriteRequest::new(Request {
                domain_name: domain,
                action: Action::Failed,
            }))
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
    async fn generate_domain(
        &self,
        request: CertificateRequest,
        storage: &[Box<dyn StoreTLS + Sync + Send + 'static>],
    ) {
        let domain = request.domain().to_owned();
        if self
            .generate_domain_1(domain.clone(), request.propagate())
            .await
            .is_err()
        {
            return;
        }

        let acme_acc = match self.get_acme_account().await {
            Some(acc) => acc,
            None => {
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
            .raft
            .client_write(ClientWriteRequest::new(Request {
                domain_name: domain.clone(),
                action: Action::Finish,
            }))
            .await
        {
            log::error!("Notifying Cluster about finished Status: {:?}", e);
            return;
        }

        // These are the final certificates
        let certs = order.certificate().await.unwrap().unwrap();

        // Store the Certificate in all the needed Places
        for cert in certs.iter() {
            for store in storage.iter() {
                store.store(domain.clone(), &private_key, cert).await;
            }
        }
    }

    async fn listen(mut self, storage: Vec<Box<dyn StoreTLS + Sync + Send + 'static>>) {
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
                log::error!("Domain already has a Certificate: {:?}", domain);
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

            // TODO
            let member_count = value.membership_config.members.len();
            log::info!("Current-Member-Count: {:?}", member_count);
        }
    }

    /// Starts a background-task, which will try to obtain Certificates for all the
    /// Domains it receives over the given Channel
    pub fn start(self, stores: Vec<Box<dyn StoreTLS + Sync + Send + 'static>>) -> CertificateQueue {
        let metrics = self.raft.metrics();

        tokio::task::spawn(Self::run_metrics(metrics));

        let tx = self.tx.clone();
        tokio::task::spawn(self.listen(stores));

        tx
    }
}
