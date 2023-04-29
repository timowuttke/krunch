FROM debian:11.3-slim

ARG SKAFFOLD_VERSION=2.3.1
ARG DOCKER_VERSION=23.0.4
ARG HELM_VERSION=3.2.0
ARG KUBECTL_VERSION=1.26.0
ARG K9S_VERSION=0.27.3

# install tools
RUN apt-get update && apt-get -y install ca-certificates curl gnupg wget

# install docker-ce-cli
RUN install -m 0755 -d /etc/apt/keyrings && \
    curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg && \
    chmod a+r /etc/apt/keyrings/docker.gpg && \
    echo "deb [arch="$(dpkg --print-architecture)" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
    "$(. /etc/os-release && echo "$VERSION_CODENAME")" stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null && \
    apt-get update && apt-get -y install docker-ce-cli=5:${DOCKER_VERSION}-1~debian.11~bullseye docker-buildx-plugin

# install helm
RUN wget https://get.helm.sh/helm-v${HELM_VERSION}-linux-amd64.tar.gz && \
    tar -zxvf helm-v${HELM_VERSION}-linux-amd64.tar.gz && \
    mv linux-amd64/helm /usr/local/bin/helm && \
    chmod +x /usr/local/bin/helm

# install kubectl
RUN curl -LO https://dl.k8s.io/release/v${KUBECTL_VERSION}/bin/linux/amd64/kubectl && \
    mv kubectl /usr/local/bin/kubectl && \
    chmod +x /usr/local/bin/kubectl

# install skaffold
RUN curl -Lo skaffold https://storage.googleapis.com/skaffold/releases/v${SKAFFOLD_VERSION}/skaffold-linux-amd64 && \
    mv skaffold /usr/local/bin/skaffold && \
    chmod +x /usr/local/bin/skaffold

# install k9s
RUN curl -Lo k9s.tar.gz https://github.com/derailed/k9s/releases/download/v${K9S_VERSION}/k9s_Linux_amd64.tar.gz && tar -xzf k9s.tar.gz && \
    mv k9s /usr/local/bin/k9s && \
    chmod +x /usr/local/bin/k9s


WORKDIR /krunch

ENTRYPOINT ["tail", "-f", "/dev/null"]