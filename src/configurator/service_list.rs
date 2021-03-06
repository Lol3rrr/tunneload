use crate::{general::Shared, rules::Service};

#[derive(Debug, Clone)]
pub struct ServiceList(std::sync::Arc<std::sync::Mutex<Vec<Shared<Service>>>>);

impl ServiceList {
    pub fn new() -> Self {
        let services = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        Self(services)
    }

    pub fn set_service(&self, n_srv: Service) {
        let mut inner = self.0.lock().unwrap();

        for tmp in inner.iter() {
            let tmp_value = tmp.get();
            if tmp_value.name() == n_srv.name() {
                tmp.update(n_srv);
                return;
            }
        }

        inner.push(Shared::new(n_srv));
    }

    pub fn get_service(&self, name: &str) -> Option<Shared<Service>> {
        let inner = self.0.lock().unwrap();

        for tmp in inner.iter() {
            let value = tmp.get();
            if value.name() == name {
                return Some(tmp.clone());
            }
        }

        None
    }
}
