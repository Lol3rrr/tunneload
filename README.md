# Tunneload
[Internal Rust-Docs](https://lol3rrr.github.io/tunneload/tunneload/index.html)
A simple and flexible Load-Balancer that can easily integrate with other Tunneler-Software

## CLI Options
Key | Default | Description
--- | --- | ---
--dashboard={true|false} | disabled | Enables the internal Dashboard-Service
--kube.namespaces={name} | "default" | The Namespaces to use for the General Kubernetes-Configurator
--kube.traefik={true|false} | disabled | Enables the Kubernetes-Traefik-Configurator
--kube.traefik_namespaces={name} | "default" | The Namespaces to use for the Traefik Kubernetes-Configurator
--kube.ingress={true|false} | disabled | Enables the Kubernetes-Ingress-Configurator
--kube.ingress_priorit={new priority} | 100 | The Priority to use for Routes loaded from the Kubernetes-Ingress-Configurator
--kube.ingress_namespaces={name} | "default" | The Namespaces to use for the Ingress Kubernetes-Configurator
--file-conf={path} | disabled | Enables the File-Configurator for the given file/directory
--webserver.{name}.port={port} | disabled | Enables the Webserver-Entrypoint on the given Port
--webserver.{name}.tls={port} | disabled | Enables the TLS version of the Webserver-Entrypoint on the given Port
--metrics={port} | disabled | Exposes Prometheus metrics on the given port and `/metrics` path
--plugins={path} | disabled | The Path to use for loading Plugins
--tunneler.{name}.key={path} | $HOME/.tunneler/key | The File where the Tunneler-Key is stored
--tunneler.{name}.addr={addr} | localhost | The Address of the Tunneler-Server
--tunneler.{name}.port={port} | 8081 | The Port on which to bind the Client on the Tunneler-Server
--tunneler.{name}.public_port={port} | The Port on which to listen for Requests on the Tunneler-Server
--tunneler.{name}.tls={true|false} | disabled | Enables the Tunneler-Entrypoint with TLS enabled
--auto_tls.enable={true|false} | disabled | Enables the Auto-TLS feature
--auto_tls.production={true|false} | disabled | Enables the Production Setting for Lets-Encrypt
--auto_tls.service={name} | () | The Kubernetes-Service to discover other Tunneload instances
--auto_tls.namespace={namespace} | "default" | The Kubernetes Namespace for the Service
--auto_tls.file.path={path} | disabled | The Path from which to load the Cluster-Configuration
--auto_tls.file.dir={dir} | disabled | The Directory where the Certificates should be saved to and loaded from
--auto_tls.cluster.port={port} | 8375 | The Port to use for Cluster communication between instances

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
