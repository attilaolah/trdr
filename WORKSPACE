workspace(name = "trdr")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    urls = [
        "https://github.com/bazelbuild/rules_rust/releases/download/{version}/rules_rust-v{version}.tar.gz".format(version="0.18.0"),
    ],
    sha256 = "2466e5b2514772e84f9009010797b9cd4b51c1e6445bbd5b5e24848d90e6fb2e",
)

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

rules_rust_dependencies()

rust_register_toolchains()

load("@rules_rust//tools/rust_analyzer:deps.bzl", "rust_analyzer_dependencies")

rust_analyzer_dependencies()

load("@rules_rust//crate_universe:defs.bzl", "crates_repository")

crates_repository(
    name = "crate_index",
    cargo_lockfile = "//:Cargo.lock",
    lockfile = "//:Cargo.bazel.lock",
    manifests = ["//:Cargo.toml"],
)

load("@crate_index//:defs.bzl", "crate_repositories")

crate_repositories()
