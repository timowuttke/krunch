use crate::shared::handle_output;
use anyhow::{anyhow, Result};
use std::process::Command;

pub fn write_to_environment(key: &str, value: String) -> Result<()> {
    let output = Command::new("reg")
        .arg("add")
        .arg("HKEY_CURRENT_USER\\Environment")
        .arg("/v")
        .arg(key)
        .arg("/t")
        .arg("REG_SZ")
        .arg("/d")
        .arg(value)
        .arg("/f")
        .output()
        .expect("failed to execute process");

    handle_output(output)?;

    let output = Command::new("SETX")
        .arg("USERNAME")
        .arg("%USERNAME%")
        .output()
        .expect("failed to execute process");

    handle_output(output)?;

    Ok(())
}

pub fn read_from_environment(key: &str) -> Result<String> {
    let output = Command::new("reg")
        .arg("query")
        .arg("HKEY_CURRENT_USER\\Environment")
        .arg("/v")
        .arg(key)
        .output()
        .expect("failed to execute process");

    let tmp = handle_output(output)?;

    if let Some(line) = tmp.lines().last() {
        let words: Vec<&str> = line.split_whitespace().collect();
        if let Some(value) = words.last() {
            return Ok(value.to_string());
        }
    }

    Err(anyhow!(
        "failed to read key {} from the windows registry",
        key
    ))
}

pub fn delete_from_environment(key: &str) -> Result<()> {
    let output = Command::new("reg")
        .arg("delete")
        .arg("HKEY_CURRENT_USER\\Environment")
        .arg("/v")
        .arg(key)
        .arg("/f")
        .output()
        .expect("failed to execute process");

    handle_output(output)?;

    Ok(())
}
