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

#### List
```bash
fbtoggl time-entries list [--range "today"]
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
