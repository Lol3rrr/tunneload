mod general;
mod ingress;
mod traefik;

pub async fn run() {
    general::load_service().await;

    traefik::load_middleware().await;
    traefik::load_rules().await;
}
