# Devcontainer Wrapper — TODO

A checklist for building a small CLI that manages development containers through Podman or Docker.

## 1. Goals and design decisions

- [ ] Provide a simple CLI for creating, opening, listing, and managing development containers.
- [ ] Support Podman first; keep the container-engine calls behind a small adapter so Docker can be supported too.
- [ ] Only show containers created/managed by this tool in `devcontainer list`.
- [ ] Identify a project by its canonical (absolute, symlink-resolved) workspace path, not only by its container name.
- [ ] Use container **labels** as the source of truth for ownership and project metadata.
- [ ] Use image **tags** only to name/version built images; do not use tags as a substitute for container labels.
- [ ] Keep a reusable global default configuration and support project-specific configuration.
- [ ] Do not require Compose for a basic single-container project. Support Compose as an optional feature for multi-service projects.
- [ ] Keep project files and source code on the host; mount the workspace into the container.
- [ ] Make destructive operations explicit. Never prune or remove unrelated containers, images, or volumes.

## 2. Proposed CLI

### Core commands

- [ ] `devcontainer run` — resolve config, build the image if needed, create the container, and start it in the background.
- [ ] `devcontainer open` — find the current workspace's container, start it if stopped, then open an interactive shell with `exec`.
- [ ] `devcontainer list` — list only containers with the wrapper's managed label.
- [ ] `devcontainer stop` — stop the current workspace's container.
- [ ] `devcontainer restart` — restart the current workspace's container.
- [ ] `devcontainer remove` — remove the current workspace's container (ask for confirmation where appropriate).
- [ ] `devcontainer logs` — show logs for the current workspace's container; consider a `--follow` option.
- [ ] `devcontainer inspect` — show the resolved config and container details.

### Configuration and maintenance commands

- [ ] `devcontainer init` — create `.devcontainer/` in the current project, optionally copying the global default template.
- [ ] `devcontainer config` — show the configuration selected for the current workspace.
- [ ] `devcontainer config --global` — show or edit the global wrapper configuration.
- [ ] `devcontainer build` — build/rebuild the configured image without opening a shell.
- [ ] `devcontainer exec <command...>` — run an arbitrary command in the current workspace's container.
- [ ] `devcontainer prune` — offer to remove stopped containers managed by this tool only; show the candidates before removal.
- [ ] `devcontainer version` — print wrapper and selected engine versions.
- [ ] `devcontainer help` — document commands, options, and config discovery.

### Useful options to consider

- [ ] `--project <path>` — operate on a workspace other than the current directory.
- [ ] `--engine podman|docker` — override the configured engine for one invocation.
- [ ] `--config <path>` — use an explicit config file.
- [ ] `--shell <path>` — override the shell used by `open`.
- [ ] `--rebuild` — rebuild the image before starting.
- [ ] `--no-cache` — build without the engine's build cache.
- [ ] `--detach` — keep the container running in the background (likely the default for `run`).
- [ ] `--` — pass remaining arguments to a command executed inside the container.

## 3. Configuration layout

Respect the XDG Base Directory variables, with sensible defaults when they are unset:

```text
$XDG_CONFIG_HOME/devcontainer-wrapper/       # default: ~/.config/devcontainer-wrapper
├── config.toml                               # wrapper settings
└── default/                                  # global default devcontainer template
    ├── devcontainer.json
    └── Dockerfile

$XDG_DATA_HOME/devcontainer-wrapper/          # default: ~/.local/share/devcontainer-wrapper
└── ...                                        # optional wrapper-owned persistent metadata

$XDG_CACHE_HOME/devcontainer-wrapper/         # default: ~/.cache/devcontainer-wrapper
└── ...                                        # optional build/config cache

my-project/
├── src/
└── .devcontainer/
    ├── devcontainer.json                     # project-specific configuration
    ├── Dockerfile                             # optional project image definition
    └── compose.yaml                           # optional multi-service setup
```

- [ ] Implement XDG path resolution (`XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME`).
- [ ] Create the global config/template directories when needed; do not silently overwrite existing user files.
- [ ] Search upward from the current directory for a project `.devcontainer/` directory, stopping at the filesystem root.
- [ ] If a project config is found, use it. Otherwise, use the global `default/` template.
- [ ] Keep project config selection separate from wrapper-wide settings in `config.toml`.
- [ ] Do not automatically merge a project config with the global template in the first version; merging build settings, mounts, and services can be ambiguous.
- [ ] Make `devcontainer init` copy the global template into the project so the project can customize it independently.
- [ ] Document which config format and which subset of the Dev Container Specification are supported.

### Example global `config.toml`

```toml
# ~/.config/devcontainer-wrapper/config.toml

engine = "podman"             # podman | docker | auto
project_config_dir = ".devcontainer"
default_profile = "default"
default_shell = "/bin/bash"
auto_start = true
workspace_target = "/workspace"

[container]
# A persistent idle process keeps the container available for `exec` sessions.
keepalive_command = ["sleep", "infinity"]

[labels]
managed = "devcontainer-wrapper.managed"
workspace = "devcontainer-wrapper.workspace"
config = "devcontainer-wrapper.config"
project_id = "devcontainer-wrapper.project-id"
```

- [ ] Validate the global TOML config and report the file and setting that caused an error.
- [ ] Define defaults in code so a minimal or missing `config.toml` still works.
- [ ] Decide whether `engine = "auto"` should prefer Podman when both engines are installed.
- [ ] Avoid storing secrets in `config.toml` or labels. Pass secrets through a dedicated, documented mechanism if required.

### Example project `devcontainer.json`

A deliberately small first-version example using a Dockerfile:

```json
{
  "name": "my-project",
  "build": {
    "dockerfile": "Dockerfile",
    "context": ".."
  },
  "workspaceFolder": "/workspace",
  "remoteUser": "dev"
}
```

- [ ] Resolve paths in `build.dockerfile` and `build.context` relative to the config file/project according to the supported spec behavior.
- [ ] Mount the project workspace at `workspaceFolder` (or a documented default such as `/workspace`).
- [ ] Decide how to handle projects without a Dockerfile (for example, an `image` property).
- [ ] Support a clearly documented initial subset of `devcontainer.json`; reject unsupported fields with a helpful warning/error rather than silently pretending to support them.
- [ ] Consider adding support for `workspaceMount`, `containerEnv`, `remoteEnv`, `forwardPorts`, `runArgs`, `postCreateCommand`, and `remoteUser` in later iterations.

## 4. Container identity and labels

Add labels to every container created by the wrapper. Example values:

```text
devcontainer-wrapper.managed=true
devcontainer-wrapper.workspace=/absolute/path/to/project
devcontainer-wrapper.project-id=<stable-id>
devcontainer-wrapper.config=/absolute/path/to/.devcontainer/devcontainer.json
```

- [ ] Add the managed label to every wrapper-created container.
- [ ] Add a canonical workspace-path label.
- [ ] Add a stable project ID (for example, a hash of the canonical workspace path).
- [ ] Add the config path/profile label where applicable.
- [ ] Filter `list` and all discovery operations by the managed label.
- [ ] Filter current-project operations by both managed status and workspace/project ID.
- [ ] Handle multiple matching containers explicitly (for example, report ambiguity or select by a documented profile/name); do not pick an arbitrary result.
- [ ] Treat labels as metadata, not as an authorization boundary or a security mechanism.

### Labels vs. image tags

- **Label:** key/value metadata attached to a container or image; useful for filtering managed containers.
- **Tag:** a name/version attached to an image, such as `my-devcontainer:latest` or `my-devcontainer:rust`.
- [ ] Generate predictable image names/tags for project builds.
- [ ] Do not assume an image tag uniquely identifies a running container; several containers can use the same image.

## 5. Podman command reference

These are the main engine commands the wrapper will likely call. Prefer invoking the executable with an argument array (not a shell-concatenated command string).

### Build and image operations

```bash
# Build an image from a Dockerfile
podman build -f .devcontainer/Dockerfile -t my-devcontainer:latest .

# Pull an existing image
podman pull debian:stable-slim

# List images
podman image ls

# Add another tag to an image
podman tag my-devcontainer:latest my-devcontainer:v1

# Inspect image metadata
podman image inspect my-devcontainer:latest
```

- [ ] Build using the configured Dockerfile and build context.
- [ ] Generate a stable image name for each project/profile.
- [ ] Decide how to handle rebuilds and stale images.
- [ ] Do not remove images automatically when removing a container unless explicitly requested.

### Create, start, run, stop, and remove

```bash
# Create a container without starting it
podman create --name my-project my-devcontainer:latest sleep infinity

# Start an existing container
podman start my-project

# Create and start a detached persistent container
podman run -d --name my-project my-devcontainer:latest sleep infinity

# Stop, restart, or remove a container
podman stop my-project
podman restart my-project
podman rm my-project

# Force-stop or pause/unpause (advanced operations)
podman kill my-project
podman pause my-project
podman unpause my-project
```

- [ ] Decide whether implementation uses `podman run -d` or `podman create` followed by `podman start`.
- [ ] Keep the container's main process alive so `open` can create one or more shell sessions with `exec`.
- [ ] Detect and handle created, running, stopped, paused, and missing container states.
- [ ] Make `remove` handle stopped containers and provide an explicit force option if needed.

### Workspace bind mounts and shell access

No Compose file is required to mount the current host directory:

```bash
podman run -d \
  --name my-project \
  --label devcontainer-wrapper.managed=true \
  --label devcontainer-wrapper.workspace="$(realpath .)" \
  --mount type=bind,src="$(realpath .)",target=/workspace \
  --workdir /workspace \
  my-devcontainer:latest \
  sleep infinity

# Open an interactive shell in the running container
podman exec -it my-project /bin/bash

# Alternative shell if Bash is not installed
podman exec -it my-project /bin/sh

# Run a one-off command inside the container
podman exec my-project npm test
```

- [ ] Canonicalize the host workspace path before using it as a label or mount source.
- [ ] Mount the workspace read/write by default; consider a read-only option.
- [ ] Set the container working directory to the configured workspace folder.
- [ ] Pick the configured shell, then fall back from Bash to `sh` when appropriate.
- [ ] Consider Podman's user-namespace/ownership behavior (for example, rootless operation and `--userns=keep-id`) and document the chosen behavior.
- [ ] Document SELinux bind-mount options (`:Z`/`:z` or equivalent mount options) for SELinux-enabled hosts; do not add them blindly on every system.

### Listing, filtering, inspecting, and status

```bash
# Running containers
podman ps

# All containers, including stopped ones
podman ps -a

# Only containers managed by this wrapper
podman ps -a --filter label=devcontainer-wrapper.managed=true

# Only the current workspace's managed container
podman ps -a \
  --filter label=devcontainer-wrapper.managed=true \
  --filter label=devcontainer-wrapper.workspace="$(realpath .)"

# Inspect a container
podman inspect my-project

# Read its state
podman inspect --format '{{.State.Status}}' my-project

# Check whether a named container exists
podman container exists my-project

# Show logs, ports, processes, and resource usage
podman logs my-project
podman logs --follow my-project
podman port my-project
podman top my-project
podman stats my-project
```

- [ ] Use `podman ps -a` with label filters for discovery, so stopped devcontainers are also found.
- [ ] Parse structured inspect output rather than scraping human-readable tables.
- [ ] Use exit status from `podman container exists` to check existence.
- [ ] Distinguish engine errors from “no matching container”.
- [ ] Keep normal `list` output concise; provide a verbose mode for state, image, workspace, and ID.

### Logs and process interaction

```bash
podman logs my-project
podman logs --follow my-project
podman exec -it my-project /bin/bash
podman attach my-project
podman wait my-project
```

- [ ] Use `exec` for opening a new shell; use `attach` only when attaching to the container's main process is intended.
- [ ] Support log following and interrupt handling.
- [ ] Preserve the exit code of commands run through `devcontainer exec`.

### Volumes and networks (optional for later)

```bash
# Volumes
podman volume create my-volume
podman volume ls
podman volume inspect my-volume
podman volume rm my-volume

# Networks
podman network create my-network
podman network ls
podman network inspect my-network
podman network connect my-network my-project
podman network disconnect my-network my-project
podman network rm my-network
```

- [ ] Add named-volume support only when a use case requires data persistence beyond the workspace bind mount.
- [ ] Add custom network support for multi-container projects.
- [ ] If the wrapper creates volumes or networks, label/name them so cleanup can identify wrapper-owned resources safely.

### Events and cleanup (optional)

```bash
podman events
podman system df
podman image prune
podman system prune
```

- [ ] Consider `podman events` for detecting container exits or external removal.
- [ ] Show disk usage with `podman system df` in a diagnostics command.
- [ ] Never run broad `podman system prune` as part of normal wrapper cleanup; it can affect unrelated resources.
- [ ] Make wrapper cleanup target only resources that can be positively identified as wrapper-owned.

## 6. Optional Compose support

- [ ] Detect a Compose file only when the project configuration explicitly requests Compose, or document a deliberate discovery rule.
- [ ] Support `compose.yaml`/`compose.yml` through the configured Compose provider.
- [ ] For Podman, check that a Compose provider (such as `podman-compose` or Docker Compose) is installed; `podman compose` delegates to an external provider.
- [ ] Keep Compose project naming isolated so unrelated Compose projects are not listed as this wrapper's devcontainers.
- [ ] Document that a Compose project can contain multiple services/containers and that `open` must select the configured development service.
- [ ] Keep single-container `Dockerfile` builds independent of Compose.

Example Compose service with a workspace bind mount:

```yaml
services:
  dev:
    build:
      context: ..
      dockerfile: .devcontainer/Dockerfile
    working_dir: /workspace
    volumes:
      - ..:/workspace
    command: sleep infinity
```

## 7. Podman and Docker compatibility

- [ ] Implement a small engine interface for the operations the wrapper needs: build, list/filter, inspect, create/run, start, stop, remove, exec, and logs.
- [ ] Keep Podman-specific and Docker-specific argument differences inside their respective adapters.
- [ ] Detect whether the selected engine executable exists and report a helpful error if it does not.
- [ ] Capture stdout, stderr, and exit status separately.
- [ ] Avoid shell interpolation; pass arguments as an array to the process-spawning API.
- [ ] Test rootless Podman and Docker separately.
- [ ] Document differences around Compose, user IDs, networking, mounts, and permissions.

## 8. Implementation checklist

### MVP

- [ ] Create the CLI project and command parser.
- [ ] Implement config path resolution and defaults.
- [ ] Implement project discovery (`.devcontainer/`) and global fallback.
- [ ] Implement config parsing and validation.
- [ ] Implement Podman command execution with safe argument handling.
- [ ] Implement image build.
- [ ] Implement container creation with managed/workspace labels and bind mount.
- [ ] Implement `list` using label filters only.
- [ ] Implement `open` using `start` + `exec`.
- [ ] Implement `stop`, `restart`, `remove`, `logs`, and `inspect`.
- [ ] Add useful error messages for missing engine, missing config, build failure, no matching container, and multiple matches.

### Quality and safety

- [ ] Test paths containing spaces and non-ASCII characters.
- [ ] Test symlinked project directories and ensure the same canonical path identifies the same workspace.
- [ ] Test projects with no config, global config only, and project config.
- [ ] Test stopped, running, paused, exited, missing, and duplicate containers.
- [ ] Test engine command failures and preserve useful stderr output.
- [ ] Ensure list/cleanup operations never include containers without the managed label.
- [ ] Ensure `remove` and `prune` cannot accidentally remove unrelated user containers.
- [ ] Add unit tests for config resolution, labels, project IDs, and command construction.
- [ ] Add integration tests using a disposable Podman environment.
- [ ] Document permissions, rootless mode, and host/container file ownership.

### Later enhancements

- [ ] Docker engine adapter.
- [ ] Compose/multi-service support.
- [ ] Port forwarding and port display.
- [ ] Persistent named volumes.
- [ ] Multiple profiles per project (for example, `default`, `rust`, `node`).
- [ ] Lifecycle hooks such as `postCreateCommand`.
- [ ] Better shell selection and terminal configuration.
- [ ] Container health/status display.
- [ ] Optional JSON output for scripting.

## 9. Documentation links

- [Podman command reference](https://docs.podman.io/en/latest/Commands.html)
- [podman ps](https://docs.podman.io/en/latest/markdown/podman-ps.1.html)
- [podman inspect](https://docs.podman.io/en/latest/markdown/podman-inspect.1.html)
- [podman run](https://docs.podman.io/en/latest/markdown/podman-run.1.html)
- [podman exec](https://docs.podman.io/en/latest/markdown/podman-exec.1.html)
- [podman compose](https://docs.podman.io/en/latest/markdown/podman-compose.1.html)
- [Dev Container Specification](https://containers.dev/implementors/spec/)
- [Dev Container `devcontainer.json` reference](https://containers.dev/implementors/json_reference/)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir/latest/)

