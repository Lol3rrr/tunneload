mod general;
mod ingress;
mod traefik;

pub async fn run() {
    traefik::load_middleware().await;
    traefik::load_rules().await;
}
