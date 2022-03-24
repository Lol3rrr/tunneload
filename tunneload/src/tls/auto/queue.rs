/// A Request for a single new Certificate for the given Domain
#[derive(Debug, Clone, PartialEq)]
pub struct CertificateRequest {
    /// The Target domain for the Certificate
    domain: String,
    /// Whether or not the Cluster should be notified in case the
    /// current Node is not the Leader
    propagate: bool,
    /// Whether or not the Requested Domain is to be renewed or if the Domain
    /// needs an entirely new Certificate
    renew: bool,
}

impl CertificateRequest {
    /// Creates a new Request for the given Domain
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            propagate: true,
            renew: false,
        }
    }

    /// Disables the propagation of the CertificateRequest to the Rest of
    /// the Cluster, this should not be touched by normal consumers.
    pub fn disable_propagate(&mut self) {
        self.propagate = false;
    }

    /// Marks the Request as being intended to renew the Certificate for the
    /// given Domain instead of generating an entirely new one
    pub fn renew_cert(&mut self) {
        self.renew = true;
    }

    /// The Domain of the Certificate
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Whether or not the Request should be propagated
    pub fn propagate(&self) -> bool {
        self.propagate
    }

    /// Whether or not the Request is intended to renew the Certificate
    pub fn renew(&self) -> bool {
        self.renew
    }
}

/// The Queue for Requested Certificates
#[derive(Clone)]
pub struct CertificateQueue {
    tx: tokio::sync::mpsc::UnboundedSender<CertificateRequest>,
}

impl std::fmt::Debug for CertificateQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl CertificateQueue {
    /// Creates new Queue-Pair of Sender and Receiver
    pub fn new() -> (
        Self,
        tokio::sync::mpsc::UnboundedReceiver<CertificateRequest>,
    ) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        (Self { tx }, rx)
    }

    /// Adds a new Request for a Certificate for the given Domain to the Queue
    pub fn request(&self, domain: String) {
        let req = CertificateRequest::new(domain);
        if let Err(e) = self.tx.send(req) {
            tracing::error!("Could not add to CertificateQueue: {:?}", e);
        }
    }

    /// Adds the given Request to the Queue for Certificates
    pub fn custom_request(&self, req: CertificateRequest) {
        if let Err(e) = self.tx.send(req) {
            tracing::error!("Could not add to CertificateQueue: {:?}", e);
        }
    }
}
