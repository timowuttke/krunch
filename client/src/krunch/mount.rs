use crate::Krunch;
use anyhow::Result;
use std::env;
use std::process::Stdio;

use tokio::process::Command;

impl Krunch {
    pub async fn mount_current_path(&mut self) -> Result<()> {
        let current_dir_path_buff = env::current_dir()?;
        let current_dir_str = current_dir_path_buff.as_path().to_str().unwrap();

        let mount = Command::new("minikube")
            .arg("mount")
            .arg(format!("{}:/krunch", current_dir_str).as_str())
            .stdout(Stdio::null())
            .spawn()?;

        self.mount = Some(mount);

        let pod_name = self.get_krunch_pod_name().await.unwrap();
        self.execute_pod_command(format!("kubectl delete po {}", pod_name))
            .await?;

        self.wait_for_pod_to_be_healthy().await?;

        Ok(())
    }

    pub async fn unmount(&mut self) -> Result<()> {
        if let Some(mut inner) = self.mount.take() {
            inner.kill().await?;

            Krunch::execute_host_command(
                "minikube ssh \"sudo umount /krunch && sudo rm -rf /krunch\"",
            )
            .await?;

            self.mount = None;
        }

        Ok(())
    }
}
