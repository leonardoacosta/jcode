#!/usr/bin/env python3
"""Raw source identity and opt-in Linux namespace-private compiler inputs.

Source attestation is not reproducible-build/toolchain attestation. Host same-UID
process injection, kernel compromise and malicious compiler output are outside
this boundary. Compiler children cannot write or replace source snapshot bytes.
"""
import hashlib
import fcntl
import io
from pathlib import Path
import shutil
import os
import stat
import subprocess
import sys
import tarfile
import time
import tomllib



def sealed(content):
    fd = os.memfd_create("jcode-input", os.MFD_ALLOW_SEALING)
    with os.fdopen(os.dup(fd), "wb") as stream:
        stream.write(content)
    os.lseek(fd, 0, os.SEEK_SET)
    fcntl.fcntl(fd, fcntl.F_ADD_SEALS, fcntl.F_SEAL_WRITE | fcntl.F_SEAL_GROW |
                fcntl.F_SEAL_SHRINK | fcntl.F_SEAL_SEAL)
    return fd


def snapshot(root, immutable=False):
    env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    env["GIT_OPTIONAL_LOCKS"] = "0"

    def git(*args):
        return subprocess.check_output(
            ["git", "-c", "core.fsmonitor=false", "-c", "core.untrackedCache=false", *args],
            cwd=root, env=env, stderr=subprocess.DEVNULL, timeout=30)

    def credential(path):
        parts = os.fsdecode(path).lower().split("/")
        if any(a == ".cargo" and b in ("credentials", "credentials.toml") or
               a == ".docker" and b == "config.json" for a, b in zip(parts, parts[1:])):
            return True
        return any(p in (".aws", ".ssh", ".gnupg", ".env", ".npmrc", ".pypirc", ".netrc",
                         "credentials.json", ".git-credentials", "id_rsa", "id_ed25519")
                   or p.startswith(".env.") or p.endswith((".pem", ".key", ".p12", ".pfx"))
                   for p in parts)

    head = git("rev-parse", "HEAD").strip()
    hash_name = "sha1" if len(head) == 40 else "sha256"
    committed = {}
    for entry in git("ls-tree", "-rz", head.decode()).split(b"\0"):
        if entry:
            meta, path = entry.split(b"\t", 1)
            mode, kind, oid = meta.split()
            committed[path] = (mode, oid)
    paths = set(git("ls-files", "-z", "--cached", "--others", "--exclude-standard").split(b"\0"))
    paths.discard(b"")
    tracked = set(git("ls-files", "-z", "--cached").split(b"\0")) | committed.keys()
    digest = hashlib.sha256(head + b"\0")
    dirty = False
    if immutable:
        paths.add(b"Cargo.lock")
    files = []
    total = 0
    root_fd = os.open(root, os.O_RDONLY | os.O_DIRECTORY)
    for path in sorted(paths | committed.keys()):
        digest.update(path + b"\0")
        if credential(path):
            # Never read credentials. A tracked credential could be an arbitrary
            # build input, so this source state cannot prove cleanliness.
            if path in tracked:
                raise ValueError("tracked private input")
            dirty = True
            digest.update(b"excluded\0")
            continue
        if path not in tracked and any(p.lower() in (b"target", b"node_modules", b"dist", b".git", b".jcode")
                                         for p in path.split(b"/")):
            dirty = True
            digest.update(b"excluded\0")
            continue
        parts = path.split(b"/")
        if any(p in (b"", b".", b"..") for p in parts):
            raise ValueError("invalid source path")
        parent = os.dup(root_fd)
        try:
            for part in parts[:-1]:
                next_fd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=parent)
                os.close(parent)
                parent = next_fd
            fd = os.open(parts[-1], os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
            with os.fdopen(fd, "rb") as source:
                info = os.fstat(source.fileno())
                if not stat.S_ISREG(info.st_mode):
                    raise ValueError("unsupported source file")
                blob = hashlib.new(hash_name, b"blob " + str(info.st_size).encode() + b"\0")
                content = source.read(64 * 1024 * 1024 + 1)
                if len(content) > 64 * 1024 * 1024:
                    raise ValueError("source file limit exceeded")
                blob.update(content)
                total += len(content)
                if total > 512 * 1024 * 1024 or len(files) >= 4096:
                    raise ValueError("source snapshot limit exceeded")
                after = os.fstat(source.fileno())
                if (info.st_size, info.st_mtime_ns, info.st_ctime_ns) != (
                        after.st_size, after.st_mtime_ns, after.st_ctime_ns):
                    raise ValueError("source changed during snapshot")
                mode = b"100755" if info.st_mode & 0o111 else b"100644"
                current = (mode, blob.hexdigest().encode())
                if immutable:
                    files.append((os.fsdecode(path), mode == b"100755", sealed(content)))
        except FileNotFoundError:
            current = (b"missing", b"")
        finally:
            os.close(parent)
        dirty |= committed.get(path) != current
        digest.update(current[0] + b"\0" + current[1] + b"\0")
    os.close(root_fd)
    if git("rev-parse", "HEAD").strip() != head:
        raise ValueError("HEAD changed during snapshot")
    return {"git_hash": git("rev-parse", "--short", head.decode()).decode().strip(),
            "git_dirty": str(dirty).lower(), "fingerprint": digest.hexdigest(),
            "git_date": git("log", "-1", "--format=%ci").decode().strip()}, files


def build(root, argv):
    if sys.platform != "linux" or not shutil.which("bwrap"):
        raise ValueError("immutable build requires Linux bubblewrap")
    if not argv or argv[0] != "build":
        raise ValueError("immutable mode supports cargo build only")
    # No command/config/path escape hatches. Project Cargo config is snapshotted.
    if any(a == "--" or a.startswith(("--manifest-path", "--config", "--target-dir")) for a in argv):
        raise ValueError("immutable build does not support manifest/config/target-dir overrides")
    metadata, files = snapshot(root, immutable=True)
    if not any(p == "Cargo.lock" for p, _, _ in files):
        raise ValueError("immutable build requires a local Cargo.lock")
    toolchain = Path(subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip()).resolve()
    cargo_home = Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo"))).resolve()
    selected = {p for p, _, _ in files}
    # Cargo path dependencies and source replacements must resolve inside the
    # selected snapshot, never writable output/temp or host system mounts.
    for path, _, fd in files:
        if not path.endswith("Cargo.toml") and path not in (".cargo/config", ".cargo/config.toml"):
            continue
        data = os.pread(fd, os.fstat(fd).st_size, 0).decode()
        base = Path(path).parent if path.endswith("Cargo.toml") else Path(".")
        def check_paths(value):
            if isinstance(value, dict):
                for key, child in value.items():
                    if key in ("path", "directory", "local-registry") and isinstance(child, str):
                        relative = Path(os.path.normpath(str(base / child)))
                        if relative.is_absolute() or ".." in relative.parts:
                            raise ValueError("external Cargo input: " + path)
                        if not any(p == str(relative) or p.startswith(str(relative) + "/") for p in selected):
                            raise ValueError("unselected Cargo input: " + path)
                    check_paths(child)
            elif isinstance(value, list):
                for child in value:
                    check_paths(child)
        check_paths(tomllib.loads(data))
    target = Path(os.environ.get("CARGO_TARGET_DIR", str(Path(root) / "target"))).resolve()
    target = target / "immutable" / metadata["fingerprint"]
    target.mkdir(parents=True, exist_ok=True)
    args = ["bwrap", "--unshare-all", "--unshare-user", "--die-with-parent",
            "--new-session", "--cap-drop", "ALL", "--clearenv"]
    for path in ["/usr", "/bin", "/lib", "/lib64"]:
        if Path(path).exists():
            args += ["--ro-bind", path, path]
    for path in ["/etc/ld.so.cache", "/etc/localtime"]:
        if Path(path).exists():
            args += ["--ro-bind", path, path]
    args += ["--ro-bind", str(toolchain), "/toolchain", "--tmpfs", "/cargo"]
    for name in ["registry", "git"]:
        path = cargo_home / name
        if path.exists():
            args += ["--ro-bind", str(path), "/cargo/" + name]
    args += ["--bind", str(target), "/target", "--tmpfs", "/tmp", "--dir", "/home/build",
             "--proc", "/proc", "--dev", "/dev", "--tmpfs", "/src"]
    # One sealed archive avoids bubblewrap's argument and inherited-FD limits
    # on the real workspace. The outer namespace extracts only trusted paths.
    archive = io.BytesIO()
    with tarfile.open(fileobj=archive, mode="w") as tar:
        for path, executable, fd in files:
            info = tarfile.TarInfo(path)
            info.mtime = time.time()
            info.mode = 0o555 if executable else 0o444
            info.size = os.fstat(fd).st_size
            with os.fdopen(os.dup(fd), "rb") as stream:
                tar.addfile(info, stream)
            os.close(fd)
    payload = sealed(archive.getvalue())
    args += ["--ro-bind-data", str(payload), "/source.tar"]
    attestation = sealed("\n".join(k + "=" + v for k, v in metadata.items()).encode())
    args += ["--ro-bind-data", str(attestation), "/jcode-immutable",
             "--chdir", "/src"]
    env = {"PATH": "/toolchain/bin:/usr/bin:/bin", "HOME": "/home/build", "CARGO_HOME": "/cargo",
           "CARGO_TARGET_DIR": "/target", "RUSTC": "/toolchain/bin/rustc",
           "TMPDIR": "/tmp", "CARGO_NET_OFFLINE": "true",
           "JCODE_BUILD_GIT_HASH": metadata["git_hash"], "JCODE_BUILD_GIT_DIRTY": metadata["git_dirty"],
           "JCODE_BUILD_GIT_DATE": metadata["git_date"],
           "JCODE_BUILD_SOURCE_FINGERPRINT": metadata["fingerprint"]}
    # Only documented build policy knobs cross the boundary, never arbitrary
    # host env, Cargo credentials/config, LD_PRELOAD or caller provenance.
    for name in ["CARGO_BUILD_JOBS", "CARGO_INCREMENTAL", "CARGO_PROFILE_SELFDEV_INCREMENTAL",
                 "CARGO_PROFILE_SELFDEV_CODEGEN_UNITS", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER",
                 "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS", "JCODE_PARALLEL_FRONTEND",
                 "JCODE_FRONTEND_THREADS"]:
        if name in os.environ:
            env[name] = os.environ[name]
    wrapper = os.environ.get("RUSTC_WRAPPER")
    if wrapper:
        local = Path(wrapper).absolute().relative_to(Path(root))
        if str(local) not in {p for p, _, _ in files}:
            raise ValueError("immutable compiler wrapper is not a selected source input")
        env["RUSTC_WRAPPER"] = "/src/" + str(local)
    for name, value in env.items():
        args += ["--setenv", name, value]
    print("dev_cargo: immutable source=" + metadata["fingerprint"] + " target=" + str(target), file=sys.stderr)
    argv = [a for a in argv if a not in ("--offline", "--locked", "--frozen")]
    # Only this trusted bootstrap runs with a writable /src. The compiler and
    # all build scripts start after the nested namespace locks that mount.
    bootstrap = 'tar --extract --file=/source.tar --directory=/src; exec "$@"'
    inner = ["/usr/bin/bwrap", "--unshare-user", "--unshare-pid", "--disable-userns", "--cap-drop", "ALL",
             "--ro-bind", "/", "/", "--ro-bind", "/src", "/src",
             "--bind", "/target", "/target", "--bind", "/tmp", "/tmp", "--dev", "/dev",
             "--proc", "/proc", "--chdir", "/src"]
    return subprocess.run(args + ["/bin/sh", "-ec", bootstrap, "bootstrap", *inner,
                                 "/toolchain/bin/cargo", *argv, "--offline", "--locked"],
                          pass_fds=[payload, attestation]).returncode


if __name__ == "__main__":
    try:
        if sys.argv[1] == "--fingerprint":
            metadata, _ = snapshot(sys.argv[2])
            print(metadata["git_hash"], metadata["git_dirty"], metadata["fingerprint"])
        else:
            sys.exit(build(sys.argv[1], sys.argv[2:]))
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print("dev_cargo: immutable/source snapshot failed: " + str(error), file=sys.stderr)
        sys.exit(1)
