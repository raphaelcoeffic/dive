pub mod base_image;
pub mod fork;
pub mod image_builder;
pub mod namespaces;
pub mod nixos;
pub mod nixpkgs;
pub mod overlay;
pub mod pid_lookup;
pub mod shared_mount;
pub mod shell;

#[cfg(feature = "embedded_image")]
mod embedded_image;

mod pid_file;

pub(crate) const NIX_VERSION: &str = "2.34.9";
pub(crate) const NIXOS_VERSION: &str = "26.05";

pub(crate) const CACHE_HOME: &str = "/nix/.cache";
pub(crate) const CONFIG_HOME: &str = "/nix/.config";

pub(crate) const USER_ENV_DIR: &str = "/nix/.env";
pub(crate) const BASE_DIR: &str = "/nix/.base";
pub(crate) const BASE_PACKAGES_FILE: &str = "/nix/.base.packages";
pub(crate) const ETC_DIR: &str = "/nix/etc";

pub(crate) const SSL_CERTS: &str = "/nix/.base/etc/ssl/certs/ca-bundle.crt";

/// CA roots distrusted in the base image's `cacert` (NSS labels).
///
/// NSS 3.131.1 removed these after the eMudhra intermediate CA compromise
/// (October 2026), but nixos-26.05 is still at NSS 3.126.
pub(crate) const CACERT_BLOCKLIST: &[&str] = &[
    "emSign ECC Root CA - G3",
    "emSign Root CA - C1",
    "emSign ECC Root CA - C3",
];

pub const BASE_PACKAGES: &[&str] = &[
    "bash",
    "cacert",
    "coreutils",
    "curl",
    "diffutils",
    "dig",
    "findutils",
    "gnugrep",
    "gnused",
    "gnutar",
    "gzip",
    "helix",
    "htop",
    "iproute2",
    "iputils",
    "jq",
    "kitty.terminfo",
    "less",
    "lsof",
    "man",
    "nano",
    "netcat-openbsd",
    "procps",
    "sngrep",
    "sqlite",
    "strace",
    "tcpdump",
    "util-linux",
    "vim",
    "xz",
    "zsh",
    "zsh-prezto",
    "zsh-autosuggestions",
    "zsh-completions",
    "zsh-fast-syntax-highlighting",
];
