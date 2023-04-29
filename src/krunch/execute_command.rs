use crate::krunch::NAMESPACE;
use crate::Krunch;
use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, AttachParams};
use tokio::process::Command;

impl Krunch {
    pub async fn execute_pod_command(
        &self,
        command: String,
        print_to_stdout: bool,
        print_to_stderr: bool,
    ) -> Result<(String, String)> {
        let mut sh_command = vec!["sh".to_string(), "-c".to_string()];
        sh_command.push(command);

        let pod_name: String = match self.get_krunch_pod_name().await {
            None => return Err(anyhow!("krunch pod not found, consider \"krunch init\"")),
            Some(inner) => inner,
        };

        let ap = AttachParams::default();
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), NAMESPACE);
        let mut attached = pods.exec(&pod_name, sh_command, &ap).await?;

        let mut stdout_reader = attached.stdout().unwrap();
        let mut stdout = tokio::io::stdout();
        let mut stdout_buffer = Vec::new();

        let mut stderr_reader = attached.stderr().unwrap();
        let mut stderr = tokio::io::stderr();
        let mut stderr_buffer = Vec::new();

        if print_to_stdout {
            tokio::spawn(async move {
                tokio::io::copy(&mut stdout_reader, &mut stdout)
                    .await
                    .unwrap();
            });
        } else {
            tokio::io::copy(&mut stdout_reader, &mut stdout_buffer)
                .await
                .unwrap();
        };

        if print_to_stderr {
            tokio::spawn(async move {
                tokio::io::copy(&mut stderr_reader, &mut stderr)
                    .await
                    .unwrap();
            });
        } else {
            tokio::io::copy(&mut stderr_reader, &mut stderr_buffer)
                .await
                .unwrap();
        };

        attached.take_status().unwrap().await.unwrap();

        Ok((
            String::from_utf8(stdout_buffer)?,
            String::from_utf8(stderr_buffer)?,
        ))
    }

    pub async fn execute_host_command(command: &str) -> Result<(String, String, i32)> {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .arg("-/C")
                .arg(command)
                .output()
                .await
                .expect("failed to execute process")
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .await
                .expect("failed to execute process")
        };

        Ok((
            String::from_utf8(output.stdout)?,
            String::from_utf8(output.stderr)?,
            output.status.code().unwrap(),
        ))
    }
}
