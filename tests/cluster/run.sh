trap "exit" INT TERM
trap "kill 0" EXIT

cargo build

export RUST_LOG=tunneload=info
./target/debug/tunneload --webserver=8081 --file-conf=examples/testing/config.yaml --auto-tls.enable --auto-tls.file.path=tests/cluster/config.yaml --auto-tls.cluster.port=3300 --kube.traefik &
./target/debug/tunneload --webserver=8082 --file-conf=examples/testing/config.yaml --auto-tls.enable --auto-tls.file.path=tests/cluster/config.yaml --auto-tls.cluster.port=3301 --kube.traefik &
./target/debug/tunneload --webserver=8083 --file-conf=examples/testing/config.yaml --auto-tls.enable --auto-tls.file.path=tests/cluster/config.yaml --auto-tls.cluster.port=3302 --kube.traefik &

wait
