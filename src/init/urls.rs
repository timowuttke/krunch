use reqwest::Url;

const KUBECTL_VERSION: &str = "1.26.0";
const HELM_VERSION: &str = "3.2.0";
const MKCERT_VERSION: &str = "1.4.4";
const SKAFFOLD_VERSION: &str = "2.3.1";
const K9S_VERSION: &str = "0.27.3";
const DOCKER_VERSION: &str = "23.0.4";
const BUILDX_VERSION: &str = "0.10.4";

pub struct Download {
    pub target: String,
    pub source: Url,
}

enum TargetOs {
    Windows,
    MacOs,
    Linux,
}

enum TargetArch {
    Amd64,
    Arm64,
}

pub fn get_downloads() -> Vec<Download> {
    let os;
    let arch;
    let ext_str;

    if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        os = TargetOs::Windows;
        arch = TargetArch::Amd64;
        ext_str = ".exe"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        os = TargetOs::MacOs;
        arch = TargetArch::Amd64;
        ext_str = ""
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        os = TargetOs::MacOs;
        arch = TargetArch::Arm64;
        ext_str = ""
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        os = TargetOs::Linux;
        arch = TargetArch::Amd64;
        ext_str = ""
    } else {
        panic!("architecture not supported")
    };

    let docker = Download {
        target: format!("docker{}", ext_str),
        source: get_docker_url(&os, &arch, DOCKER_VERSION),
    };

    let buildx = Download {
        target: format!("docker-buildx{}", ext_str),
        source: get_buildx_url(&os, &arch, BUILDX_VERSION),
    };

    let kubectl = Download {
        target: format!("kubectl{}", ext_str),
        source: get_kubectl_url(&os, &arch, KUBECTL_VERSION),
    };

    let helm = Download {
        target: format!("helm{}", ext_str),
        source: get_helm_url(&os, &arch, HELM_VERSION),
    };

    let mkcert = Download {
        target: format!("mkcert{}", ext_str),
        source: get_mkcert_url(&os, &arch, MKCERT_VERSION),
    };

    let skaffold = Download {
        target: format!("skaffold{}", ext_str),
        source: get_skaffold_url(&os, &arch, SKAFFOLD_VERSION),
    };

    let k9s = Download {
        target: format!("k9s{}", ext_str),
        source: get_k9s_url(&os, &arch, K9S_VERSION),
    };

    vec![docker, buildx, kubectl, helm, mkcert, skaffold, k9s]
}

fn get_docker_url(os: &TargetOs, arch: &TargetArch, version: &str) -> Url {
    let os_str = match os {
        TargetOs::Windows => "win",
        TargetOs::MacOs => "mac",
        TargetOs::Linux => "linux",
    };

    let ext = match os {
        TargetOs::Windows => ".zip",
        TargetOs::MacOs => ".tgz",
        TargetOs::Linux => ".tgz",
    };

    let arch_str = match arch {
        TargetArch::Amd64 => "x86_64",
        TargetArch::Arm64 => "aarch64",
    };

    return Url::parse(&*format!(
        "https://download.docker.com/{}/static/stable/{}/docker-{}{}",
        os_str, arch_str, version, ext
    ))
    .expect("failed to parse URL");
}

fn get_kubectl_url(os: &TargetOs, arch: &TargetArch, version: &str) -> Url {
    let os_str = match os {
        TargetOs::Windows => "windows",
        TargetOs::MacOs => "darwin",
        TargetOs::Linux => "linux",
    };

    let ext = match os {
        TargetOs::Windows => ".exe",
        TargetOs::MacOs => "",
        TargetOs::Linux => "",
    };

    let arch_str = match arch {
        TargetArch::Amd64 => "amd64",
        TargetArch::Arm64 => "arm64",
    };

    return Url::parse(&*format!(
        "https://dl.k8s.io/v{}/bin/{}/{}/kubectl{}",
        version, os_str, arch_str, ext
    ))
    .expect("failed to parse URL");
}

fn get_helm_url(os: &TargetOs, arch: &TargetArch, version: &str) -> Url {
    let os_str = match os {
        TargetOs::Windows => "windows",
        TargetOs::MacOs => "darwin",
        TargetOs::Linux => "linux",
    };

    let ext = match os {
        TargetOs::Windows => ".zip",
        TargetOs::MacOs => ".tar.gz",
        TargetOs::Linux => ".tar.gz",
    };

    let arch_str = match arch {
        TargetArch::Amd64 => "amd64",
        TargetArch::Arm64 => "arm64",
    };

    return Url::parse(&*format!(
        "https://get.helm.sh/helm-v{}-{}-{}{}",
        version, os_str, arch_str, ext
    ))
    .expect("failed to parse URL");
}

fn get_mkcert_url(os: &TargetOs, arch: &TargetArch, version: &str) -> Url {
    let os_str = match os {
        TargetOs::Windows => "windows",
        TargetOs::MacOs => "darwin",
        TargetOs::Linux => "linux",
    };

    let arch_str = match arch {
        TargetArch::Amd64 => "amd64",
        TargetArch::Arm64 => "arm64",
    };

    return Url::parse(&*format!(
        "https://dl.filippo.io/mkcert/v{}?for={}/{}",
        version, os_str, arch_str
    ))
    .expect("failed to parse URL");
}

fn get_skaffold_url(os: &TargetOs, arch: &TargetArch, version: &str) -> Url {
    let os_str = match os {
        TargetOs::Windows => "windows",
        TargetOs::MacOs => "darwin",
        TargetOs::Linux => "linux",
    };

    let ext = match os {
        TargetOs::Windows => ".exe",
        TargetOs::MacOs => "",
        TargetOs::Linux => "",
    };

    let arch_str = match arch {
        TargetArch::Amd64 => "amd64",
        TargetArch::Arm64 => "arm64",
    };

    return Url::parse(&*format!(
        "https://storage.googleapis.com/skaffold/releases/v{}/skaffold-{}-{}{}",
        version, os_str, arch_str, ext
    ))
    .expect("failed to parse URL");
}

fn get_k9s_url(os: &TargetOs, arch: &TargetArch, version: &str) -> Url {
    let os_str = match os {
        TargetOs::Windows => "Windows",
        TargetOs::MacOs => "Darwin",
        TargetOs::Linux => "Linux",
    };

    let ext = match os {
        TargetOs::Windows => ".tar.gz",
        TargetOs::MacOs => ".tar.gz",
        TargetOs::Linux => ".tar.gz",
    };

    let arch_str = match arch {
        TargetArch::Amd64 => "amd64",
        TargetArch::Arm64 => "arm64",
    };

    return Url::parse(&*format!(
        "https://github.com/derailed/k9s/releases/download/v{}/k9s_{}_{}{}",
        version, os_str, arch_str, ext
    ))
    .expect("failed to parse URL");
}

fn get_buildx_url(os: &TargetOs, arch: &TargetArch, version: &str) -> Url {
    let os_str = match os {
        TargetOs::Windows => "windows",
        TargetOs::MacOs => "darwin",
        TargetOs::Linux => "linux",
    };

    let ext = match os {
        TargetOs::Windows => ".exe",
        TargetOs::MacOs => "",
        TargetOs::Linux => "",
    };

    let arch_str = match arch {
        TargetArch::Amd64 => "amd64",
        TargetArch::Arm64 => "arm64",
    };

    return Url::parse(&*format!(
        "https://github.com/docker/buildx/releases/download/v{}/buildx-v{}.{}-{}{}",
        version, version, os_str, arch_str, ext
    ))
    .expect("failed to parse URL");
}
