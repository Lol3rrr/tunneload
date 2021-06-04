use std::{collections::HashMap, sync::atomic};

use serde::{Deserialize, Serialize};

use crate::{
    configurator::{RuleList, ServiceList},
    rules::{Matcher, Rule},
    tls::auto::{
        consensus::{Action, Request},
        CertificateQueue, CertificateRequest, ChallengeList, ChallengeState,
    },
};

/// This struct holds all the Data to properly interact with Tunneload
///
/// This should not be serialized as it does not directly contain any
/// sort of configuration and is just for communication with Tunneload and
/// is therefore unique to each individual Instance.
#[derive(Debug)]
struct InternalState {
    /// The current RuleList used by Tunneload
    pub rules: RuleList,
    /// The current ServiceList used by Tunneload
    pub service_list: ServiceList,
    /// The Queue to request new Certificates
    pub cert_queue: CertificateQueue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateMachine {
    last_applied_log: atomic::AtomicU64,

    // This holds all the Challenge Entries
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
            service_list: services,
            cert_queue,
        });

        Self {
            last_applied_log: atomic::AtomicU64::new(0),
            challenges,
            internal,
        }
    }

    pub fn update_last_log(&self, value: u64) {
        self.last_applied_log.store(value, atomic::Ordering::SeqCst);
    }

    pub fn last_log(&self) -> u64 {
        self.last_applied_log.load(atomic::Ordering::SeqCst)
    }

    pub fn replace_challenges(&self, map: HashMap<String, ChallengeState>) {
        self.challenges.set_map(map);
    }
    pub fn clone_challenges(&self) -> HashMap<String, ChallengeState> {
        self.challenges.clone_map()
    }

    fn generate_rule_name(domain: &str) -> String {
        format!("ACME-{}", domain)
    }

    pub fn apply(&self, data: &Request) {
        let domain = data.domain_name.clone();
        match &data.action {
            Action::MissingCert => {
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
            Action::VerifyingData(data) => {
                let n_state = ChallengeState::Data(data.clone());
                self.challenges.update_state(domain.clone(), n_state);

                log::warn!("Got Verifying Data for domain: {:?}", domain);

                // Generate Rules and update the internal
                let n_matcher = Matcher::And(vec![
                    Matcher::Domain(domain.clone()),
                    Matcher::PathPrefix("/.well-known/".to_owned()),
                ]);

                let internal = self.internal.as_ref().unwrap();

                let service = internal
                    .service_list
                    .get("acme@internal")
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
            Action::Failed => {
                self.challenges.remove_state(&domain);

                let internals = self.internal.as_ref().unwrap();
                internals
                    .rules
                    .remove_rule(Self::generate_rule_name(&domain));
            }
            Action::Finish => {
                let n_state = ChallengeState::Finished;
                self.challenges.update_state(domain.clone(), n_state);

                let internals = self.internal.as_ref().unwrap();
                internals
                    .rules
                    .remove_rule(Self::generate_rule_name(&domain));
            }
        };
    }
}
