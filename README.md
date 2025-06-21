<div align="center">

# fbtoggl

![https://crates.io/crates/fbtoggl](https://img.shields.io/crates/v/fbtoggl)
![https://github.com/icepuma/fbtoggl/actions/workflows/ci.yaml](https://github.com/icepuma/fbtoggl/actions/workflows/ci.yaml/badge.svg)

Interact with track.toggl.com via terminal.

[Installation](#installation) •
[Usage](#usage)

</div>

## Installation
* cargo
  ```bash
  cargo install fbtoggl
  ```
* Precompiled binary

## Shell completions

Generate shell completions for your shell:

```bash
# Bash
fbtoggl --completions bash > ~/.local/share/bash-completion/completions/fbtoggl

# Zsh
fbtoggl --completions zsh > ~/.zfunc/_fbtoggl
# Add to ~/.zshrc: fpath=(~/.zfunc $fpath)

# Fish
fbtoggl --completions fish > ~/.config/fish/completions/fbtoggl.fish
```

## Usage

### Init
1. Get API token from [profile page](https://track.toggl.com/profile).
2. Call `fbtoggl init` which prompts an input for the API token

### Workspaces
```bash
fbtoggl workspaces list
```

### Projects
```bash
fbtoggl projects list
```

### Clients
```bash
fbtoggl clients list
```

```bash
fbtoggl clients create --name "<name>"
```

### Time entries

#### List
```bash
fbtoggl time-entries list [--range "today"]
```

#### Details
You can find the `<time entry id>` via `JSON` output of all time-entries
or the `time-entries start` command prompts it after starting a timer.

```bash
fbtoggl time-entries details --id "<time entry id>"
```

#### Create
```bash
fbtoggl time-entries create --project "<project>" --description "<description>" --start "today at 6am" --duration "8 hours" [--lunch-break]
```

```bash
fbtoggl time-entries create --project "<project>" --description "<description>" --start "today at 6am" --end "today at 6pm" [--lunch-break]
```

#### Start
```bash
fbtoggl time-entries start --project "<project>" --description "<description>"
```

#### Stop
You can find the `<time entry id>` via `JSON` output of all time-entries
or the `time-entries start` command prompts it after starting a timer.

```bash
fbtoggl time-entries start --id "<time entry id>" --project "<project>" --description "<description>"
```

#### Delete
You can find the `<time entry id>` via `JSON` output of all time-entries
or the `time-entries start` command prompts it after starting a timer.

```bash
fbtoggl time-entries delete --id "<time entry id>"
```
