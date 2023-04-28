use crate::krunch::NAMESPACE;
use crate::Krunch;
use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, AttachParams};

impl Krunch {
    pub async fn execute_pod_command(&self, command: String) -> Result<()> {
        let mut sh_command = vec!["sh".to_string(), "-c".to_string()];
        sh_command.push(command);

        let pod_name: String = match self.get_krunch_pod_name().await {
            None => return Err(anyhow!("Pod not found")),
            Some(inner) => inner,
        };

        let ap = AttachParams::default();
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), NAMESPACE);
        let mut attached = pods.exec(&pod_name, sh_command, &ap).await?;

        let mut stdout_reader = attached.stdout().unwrap();
        let mut stdout = tokio::io::stdout();

        let mut stderr_reader = attached.stderr().unwrap();
        let mut stderr = tokio::io::stderr();

        tokio::spawn(async move {
            tokio::io::copy(&mut stdout_reader, &mut stdout)
                .await
                .unwrap();
        });

        tokio::spawn(async move {
            tokio::io::copy(&mut stderr_reader, &mut stderr)
                .await
                .unwrap();
        });

        attached.take_status().unwrap().await.unwrap();

        Ok(())
    }
}
