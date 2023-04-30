use anyhow::Result;
use reqwest::Url;

const KUBECTL_VERSION: &str = "1.26.0";
const HELM_VERSION: &str = "3.2.0";
const SKAFFOLD_VERSION: &str = "2.3.1";
const K9S_VERSION: &str = "0.27.3";
const DOCKER_VERSION: &str = "23.0.4";

pub struct DownloadUrls {
    pub docker: Url,
    pub kubectl: Url,
    pub helm: Url,
    pub skaffold: Url,
    pub k9s: Url,
}

impl DownloadUrls {
    pub fn new() -> Result<Self> {
        return if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
            Ok(Self {
                docker: Url::parse(format!(
                    "https://download.docker.com/win/static/stable/x86_64/docker-{}.zip",
                    DOCKER_VERSION).as_str())?,
                kubectl: Url::parse(format!(
                    "https://dl.k8s.io/v{}/bin/windows/amd64/kubectl.exe",
                    KUBECTL_VERSION
                ).as_str())?,
                helm: Url::parse(format!(
                    "https://get.helm.sh/helm-v{}-windows-amd64.zip",
                    HELM_VERSION
                ).as_str())?,
                skaffold: Url::parse(format!(
                    "https://storage.googleapis.com/skaffold/releases/v{}/skaffold-windows-amd64",
                    SKAFFOLD_VERSION
                ).as_str())?,
                k9s: Url::parse(format!(
                    "https://github.com/derailed/k9s/releases/download/v{}/k9s_Windows_amd64.tar.gz",
                    K9S_VERSION
                ).as_str())?,
            })
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            Ok(Self {
                docker: Url::parse(
                    format!(
                        "https://download.docker.com/mac/static/stable/x86_64/docker-{}.tgz",
                        DOCKER_VERSION
                    )
                    .as_str(),
                )?,
                kubectl: Url::parse(
                    format!(
                        "https://dl.k8s.io/v{}/bin/darwin/amd64/kubectl",
                        KUBECTL_VERSION
                    )
                    .as_str(),
                )?,
                helm: Url::parse(
                    format!(
                        "https://get.helm.sh/helm-v{}-darwin-amd64.tar.gz",
                        HELM_VERSION
                    )
                    .as_str(),
                )?,
                skaffold: Url::parse(
                    format!(
                    "https://storage.googleapis.com/skaffold/releases/v{}/skaffold-darwin-amd64",
                    SKAFFOLD_VERSION
                )
                    .as_str(),
                )?,
                k9s: Url::parse(
                    format!(
                    "https://github.com/derailed/k9s/releases/download/v{}/k9s_Darwin_amd64.tar.gz",
                    K9S_VERSION
                )
                    .as_str(),
                )?,
            })
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            Ok(Self {
                docker: Url::parse(
                    format!(
                        "https://download.docker.com/mac/static/stable/aarch64/docker-{}.tgz",
                        DOCKER_VERSION
                    )
                    .as_str(),
                )?,
                kubectl: Url::parse(
                    format!(
                        "https://dl.k8s.io/v{}/bin/darwin/arm64/kubectl",
                        KUBECTL_VERSION
                    )
                    .as_str(),
                )?,
                helm: Url::parse(
                    format!(
                        "https://get.helm.sh/helm-v{}-darwin-arm64.tar.gz",
                        HELM_VERSION
                    )
                    .as_str(),
                )?,
                skaffold: Url::parse(
                    format!(
                    "https://storage.googleapis.com/skaffold/releases/v{}/skaffold-darwin-arm64",
                    SKAFFOLD_VERSION
                )
                    .as_str(),
                )?,
                k9s: Url::parse(
                    format!(
                    "https://github.com/derailed/k9s/releases/download/v{}/k9s_Darwin_arm64.tar.gz",
                    K9S_VERSION
                )
                    .as_str(),
                )?,
            })
        } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
            Ok(Self {
                docker: Url::parse(
                    format!(
                        "https://download.docker.com/linux/static/stable/x86_64/docker-{}.tgz",
                        DOCKER_VERSION
                    )
                        .as_str(),
                )?,
                kubectl: Url::parse(
                    format!(
                        "https://dl.k8s.io/v{}/bin/linux/amd64/kubectl",
                        KUBECTL_VERSION
                    )
                        .as_str(),
                )?,
                helm: Url::parse(
                    format!(
                        "https://get.helm.sh/helm-v{}-linux-amd64.tar.gz",
                        HELM_VERSION
                    )
                        .as_str(),
                )?,
                skaffold: Url::parse(
                    format!(
                        "https://storage.googleapis.com/skaffold/releases/v{}/skaffold-linux-amd64",
                        SKAFFOLD_VERSION
                    )
                        .as_str(),
                )?,
                k9s: Url::parse(
                    format!(
                        "https://github.com/derailed/k9s/releases/download/v{}/k9s_Linux_amd64.tar.gz",
                        K9S_VERSION
                    )
                        .as_str(),
                )?,
            })
        } else {
            panic!("architecture not supported")
        };
    }
}
