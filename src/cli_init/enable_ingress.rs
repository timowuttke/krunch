use anyhow::Result;
use std::process::Command;

use crate::shared::commands::{get_binary_path, handle_output, Binary};
use serde_json::Value;

pub fn enable_ingress_addon_if_needed() -> Result<()> {
    let status: Value = get_minikbe_addons()?;

    if status["ingress"]["Status"] == "enabled" {
        println!("already done")
    } else {
        enable_minikube_ingress_addon()?;
        println!("success")
    }

    Ok(())
}

fn enable_minikube_ingress_addon() -> Result<()> {
    let output = Command::new(get_binary_path(Binary::Minikube)?)
        .arg("addons")
        .arg("enable")
        .arg("ingress")
        .output()
        .expect("failed to execute process");

    handle_output(output)?;

    Ok(())
}

fn get_minikbe_addons() -> Result<Value> {
    let output = Command::new(get_binary_path(Binary::Minikube)?)
        .arg("addons")
        .arg("list")
        .arg("--output")
        .arg("json")
        .output()
        .expect("failed to execute process");

    let value: Value = serde_json::from_str(&*handle_output(output)?)?;

    Ok(value)
}
