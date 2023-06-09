![krunch_logo](https://user-images.githubusercontent.com/47751895/235236895-9b07f0fe-351d-4ef1-8713-0d98888af5ce.svg)

Containerized development setups have many advantages, but are often annoyingly cumbersome to configure. Krunch solves
this problem by offering a one-step solution to create a Minikube-based local development setup. 

It does not matter if you want to transition away from Docker Desktop, have never worked with containers and Kubernetes 
before, or if you are looking for a uniform setup across your team. Krunch will get you started building Docker images
and deploying containers in no time!

## Prerequisites
- [minikube](https://minikube.sigs.k8s.io/docs/start/) is up and running
- nothing else :)

## What it does
![krunch](https://github.com/timowuttke/krunch/assets/47751895/ec4cfd20-50b6-4d47-922e-c987de84570b)

With `krunch install` you perform the following tasks:
1. **Tools Download:** Krunch starts by downloading a collection of frequently used tools. This includes docker-cli, 
docker-buildx, kubectl, helm, [mkcert](https://github.com/FiloSottile/mkcert), 
[skaffold](https://github.com/GoogleContainerTools/skaffold), 
and [k9s](https://github.com/derailed/k9s).
2. **Tools Installation:** The downloaded tools are then placed in `$HOME/.krunch/bin`.
3. **Environment Setup:** Krunch adds the `$HOME/.krunch/bin` directory to your `$PATH` environment variable so that you
can easily execute the downloaded tools.
4. **Docker-CLI Configuration:** The newly downloaded Docker-CLI is pointed towards the Docker Engine running inside 
Minikube.
5. **Ingress Add-on:** Krunch ensures that the Minikube Ingress Add-on is enabled.
6. **Hosts File Update:** Then `k8s.local [minikube ip]` is added to your `etc/hosts` file, so that you can access 
your deployments in Minikube via `http://k8s.local`.
7. **Enable HTTPS:** To enable access over HTTPS as well, Krunch uses [mkcert](https://github.com/FiloSottile/mkcert) 
to create a fake Certificate Authority and a TLS secret within Minikube.

And with `krunch remove`, you revert the above.
