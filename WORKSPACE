workspace(name = "trdr")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

## Rust:

http_archive(
    name = "rules_rust",
    sha256 = "2466e5b2514772e84f9009010797b9cd4b51c1e6445bbd5b5e24848d90e6fb2e",
    urls = [
        "https://github.com/bazelbuild/rules_rust/releases/download/{version}/rules_rust-v{version}.tar.gz".format(version = "0.18.0"),
    ],
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

## Go:

http_archive(
    name = "io_bazel_rules_go",
    build_file_content = None,
    sha256 = "dd926a88a564a9246713a9c00b35315f54cbd46b31a26d5d8fb264c07045f05d",
    urls = [
        "https://github.com/bazelbuild/rules_go/releases/download/v{version}/rules_go-v{version}.zip".format(version = "0.38.1"),
    ],
)

load("@io_bazel_rules_go//go:deps.bzl", "go_register_toolchains", "go_rules_dependencies")

go_rules_dependencies()

go_register_toolchains(version = "1.20.2")

## Google Test:

http_archive(
    name = "com_google_googletest",
    strip_prefix = "googletest-{version}".format(version = "1.13.0"),
    urls = ["https://github.com/google/googletest/archive/refs/tags/v{version}.tar.gz".format(version = "1.13.0")],
)

## Proto:

# NOTE: Version 22.2 won't build due to some broken Abseil dependency.
http_archive(
    name = "com_google_protobuf",
    sha256 = "3bd7828aa5af4b13b99c191e8b1e884ebfa9ad371b0ce264605d347f135d2568",
    strip_prefix = "protobuf-{version}".format(version = "3.19.4"),
    urls = ["https://github.com/protocolbuffers/protobuf/archive/v{version}.tar.gz".format(version = "3.19.4")],
)

load("@com_google_protobuf//:protobuf_deps.bzl", "protobuf_deps")

protobuf_deps()

## Buildifier:
http_archive(
    name = "com_github_bazelbuild_buildtools",
    sha256 = "ae34c344514e08c23e90da0e2d6cb700fcd28e80c02e23e4d5715dddcb42f7b3",
    strip_prefix = "buildtools-{version}".format(version = "4.2.2"),
    urls = ["https://github.com/bazelbuild/buildtools/archive/refs/tags/{version}.tar.gz".format(version = "4.2.2")],
)
