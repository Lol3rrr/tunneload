# Tunneload
[Internal Rust-Docs](https://lol3rrr.github.io/tunneload/tunneload/index.html)
A simple and flexible Load-Balancer that can easily integrate with other Tunneler-Software

## CLI Options
Key | Default | Description
--- | --- | ---
--dashboard | disabled | Enables the internal Dashboard-Service
--kube.traefik | disabled | Enables the Kubernetes-Traefik-Configurator
--kube.ingress | disabled | Enables the Kubernetes-Ingress-Configurator
--kube.ingress.priority {new priority} | 100 | The Priority to use for Routes loaded from the Kubernetes-Ingress-Configurator
--file-conf {path} | disabled | Enables the File-Configurator for the given file/directory
--webserver {port} | disabled | Enables the Webserver-Entrypoint on the given Port
--webserver.tls {port} | disabled | Enables the TLS version of the Webserver-Entrypoint on the given Port
--metrics {port} | disabled | Exposes Prometheus metrics on the given port and `/metrics` path
--tunneler | disabled | Enables the Tunneler-Entrypoint
--tunneler.key | $HOME/.tunneler/key | The File where the Tunneler-Key is stored
--tunneler.addr | localhost | The Address of the Tunneler-Server
--tunneler.port | 8081 | The Port on which to bind the Client on the Tunneler-Server
--tunneler.tls | disabled | Enables the Tunneler-Entrypoint with TLS enabled
--tunneler.key.tls | $HOME/.tunneler/key | The File where the Tunneler-Key is stored
--tunneler.addr.tls | localhost | The Address of the Tunneler-Server
--tunneler.port.tls | 8081 | The Port on which to bind the Client on the Tunneler-Server

## Environment-Variables
Key | Default | Description
--- | --- | ---
THREADS | 6 | The Number of threads the Runtime should use
RUST_LOG | tunneload=info | The Logging Level to use
RUST_LOG_COLOR | false | Whether or not the output should be color coded

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


## The Dashboard
The Dashboard is written using Svelte.

### Development
* navigate to 'src/internal_services/dashboard/website' using `cd src/internal_services/dashboard/website`
* Start the development server using `npm run dev`
