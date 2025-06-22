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
fbtoggl completions bash > ~/.local/share/bash-completion/completions/fbtoggl

# Zsh
fbtoggl completions zsh > ~/.zfunc/_fbtoggl
# Add to ~/.zshrc: fpath=(~/.zfunc $fpath)

# Fish
fbtoggl completions fish > ~/.config/fish/completions/fbtoggl.fish
```

## Usage

### Init
1. Get API token from [profile page](https://track.toggl.com/profile).
2. Call `fbtoggl config init` which prompts an input for the API token

### Timer Management

#### Start a timer
```bash
# Start a billable timer (default)
fbtoggl start --project "<project>" --description "<description>"

# Start a non-billable timer with tags
fbtoggl start --project "<project>" --description "<description>" --non-billable --tags "tag1,tag2"
```

#### Stop a timer
```bash
# Stop the currently running timer
fbtoggl stop

# Stop a specific timer by ID
fbtoggl stop --id "<time entry id>"
```

#### Show current timer
```bash
fbtoggl current
```

#### Continue a timer
```bash
# Continue the last timer
fbtoggl continue

# Continue a specific timer by ID
fbtoggl continue --id "<time entry id>"
```

### Time Entry Management

#### List time entries
```bash
# List today's entries (default)
fbtoggl log

# List entries for a specific range
fbtoggl log --range "yesterday"
fbtoggl log --range "this-week"
fbtoggl log --range "2021-11-01|2021-11-07"

# Show missing entries (workdays only)
fbtoggl log --missing
```

#### Add completed time entry
```bash
# Add entry with duration
fbtoggl add --project "<project>" --description "<description>" --start "today at 9am" --duration "8 hours"

# Add entry with start and end time
fbtoggl add --project "<project>" --description "<description>" --start "today at 9am" --end "today at 5pm"

# Add entry with lunch break (splits into two 4-hour entries)
fbtoggl add --project "<project>" --description "<description>" --start "today at 9am" --end "today at 6pm" --lunch-break

# Add non-billable entry
fbtoggl add --project "<project>" --description "<description>" --start "today at 9am" --duration "1 hour" --non-billable
```

#### Show entry details
```bash
fbtoggl show <time-entry-id>
```

#### Edit time entry
```bash
# Edit multiple fields
fbtoggl edit <time-entry-id> --description "New description" --project "Different project"

# Toggle billable status
fbtoggl edit <time-entry-id> --toggle-billable

# Change time
fbtoggl edit <time-entry-id> --start "today at 8am" --end "today at 5pm"
```

#### Delete time entry
```bash
fbtoggl delete <time-entry-id>
```

### Reports

#### Detailed report with violations
```bash
# Today's report
fbtoggl report

# Custom range
fbtoggl report --range "this-week"
fbtoggl report --range "last-month"
```

#### Summary statistics
```bash
# This week's summary (default)
fbtoggl summary

# Custom range
fbtoggl summary --range "today"
fbtoggl summary --range "this-month"
```

### Resource Management

#### Workspaces
```bash
fbtoggl workspace list
```

#### Projects
```bash
# List active projects
fbtoggl project list

# List all projects (including archived)
fbtoggl project list --all

# Create new project
fbtoggl project create --name "Project Name" --billable

# Create project with client
fbtoggl project create --name "Project Name" --client "Client Name" --billable

# Create non-billable project with custom color
fbtoggl project create --name "Internal Project" --color "#ff5722"
```

#### Clients
```bash
# List active clients
fbtoggl client list

# List all clients (including archived)
fbtoggl client list --all

# Create new client
fbtoggl client create --name "<name>"
```

### Configuration

```bash
# Initialize configuration
fbtoggl config init

# Show current configuration
fbtoggl config show

# Set configuration value
fbtoggl config set api_token <new-token>
```

### Output Formats

All commands support different output formats:

```bash
# Default raw format
fbtoggl log

# JSON format
fbtoggl log --format json

# Table format
fbtoggl log --format table
```
