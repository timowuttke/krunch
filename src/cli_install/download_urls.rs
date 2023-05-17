use crate::cli_install::get_versions::{get_actual_versions, get_expected_versions};
use anyhow::{anyhow, Result};
use reqwest::Url;
use std::fmt;

pub struct Download {
    pub target: String,
    pub source: Url,
}

impl fmt::Debug for Download {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.target)
    }
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

pub fn get_necessary_downloads() -> Result<Vec<Download>> {
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

    let mut necessary_downloads = vec![];

    let expected_versions = get_expected_versions()?;
    let actual_versions = get_actual_versions()?;

    if expected_versions.docker != actual_versions.docker {
        let docker_version = expected_versions
            .docker
            .ok_or(anyhow!("failed to read config"))?;
        let docker = Download {
            target: format!("docker{}", ext_str),
            source: get_docker_url(&os, &arch, docker_version),
        };
        necessary_downloads.push(docker);
    }

    if expected_versions.buildx != actual_versions.buildx {
        let buildx_version = expected_versions
            .buildx
            .ok_or(anyhow!("failed to read config"))?;
        let buildx = Download {
            target: format!("docker-buildx{}", ext_str),
            source: get_buildx_url(&os, &arch, buildx_version),
        };
        necessary_downloads.push(buildx);
    }

    if expected_versions.kubectl != actual_versions.kubectl {
        let kubectl_version = expected_versions
            .kubectl
            .ok_or(anyhow!("failed to read config"))?;
        let kubectl = Download {
            target: format!("kubectl{}", ext_str),
            source: get_kubectl_url(&os, &arch, kubectl_version),
        };
        necessary_downloads.push(kubectl);
    }

    if expected_versions.helm != actual_versions.helm {
        let helm_version = expected_versions
            .helm
            .ok_or(anyhow!("failed to read config"))?;
        let helm = Download {
            target: format!("helm{}", ext_str),
            source: get_helm_url(&os, &arch, helm_version),
        };
        necessary_downloads.push(helm);
    }

    if expected_versions.mkcert != actual_versions.mkcert {
        let mkcert_version = expected_versions
            .mkcert
            .ok_or(anyhow!("failed to read config"))?;
        let mkcert = Download {
            target: format!("mkcert{}", ext_str),
            source: get_mkcert_url(&os, &arch, mkcert_version),
        };
        necessary_downloads.push(mkcert);
    }

    if expected_versions.skaffold != actual_versions.skaffold {
        let skaffold_version = expected_versions
            .skaffold
            .ok_or(anyhow!("failed to read config"))?;
        let skaffold = Download {
            target: format!("skaffold{}", ext_str),
            source: get_skaffold_url(&os, &arch, skaffold_version),
        };
        necessary_downloads.push(skaffold);
    }

    if expected_versions.k9s != actual_versions.k9s {
        let k9s_version = expected_versions
            .k9s
            .ok_or(anyhow!("failed to read config"))?;
        let k9s = Download {
            target: format!("k9s{}", ext_str),
            source: get_k9s_url(&os, &arch, k9s_version),
        };
        necessary_downloads.push(k9s);
    }

    Ok(necessary_downloads)
}

fn get_docker_url(os: &TargetOs, arch: &TargetArch, version: String) -> Url {
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

fn get_kubectl_url(os: &TargetOs, arch: &TargetArch, version: String) -> Url {
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

fn get_helm_url(os: &TargetOs, arch: &TargetArch, version: String) -> Url {
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

fn get_mkcert_url(os: &TargetOs, arch: &TargetArch, version: String) -> Url {
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

fn get_skaffold_url(os: &TargetOs, arch: &TargetArch, version: String) -> Url {
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

fn get_k9s_url(os: &TargetOs, arch: &TargetArch, version: String) -> Url {
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

fn get_buildx_url(os: &TargetOs, arch: &TargetArch, version: String) -> Url {
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
