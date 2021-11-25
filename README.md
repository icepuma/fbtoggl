# fbtoggl
Interact with track.toggl.com via terminal.

## CI
![example workflow](https://github.com/icepuma/fbtoggl/actions/workflows/ci.yaml/badge.svg)

## Installation
* cargo
  ```bash
  cargo install fbtoggl
  ```
* Precompiled binary

## Shell completions

WIP

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

#### Create
```bash
fbtoggl create --project "<project>" --description "<description>" --duration "8 hours"
```

```bash
fbtoggl create --project "<project>" --description "<description>" --duration "8 hours" --lunch-break
```
