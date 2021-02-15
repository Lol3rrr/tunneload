# Tunneload
A simple and flexible Load-Balancer that can easily integrate with other Tunneler-Software

## CLI Options
Key | Default | Description
--- | --- | ---
--kube-conf | disabled | Enables the Kubernetes-Configurator
--file-conf {path} | disabled | Enables the File-Configurator for the given file/directory

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
