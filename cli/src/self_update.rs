//! `aps update --global` — upgrade the machine-wide CLI binary.
//!
//! Project `aps update` stays in `update.rs`. This module fetches the GitHub
//! release asset into `$APS_HOME/bin` (default `~/.aps/bin`), matching the
//! installer's binary-first layout. No extra crates: curl + tar, same as
//! `scaffold/install`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::scaffold::mark_executable;

/// GitHub release target used in asset names (`aps-<target>.tar.gz` / `.zip`).
pub fn release_target(os: &str, arch: &str) -> Option<&'static str> {
    let os = os.to_ascii_lowercase();
    let arch = arch.to_ascii_lowercase();
    match (os.as_str(), arch.as_str()) {
        ("linux", "x86_64") => Some("x86_64-unknown-linux-gnu"),
        ("linux", "aarch64" | "arm64") => Some("aarch64-unknown-linux-gnu"),
        ("macos" | "darwin", "aarch64" | "arm64") => Some("aarch64-apple-darwin"),
        ("macos" | "darwin", "x86_64") => Some("x86_64-apple-darwin"),
        ("windows", "x86_64") => Some("x86_64-pc-windows-gnu"),
        _ => None,
    }
}

/// Download URL for a release asset.
///
/// `version` is `latest`/`main` (floating) or a semver with optional `v` prefix.
pub fn release_asset_url(version: &str, target: &str) -> String {
    let archive = if target.contains("windows") {
        "zip"
    } else {
        "tar.gz"
    };
    let v = version.trim();
    if v.is_empty() || v == "main" || v == "latest" {
        format!(
            "https://github.com/EddaCraft/anvil-plan-spec/releases/latest/download/aps-{target}.{archive}"
        )
    } else {
        let v = v.trim_start_matches('v');
        format!(
            "https://github.com/EddaCraft/anvil-plan-spec/releases/download/v{v}/aps-{target}.{archive}"
        )
    }
}

fn installed_bin_name() -> &'static str {
    if cfg!(windows) { "aps.exe" } else { "aps" }
}

/// `$APS_HOME` if set, otherwise `$HOME/.aps` / `%USERPROFILE%\.aps`.
pub fn resolve_aps_home() -> PathBuf {
    if let Some(home) = env::var_os("APS_HOME") {
        return PathBuf::from(home);
    }
    match env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")) {
        Some(home) => PathBuf::from(home).join(".aps"),
        None => PathBuf::from(".aps"),
    }
}

/// Pin from `APS_VERSION` or `VERSION`, else the latest GitHub release.
pub fn resolve_update_version() -> String {
    env::var("APS_VERSION")
        .or_else(|_| env::var("VERSION"))
        .unwrap_or_else(|_| "latest".to_string())
}

/// Places a downloaded release binary into `dest_dir`.
pub trait BinaryInstaller {
    fn install(&self, url: &str, dest_dir: &Path) -> Result<(), String>;
}

/// Production installer: `curl` the GitHub asset, `tar` it, replace `bin/aps`.
pub struct CurlInstaller;

impl BinaryInstaller for CurlInstaller {
    fn install(&self, url: &str, dest_dir: &Path) -> Result<(), String> {
        let tmp = env::temp_dir().join(format!(
            "aps-self-update-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&tmp).map_err(|err| format!("temp dir: {err}"))?;
        let result = (|| {
            let archive_name = if url.ends_with(".zip") {
                "aps.zip"
            } else {
                "aps.tar.gz"
            };
            let archive = tmp.join(archive_name);
            run(
                "curl",
                &[
                    "-fsSL",
                    url,
                    "-o",
                    archive.to_str().ok_or("archive path is not UTF-8")?,
                ],
            )?;
            let tmp_str = tmp.to_str().ok_or("temp dir path is not UTF-8")?;
            let archive_str = archive.to_str().ok_or("archive path is not UTF-8")?;
            if url.ends_with(".zip") {
                run("tar", &["-xf", archive_str, "-C", tmp_str])?;
            } else {
                run("tar", &["-xzf", archive_str, "-C", tmp_str])?;
            }

            let src_name = if url.contains("windows") {
                "aps.exe"
            } else {
                "aps"
            };
            let src = tmp.join(src_name);
            if !is_regular_file(&src) {
                return Err(format!(
                    "archive did not contain a regular file {src_name} (url: {url})"
                ));
            }

            fs::create_dir_all(dest_dir)
                .map_err(|err| format!("create {}: {err}", dest_dir.display()))?;
            let dest = dest_dir.join(installed_bin_name());
            let staging = dest_dir.join(format!("{}.new", installed_bin_name()));
            fs::copy(&src, &staging).map_err(|err| format!("stage binary: {err}"))?;
            mark_executable(&staging).map_err(|err| format!("chmod: {err}"))?;
            replace_binary(&staging, &dest)?;
            Ok(())
        })();
        let _ = fs::remove_dir_all(&tmp);
        result
    }
}

/// True only for a non-symlink regular file (does not follow links).
fn is_regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|meta| meta.is_file())
}

fn replace_binary(staging: &Path, dest: &Path) -> Result<(), String> {
    if dest.exists() {
        let backup = dest.with_file_name(format!(
            "{}.old",
            dest.file_name().unwrap_or_default().to_string_lossy()
        ));
        let _ = fs::remove_file(&backup);
        if let Err(err) = fs::rename(dest, &backup) {
            let _ = fs::remove_file(staging);
            return Err(format!("could not replace {}: {err}", dest.display()));
        }
        if let Err(err) = fs::rename(staging, dest) {
            let _ = fs::rename(&backup, dest);
            return Err(format!("could not install new binary: {err}"));
        }
        let _ = fs::remove_file(&backup);
        return Ok(());
    }
    fs::rename(staging, dest).map_err(|err| format!("install binary: {err}"))
}

fn run(cmd: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .map_err(|err| format!("failed to run {cmd}: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{cmd} failed ({status})"))
    }
}

/// `aps update --global` using this process's environment.
pub fn cmd_update_global() -> i32 {
    cmd_update_global_in(
        &resolve_aps_home(),
        &CurlInstaller,
        &resolve_update_version(),
    )
}

/// Testable entry: upgrade the CLI under `home` via `installer`.
pub fn cmd_update_global_in(home: &Path, installer: &dyn BinaryInstaller, version: &str) -> i32 {
    let bin = home.join("bin");
    if !bin.is_dir() {
        eprintln!(
            "error: No global APS installation found at {}",
            home.display()
        );
        eprintln!();
        eprintln!("To install globally:");
        eprintln!(
            "  curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install | bash -s -- --cli"
        );
        eprintln!();
        return 1;
    }

    let Some(target) = release_target(env::consts::OS, env::consts::ARCH) else {
        eprintln!(
            "error: No release binary for {}/{}",
            env::consts::OS,
            env::consts::ARCH
        );
        eprintln!(
            "  Re-run the installer without expecting a native binary, or build from source:"
        );
        eprintln!("  cargo install aps-cli");
        return 1;
    };

    let url = release_asset_url(version, target);
    println!("Updating global APS CLI at {}", home.display());
    println!("  {url}");
    match installer.install(&url, &bin) {
        Ok(()) => {
            println!("Global update complete");
            println!("  {}", bin.join(installed_bin_name()).display());
            0
        }
        Err(err) => {
            eprintln!("error: failed to update the global CLI: {err}");
            eprintln!(
                "  Re-run the installer: curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install | bash -s -- --cli"
            );
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn scratch(tag: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("aps-self-update-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    struct FakeInstaller {
        payload: &'static [u8],
        fail: bool,
        url: Mutex<Option<String>>,
    }

    impl FakeInstaller {
        fn ok(payload: &'static [u8]) -> Self {
            Self {
                payload,
                fail: false,
                url: Mutex::new(None),
            }
        }

        fn failing() -> Self {
            Self {
                payload: b"",
                fail: true,
                url: Mutex::new(None),
            }
        }
    }

    impl BinaryInstaller for FakeInstaller {
        fn install(&self, url: &str, dest_dir: &Path) -> Result<(), String> {
            *self.url.lock().unwrap() = Some(url.to_string());
            if self.fail {
                return Err("download failed".into());
            }
            fs::create_dir_all(dest_dir).unwrap();
            fs::write(dest_dir.join(installed_bin_name()), self.payload).unwrap();
            Ok(())
        }
    }

    #[test]
    fn regular_file_check_rejects_symlinks() {
        let dir = scratch("symlink");
        fs::create_dir_all(&dir).unwrap();
        let target = dir.join("target");
        let link = dir.join("link");
        fs::write(&target, b"payload").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target, &link).unwrap();
            assert!(is_regular_file(&target));
            assert!(!is_regular_file(&link));
        }
        #[cfg(not(unix))]
        {
            assert!(is_regular_file(&target));
        }
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn maps_unix_and_windows_targets() {
        assert_eq!(
            release_target("linux", "x86_64"),
            Some("x86_64-unknown-linux-gnu")
        );
        assert_eq!(
            release_target("Linux", "aarch64"),
            Some("aarch64-unknown-linux-gnu")
        );
        assert_eq!(
            release_target("darwin", "arm64"),
            Some("aarch64-apple-darwin")
        );
        assert_eq!(
            release_target("macos", "x86_64"),
            Some("x86_64-apple-darwin")
        );
        assert_eq!(
            release_target("windows", "x86_64"),
            Some("x86_64-pc-windows-gnu")
        );
        assert_eq!(release_target("linux", "riscv64"), None);
    }

    #[test]
    fn builds_latest_and_pinned_asset_urls() {
        assert_eq!(
            release_asset_url("latest", "x86_64-unknown-linux-gnu"),
            "https://github.com/EddaCraft/anvil-plan-spec/releases/latest/download/aps-x86_64-unknown-linux-gnu.tar.gz"
        );
        assert_eq!(
            release_asset_url("main", "aarch64-apple-darwin"),
            "https://github.com/EddaCraft/anvil-plan-spec/releases/latest/download/aps-aarch64-apple-darwin.tar.gz"
        );
        assert_eq!(
            release_asset_url("0.8.1", "x86_64-unknown-linux-gnu"),
            "https://github.com/EddaCraft/anvil-plan-spec/releases/download/v0.8.1/aps-x86_64-unknown-linux-gnu.tar.gz"
        );
        assert_eq!(
            release_asset_url("v0.8.1", "x86_64-pc-windows-gnu"),
            "https://github.com/EddaCraft/anvil-plan-spec/releases/download/v0.8.1/aps-x86_64-pc-windows-gnu.zip"
        );
    }

    #[test]
    fn errors_when_global_bin_dir_is_missing() {
        let home = scratch("noglobal");
        fs::create_dir_all(&home).unwrap();
        assert_eq!(
            cmd_update_global_in(&home, &FakeInstaller::ok(b"x"), "latest"),
            1
        );
        fs::remove_dir_all(&home).ok();
    }

    #[test]
    fn installs_into_existing_global_bin() {
        let home = scratch("ok");
        let bin = home.join("bin");
        fs::create_dir_all(&bin).unwrap();
        fs::write(bin.join(installed_bin_name()), b"old").unwrap();
        let installer = FakeInstaller::ok(b"new-binary");
        assert_eq!(cmd_update_global_in(&home, &installer, "0.8.1"), 0);
        assert_eq!(
            fs::read(bin.join(installed_bin_name())).unwrap(),
            b"new-binary"
        );
        let url = installer.url.lock().unwrap().clone().unwrap();
        assert!(url.contains("/releases/download/v0.8.1/"));
        fs::remove_dir_all(&home).ok();
    }

    #[test]
    fn failed_install_leaves_existing_binary() {
        let home = scratch("fail");
        let bin = home.join("bin");
        fs::create_dir_all(&bin).unwrap();
        fs::write(bin.join(installed_bin_name()), b"old").unwrap();
        assert_eq!(
            cmd_update_global_in(&home, &FakeInstaller::failing(), "latest"),
            1
        );
        assert_eq!(fs::read(bin.join(installed_bin_name())).unwrap(), b"old");
        fs::remove_dir_all(&home).ok();
    }
}
