# Repository Self-Development

## Validate and promote a self-dev build

For local self-dev work on Linux x86_64, follow the documented source-build workflow: `scripts/dev_cargo.sh build --release -p jcode --bin jcode`. Validate runtime behavior against the built `target/selfdev/jcode` with its own socket, for example `./target/selfdev/jcode run --no-update --socket /run/user/1000/jcode-mytest.sock '<prompt>'`, so checks do not silently use the shared daemon. Only after validation, run `scripts/install_release.sh` to promote the release build to the active local/source-build channel. This updates the launcher to `~/.jcode/builds/current/jcode`; it is not the stable release channel.
