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
fbtoggle workspaces list
```

### Projects
```bash
fbtoggle projects list
```

### Clients
```bash
fbtoggle clients list
```

### Time entries

#### Create
```bash
fbtoggle create --project "<project>" --description "<description>" --duration "<duration-in-minutes>"
