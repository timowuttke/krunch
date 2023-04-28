use crate::Krunch;
use anyhow::Result;

impl Krunch {
    pub async fn bomb(&self, command: String) -> Result<()> {
        let ns = match Krunch::get_namespace(command.as_str()) {
            None => "default",
            Some(inner) => inner,
        };

        let helm_deployments = self
            .execute_pod_command(
                format!("helm ls -n {} --all --short", ns).to_string(),
                false,
            )
            .await?;

        if !helm_deployments.0.is_empty() {
            self.execute_pod_command(
                format!(
                    "helm ls -n {} --all --short | xargs -L1 helm -n {} delete",
                    ns, ns
                )
                .to_string(),
                true,
            )
            .await?;
        };

        self.execute_pod_command(
            format!("kubectl delete all --all -n {}", ns).to_string(),
            true,
        )
        .await?;

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
