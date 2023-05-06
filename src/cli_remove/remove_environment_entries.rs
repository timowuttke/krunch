use crate::shared::windows_registry::delete_from_environment;
use anyhow::Result;

pub fn remove_environment_entries() -> Result<()> {
    if cfg!(target_family = "unix") {
        remove_environment_entries_unix()?;
    } else if cfg!(target_family = "windows") {
        remove_environment_entries_windows()?;
    }

    Ok(())
}

fn remove_environment_entries_unix() -> Result<()> {
    println!("success");

    Ok(())
}

fn remove_environment_entries_windows() -> Result<()> {
    let delete_result: Result<_> = (|| {
        delete_from_environment("DOCKER_TLS_VERIFY")?;
        delete_from_environment("DOCKER_HOST")?;
        delete_from_environment("DOCKER_CERT_PATH")?;
        delete_from_environment("MINIKUBE_ACTIVE_DOCKERD")?;

        Ok(())
    })();

    if delete_result.is_err() {
        println!("already done");
    } else {
        println!("success");
    }

    Ok(())
}
