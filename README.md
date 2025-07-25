# Starship Custom - Personal Modifications

This Starship fork includes some custom modifications to improve the prompt user experience.

## 🚀 New Features

### 1. Truncation Modes for Git Branch

The `git_branch` module now supports different truncation modes for long branch names.

#### New Option
- **`truncation_mode`**: Controls how branch names are truncated
  - `"right"` (default): Truncates from the end (original behavior)
  - `"left"`: Truncates from the beginning, keeping the final part

#### Configuration Example
```toml
[git_branch]
truncation_length = 20
truncation_mode = "left"  # or "right"
truncation_symbol = "…"
```

#### Output Examples
For a branch named `feature/JIRATAG-12345/username_featurename`:

- **`right` mode** (20 characters): `feature/JIRATAG-1234…`
- **`left` mode** (20 characters): `…username_featurename`

### 2. Custom Units for Memory Usage

The `memory_usage` module now supports custom measurement units to display memory usage.

#### New Option
- **`unit`**: Specifies the measurement unit to use
  - Binary units: `"B"`, `"KiB"`, `"MiB"`, `"GiB"`, `"TiB"`
  - Decimal units: `"KB"`, `"MB"`, `"GB"`, `"TB"`
  - Short units: `"K"`, `"M"`, `"G"`, `"T"`

#### New Variable
- **`used`**: Shows only the used memory (without the total)

#### Configuration Example
```toml
[memory_usage]
disabled = false
threshold = 75
format = "via $symbol[${used}]($style) "
unit = "G"  # Forces the use of G
```

#### Output Examples
With `unit = "G"`:
- `ram`: `1.5G/8.0G`
- `used`: `1.5G`
