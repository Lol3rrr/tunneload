# Tunneload
A simple and flexible Load-Balancer that can easily integrate with other Tunneler-Software

## CLI Options
Key | Default | Description
--- | --- | ---
--kube.traefik | disabled | Enables the Kubernetes-Traefik-Configurator
--kube.ingress | disabled | Enables the Kubernetes-Ingress-Configurator
--file-conf {path} | disabled | Enables the File-Configurator for the given file/directory
--webserver {port} | disabled | Enables the Webserver-Entrypoint on the given Port
--tunneler | disabled | Enables the Tunneler-Entrypoint
--metrics {port} | disabled | Exposes Prometheus metrics on the given port and `/metrics` path

## Environment-Variables
Key | Default | Description
--- | --- | ---
KEY_FILE | $HOME/.tunneler/key | The File where the Key is stored
SERVER_ADDR | localhost | The Address of the Tunneler-Server
SERVER_PORT | 8081 | The Port on which to bind the Client on the Tunneler-Server
UTIME | 30s | The Time to wait between configuration updates

## Idea
Originally [tunneler](https://github.com/Lol3rrr/tunneler) was designed to solve the problem of
exposing internal services, that were not reachable from the outside, by running a server-instance
on a simple public server that can accept connections and then forward it to any number of clients
running on the "private" Servers.
However this introduced at least one more new Connection that needs to be established before it gets
to actual Infra, like a simple load-balancer/router. This is where Tunneload comes in and replaces
the old load-balancer and directly integrates the Tunneler-Client allowing it to receive the requests
and then determine where to send it like the original load-balancer which removes the extra Connection/
Hop.
