use crate::Krunch;
use anyhow::{anyhow, Result};

impl Krunch {
    pub async fn bomb(&self, command: String) -> Result<()> {
        let ns = match Krunch::get_namespace(command.as_str()) {
            None => "default",
            Some(inner) => inner,
        };

        if ns.starts_with("kube-") || ns.starts_with("ingress-") || ns.starts_with("krunch") {
            return Err(anyhow!("cleaning namespace {} is not allowed", ns));
        };

        let helm_deployments = self
            .execute_pod_command(
                format!("helm ls -n {} --all --short", ns).to_string(),
                false,
                false,
            )
            .await?
            .0;

        if !helm_deployments.is_empty() {
            self.execute_pod_command(
                format!(
                    "helm ls -n {} --all --short | xargs -L1 helm -n {} delete",
                    ns, ns
                )
                .to_string(),
                true,
                true,
            )
            .await?;
        };

        let plain_k8s_resources = self.execute_pod_command(
            format!("kubectl get all -n {} | grep -v -e \"service/kubernetes\" -e \"NAME\" -e \"^[[:blank:]]*$\" | awk '{{print $1}}'", ns).to_string(),
            false, false
        ).await?.0;

        if ns == "default" {
            self.execute_pod_command(
                format!(
                    "echo '{}' | xargs kubectl delete -n {}",
                    plain_k8s_resources, ns
                )
                .to_string(),
                true,
                false,
            )
            .await?;
        } else {
            self.execute_pod_command(
                format!("kubectl get all -n {} | grep -v -e \"NAME\" -e \"^[[:blank:]]*$\" | awk '{{print $1}}' {} | xargs kubectl delete -n {}", ns, ns, ns).to_string(),
                true, false
            )
                .await?;
        }

        Ok(())
    }

    fn get_namespace(s: &str) -> Option<&str> {
        let mut iter = s.split_whitespace();
        while let Some(sub) = iter.next() {
            if sub == "-n" || sub == "--namespace" {
                return iter.next();
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_namespace() {
        let s = "-n hello world";
        assert_eq!(Krunch::get_namespace(s), Some("hello"));

        let s = "--namespace hello world";
        assert_eq!(Krunch::get_namespace(s), Some("hello"));

        let s = "hello world -n";
        assert_eq!(Krunch::get_namespace(s), None);
    }
}
