use std::{collections::HashMap, sync::atomic};

use crate::{
    configurator::{RuleList, ServiceList},
    tls::auto::{CertificateQueue, CertificateRequest, ChallengeList, ChallengeState},
};
use general::{Group, Name};
use rules::{Matcher, Rule};

use super::{ClusterAction, ClusterRequest};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct InternalState {
    pub rules: RuleList,
    pub services: ServiceList,
    pub cert_queue: CertificateQueue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateMachine {
    last_applied_log: atomic::AtomicU64,

    challenges: ChallengeList,

    #[serde(skip_serializing, skip_deserializing)]
    internal: Option<InternalState>,
}

impl StateMachine {
    pub fn new(
        challenges: ChallengeList,
        rules: RuleList,
        services: ServiceList,
        cert_queue: CertificateQueue,
    ) -> Self {
        let internal = Some(InternalState {
            rules,
            services,
            cert_queue,
        });

        Self {
            last_applied_log: atomic::AtomicU64::new(0),
            challenges,
            internal,
        }
    }
}

impl StateMachine {
    pub fn last_log(&self) -> u64 {
        self.last_applied_log.load(atomic::Ordering::SeqCst)
    }

    pub fn update_last_log(&self, index: u64) {
        self.last_applied_log.store(index, atomic::Ordering::SeqCst);
    }

    pub fn clone_challenges(&self) -> HashMap<String, ChallengeState> {
        self.challenges.clone_map()
    }

    pub fn replace_challenges(&self, challenges: HashMap<String, ChallengeState>) {
        self.challenges.set_map(challenges);
    }

    /// Generates the Name for the Rule to match the ACME-Challenge
    fn generate_rule_name(domain: &str) -> Name {
        let name = format!("ACME-{}", domain);
        Name::new(name, Group::Internal)
    }

    pub fn apply(&self, data: &ClusterRequest) {
        let domain = data.domain.clone();
        match &data.action {
            ClusterAction::MissingCert => {
                if self.challenges.get_state(&domain).is_some() {
                    return;
                }

                tracing::debug!("Received Missing-Cert for {:?}", domain);

                let n_state = ChallengeState::Pending;
                self.challenges.update_state(domain.clone(), n_state);

                let mut req = CertificateRequest::new(domain);
                req.disable_propagate();

                self.internal
                    .as_ref()
                    .unwrap()
                    .cert_queue
                    .custom_request(req);
            }
            ClusterAction::AddVerifyingData(data) => {
                tracing::debug!("Received Verifying Data for {:?}", domain);

                let n_state = ChallengeState::Data(data.clone());
                self.challenges.update_state(domain.clone(), n_state);

                // Generate Rules and update the internal
                let n_matcher = Matcher::And(vec![
                    Matcher::Domain(domain.clone()),
                    Matcher::PathPrefix("/.well-known/".to_owned()),
                ]);

                let internal = self.internal.as_ref().unwrap();

                let service = internal
                    .services
                    .get(&Name::new("acme", Group::Internal))
                    .expect("Internal ACME-Service not found");

                let n_rule = Rule::new(
                    Self::generate_rule_name(&domain),
                    100,
                    n_matcher,
                    Vec::new(),
                    service,
                );

                // Update the Rules
                internal.rules.set_rule(n_rule);
            }
            ClusterAction::RemoveVerifyingData => {
                tracing::debug!("Received Remove-Verifying Data for {:?}", domain);

                self.challenges.remove_state(&domain);

                let rule_name = Self::generate_rule_name(&domain);
                let internals = self.internal.as_ref().unwrap();
                internals.rules.remove_rule(rule_name);
            }
        };
    }
}
