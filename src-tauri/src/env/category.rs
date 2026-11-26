//! Değişken kategorileri (yalnızca gruplama ve filtreleme için).

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Category {
    Path,
    System,
    User,
    Development,
    Credentials,
    Other,
}

const PATH_NAMES: &[&str] = &[
    "PATH",
    "PATHEXT",
    "PSMODULEPATH",
    "CLASSPATH",
    "PYTHONPATH",
    "NODE_PATH",
    "GOPATH",
];

const SYSTEM_NAMES: &[&str] = &[
    "OS",
    "COMSPEC",
    "SYSTEMROOT",
    "WINDIR",
    "SYSTEMDRIVE",
    "PROGRAMFILES",
    "PROGRAMFILES(X86)",
    "PROGRAMW6432",
    "PROGRAMDATA",
    "COMMONPROGRAMFILES",
    "COMMONPROGRAMFILES(X86)",
    "COMMONPROGRAMW6432",
    "ALLUSERSPROFILE",
    "PUBLIC",
    "NUMBER_OF_PROCESSORS",
    "DRIVERDATA",
    "COMPUTERNAME",
    "LOGONSERVER",
    "SESSIONNAME",
];

const SYSTEM_PREFIXES: &[&str] = &["PROCESSOR_"];

const USER_NAMES: &[&str] = &[
    "USERNAME",
    "USERPROFILE",
    "USERDOMAIN",
    "USERDOMAIN_ROAMINGPROFILE",
    "HOMEDRIVE",
    "HOMEPATH",
    "APPDATA",
    "LOCALAPPDATA",
    "TEMP",
    "TMP",
    "ONEDRIVE",
    "ONEDRIVECONSUMER",
    "ONEDRIVECOMMERCIAL",
];

const DEV_NAMES: &[&str] = &[
    "JAVA_HOME",
    "JDK_HOME",
    "JRE_HOME",
    "GOROOT",
    "GOBIN",
    "CARGO_HOME",
    "RUSTUP_HOME",
    "VIRTUAL_ENV",
    "PIPX_HOME",
    "M2_HOME",
    "MAVEN_HOME",
    "GRADLE_HOME",
    "KUBECONFIG",
    "PNPM_HOME",
    "NVM_HOME",
    "NVM_SYMLINK",
    "VCPKG_ROOT",
    "ANDROID_HOME",
    "ANDROID_SDK_ROOT",
    "FLUTTER_ROOT",
    "DENO_INSTALL",
    "BUN_INSTALL",
    "EDITOR",
    "VISUAL",
    "COMPOSER_HOME",
    "GEM_HOME",
    "GEM_PATH",
    "RUBYOPT",
    "NODE_ENV",
    "NODE_OPTIONS",
    "UV_CACHE_DIR",
];

const DEV_PREFIXES: &[&str] = &[
    "PYTHON", "CONDA", "NODE_", "NPM_", "YARN_", "DOTNET_", "NUGET_", "GIT_", "DOCKER_",
    "COMPOSE_", "GO", "RUST", "CARGO_", "JAVA_", "GRADLE_", "MAVEN_", "VS", "MSBUILD", "PHP",
    "UV_", "PIP_", "WSL",
];

pub fn categorize(name: &str, sensitive: bool) -> Category {
    let n = name.to_uppercase();
    if PATH_NAMES.contains(&n.as_str()) {
        return Category::Path;
    }
    if sensitive {
        return Category::Credentials;
    }
    if SYSTEM_NAMES.contains(&n.as_str()) || SYSTEM_PREFIXES.iter().any(|p| n.starts_with(p)) {
        return Category::System;
    }
    if USER_NAMES.contains(&n.as_str()) {
        return Category::User;
    }
    if DEV_NAMES.contains(&n.as_str()) || DEV_PREFIXES.iter().any(|p| n.starts_with(p)) {
        return Category::Development;
    }
    Category::Other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories() {
        assert_eq!(categorize("Path", false), Category::Path);
        assert_eq!(
            categorize("PROCESSOR_ARCHITECTURE", false),
            Category::System
        );
        assert_eq!(categorize("TEMP", false), Category::User);
        assert_eq!(categorize("JAVA_HOME", false), Category::Development);
        assert_eq!(categorize("PYTHONHOME", false), Category::Development);
        assert_eq!(categorize("GITHUB_TOKEN", true), Category::Credentials);
        assert_eq!(categorize("FOO", false), Category::Other);
    }
}
