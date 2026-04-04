use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};

use crate::config::EnvironmentConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEnvironment {
    pub os: OperatingSystem,
    pub distro: Option<LinuxDistro>,
    pub detected_package_manager: Option<PackageManager>,
    pub effective_package_manager: Option<PackageManager>,
    pub package_manager_source: Option<PackageManagerSource>,
    pub effective_package_manager_available: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingSystem {
    Macos,
    Linux,
    Unknown,
}

impl OperatingSystem {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Macos => "macos",
            Self::Linux => "linux",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxDistro {
    Arch,
    Debian,
    Ubuntu,
    Fedora,
    Unknown,
}

impl LinuxDistro {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Arch => "arch",
            Self::Debian => "debian",
            Self::Ubuntu => "ubuntu",
            Self::Fedora => "fedora",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Brew,
    Apt,
    AptGet,
    Dnf,
    Pacman,
    Paru,
    Yay,
    Unknown,
}

impl PackageManager {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Brew => "brew",
            Self::Apt => "apt",
            Self::AptGet => "apt-get",
            Self::Dnf => "dnf",
            Self::Pacman => "pacman",
            Self::Paru => "paru",
            Self::Yay => "yay",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManagerSource {
    Config,
    Auto,
}

impl PackageManagerSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Config => "config override",
            Self::Auto => "auto-detected",
        }
    }
}

pub fn resolve_environment(config: &EnvironmentConfig) -> Result<ResolvedEnvironment> {
    let path = env::var_os("PATH").map(PathBuf::from);
    resolve_environment_with_inputs(
        config,
        std::env::consts::OS,
        read_os_release().as_deref(),
        path.as_deref(),
    )
}

fn resolve_environment_with_inputs(
    config: &EnvironmentConfig,
    os_name: &str,
    os_release: Option<&str>,
    path_env: Option<&Path>,
) -> Result<ResolvedEnvironment> {
    let detected_os = detect_operating_system(os_name);
    let configured_os = parse_operating_system(&config.os)?;
    let os = configured_os.unwrap_or(detected_os);

    let configured_distro = parse_linux_distro(&config.distro)?;
    let detected_distro = if os == OperatingSystem::Linux {
        Some(detect_linux_distro(os_release))
    } else {
        None
    };

    if os != OperatingSystem::Linux && configured_distro.is_some() {
        bail!("environment.distro can only be set explicitly when os is linux")
    }

    let distro = configured_distro.or(detected_distro);
    let detected_package_manager = detect_package_manager(os, distro, path_env);
    let configured_package_manager = parse_package_manager(&config.preferred_package_manager)?;
    let effective_package_manager = configured_package_manager.or(detected_package_manager);
    let package_manager_source = if configured_package_manager.is_some() {
        effective_package_manager.map(|_| PackageManagerSource::Config)
    } else {
        effective_package_manager.map(|_| PackageManagerSource::Auto)
    };
    let effective_package_manager_available = effective_package_manager
        .map(|package_manager| is_command_available(package_manager.as_str(), path_env))
        .unwrap_or(false);

    Ok(ResolvedEnvironment {
        os,
        distro,
        detected_package_manager,
        effective_package_manager,
        package_manager_source,
        effective_package_manager_available,
    })
}

fn detect_operating_system(os_name: &str) -> OperatingSystem {
    match os_name {
        "macos" => OperatingSystem::Macos,
        "linux" => OperatingSystem::Linux,
        _ => OperatingSystem::Unknown,
    }
}

fn read_os_release() -> Option<String> {
    fs::read_to_string("/etc/os-release").ok()
}

fn detect_linux_distro(os_release: Option<&str>) -> LinuxDistro {
    let Some(os_release) = os_release else {
        return LinuxDistro::Unknown;
    };

    let normalized = os_release.to_ascii_lowercase();

    if normalized.contains("id=arch") || normalized.contains("id_like=arch") {
        LinuxDistro::Arch
    } else if normalized.contains("id=ubuntu") {
        LinuxDistro::Ubuntu
    } else if normalized.contains("id=debian") || normalized.contains("id_like=debian") {
        LinuxDistro::Debian
    } else if normalized.contains("id=fedora") || normalized.contains("id_like=fedora") {
        LinuxDistro::Fedora
    } else {
        LinuxDistro::Unknown
    }
}

fn detect_package_manager(
    os: OperatingSystem,
    distro: Option<LinuxDistro>,
    path_env: Option<&Path>,
) -> Option<PackageManager> {
    let candidates = match os {
        OperatingSystem::Macos => vec![PackageManager::Brew],
        OperatingSystem::Linux => match distro.unwrap_or(LinuxDistro::Unknown) {
            LinuxDistro::Arch => vec![
                PackageManager::Paru,
                PackageManager::Yay,
                PackageManager::Pacman,
            ],
            LinuxDistro::Debian | LinuxDistro::Ubuntu => {
                vec![PackageManager::Apt, PackageManager::AptGet]
            }
            LinuxDistro::Fedora => vec![PackageManager::Dnf],
            LinuxDistro::Unknown => vec![
                PackageManager::Paru,
                PackageManager::Yay,
                PackageManager::Pacman,
                PackageManager::Apt,
                PackageManager::AptGet,
                PackageManager::Dnf,
                PackageManager::Brew,
            ],
        },
        OperatingSystem::Unknown => vec![],
    };

    candidates
        .into_iter()
        .find(|package_manager| is_command_available(package_manager.as_str(), path_env))
}

fn is_command_available(command: &str, path_env: Option<&Path>) -> bool {
    let command = command.trim();

    if command.is_empty() {
        return false;
    }

    let Some(path_env) = path_env else {
        return false;
    };

    env::split_paths(path_env.as_os_str())
        .map(|directory| directory.join(command))
        .any(|candidate| is_executable_file(&candidate))
}

fn is_executable_file(path: &Path) -> bool {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return false,
    };

    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
}

fn parse_operating_system(value: &str) -> Result<Option<OperatingSystem>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "auto" => Ok(None),
        "macos" => Ok(Some(OperatingSystem::Macos)),
        "linux" => Ok(Some(OperatingSystem::Linux)),
        "unknown" => Ok(Some(OperatingSystem::Unknown)),
        other => bail!("unsupported environment.os value `{other}`"),
    }
}

fn parse_linux_distro(value: &str) -> Result<Option<LinuxDistro>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "auto" => Ok(None),
        "arch" => Ok(Some(LinuxDistro::Arch)),
        "debian" => Ok(Some(LinuxDistro::Debian)),
        "ubuntu" => Ok(Some(LinuxDistro::Ubuntu)),
        "fedora" => Ok(Some(LinuxDistro::Fedora)),
        "unknown" => Ok(Some(LinuxDistro::Unknown)),
        other => bail!("unsupported environment.distro value `{other}`"),
    }
}

fn parse_package_manager(value: &str) -> Result<Option<PackageManager>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "auto" => Ok(None),
        "brew" => Ok(Some(PackageManager::Brew)),
        "apt" => Ok(Some(PackageManager::Apt)),
        "apt-get" => Ok(Some(PackageManager::AptGet)),
        "dnf" => Ok(Some(PackageManager::Dnf)),
        "pacman" => Ok(Some(PackageManager::Pacman)),
        "paru" => Ok(Some(PackageManager::Paru)),
        "yay" => Ok(Some(PackageManager::Yay)),
        "unknown" => Ok(Some(PackageManager::Unknown)),
        other => bail!("unsupported environment.preferred_package_manager value `{other}`"),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        EnvironmentConfig, LinuxDistro, OperatingSystem, PackageManager, PackageManagerSource,
        detect_linux_distro, resolve_environment_with_inputs,
    };

    #[test]
    fn detects_arch_paru_over_pacman() {
        let path_env = make_path_with_commands(&["pacman", "paru"]);
        let config = EnvironmentConfig::default();

        let resolved = resolve_environment_with_inputs(
            &config,
            "linux",
            Some("ID=arch\nID_LIKE=arch\n"),
            Some(&path_env),
        )
        .expect("environment should resolve");

        assert_eq!(resolved.os, OperatingSystem::Linux);
        assert_eq!(resolved.distro, Some(LinuxDistro::Arch));
        assert_eq!(
            resolved.detected_package_manager,
            Some(PackageManager::Paru)
        );
        assert_eq!(
            resolved.package_manager_source,
            Some(PackageManagerSource::Auto)
        );

        cleanup_path_with_commands(&path_env, &["pacman", "paru"]);
    }

    #[test]
    fn detects_yay_before_pacman_when_paru_missing() {
        let path_env = make_path_with_commands(&["pacman", "yay"]);
        let config = EnvironmentConfig::default();

        let resolved = resolve_environment_with_inputs(
            &config,
            "linux",
            Some("ID=arch\nID_LIKE=arch\n"),
            Some(&path_env),
        )
        .expect("environment should resolve");

        assert_eq!(resolved.detected_package_manager, Some(PackageManager::Yay));

        cleanup_path_with_commands(&path_env, &["pacman", "yay"]);
    }

    #[test]
    fn prefers_configured_package_manager_override() {
        let path_env = make_path_with_commands(&["pacman", "paru", "yay"]);
        let config = EnvironmentConfig {
            os: "auto".into(),
            distro: "auto".into(),
            preferred_package_manager: "pacman".into(),
        };

        let resolved = resolve_environment_with_inputs(
            &config,
            "linux",
            Some("ID=arch\nID_LIKE=arch\n"),
            Some(&path_env),
        )
        .expect("environment should resolve");

        assert_eq!(
            resolved.detected_package_manager,
            Some(PackageManager::Paru)
        );
        assert_eq!(
            resolved.effective_package_manager,
            Some(PackageManager::Pacman)
        );
        assert_eq!(
            resolved.package_manager_source,
            Some(PackageManagerSource::Config)
        );
        assert!(resolved.effective_package_manager_available);

        cleanup_path_with_commands(&path_env, &["pacman", "paru", "yay"]);
    }

    #[test]
    fn errors_when_linux_distro_is_forced_on_macos() {
        let config = EnvironmentConfig {
            os: "macos".into(),
            distro: "arch".into(),
            preferred_package_manager: "auto".into(),
        };

        let error = resolve_environment_with_inputs(&config, "macos", None, None)
            .expect_err("invalid config should fail");

        assert!(
            error
                .to_string()
                .contains("environment.distro can only be set explicitly when os is linux")
        );
    }

    #[test]
    fn detects_distro_from_os_release() {
        assert_eq!(
            detect_linux_distro(Some("ID=ubuntu\n")),
            LinuxDistro::Ubuntu
        );
        assert_eq!(
            detect_linux_distro(Some("ID=debian\n")),
            LinuxDistro::Debian
        );
        assert_eq!(
            detect_linux_distro(Some("ID=fedora\n")),
            LinuxDistro::Fedora
        );
    }

    fn make_path_with_commands(commands: &[&str]) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let temp_dir = env::temp_dir().join(format!("cli-bot-pm-test-{unique}"));

        fs::create_dir_all(&temp_dir).expect("temp dir should be created");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            for command in commands {
                let path = temp_dir.join(command);
                fs::write(&path, b"#!/usr/bin/env bash\n").expect("command stub should be written");
                let mut permissions = fs::metadata(&path)
                    .expect("metadata should be readable")
                    .permissions();
                permissions.set_mode(0o755);
                fs::set_permissions(&path, permissions).expect("permissions should be set");
            }
        }

        temp_dir
    }

    fn cleanup_path_with_commands(path: &PathBuf, commands: &[&str]) {
        for command in commands {
            let _ = fs::remove_file(path.join(command));
        }

        let _ = fs::remove_dir(path);
    }
}
