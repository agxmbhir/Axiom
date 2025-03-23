# Axiom CLI Reference

A quick reference guide for the Axiom command line interface.

## Global Options

```
axiom [OPTIONS] <COMMAND>
```

| Option | Description |
|--------|-------------|
| `-l, --log-level <LEVEL>` | Set log level (error, warn, info, debug, trace) |
| `-c, --config <FILE>` | Path to configuration file |
| `-o, --output-format <FORMAT>` | Output format (text, json) |
| `-h, --help` | Print help information |
| `-V, --version` | Print version information |

## Commands

### Initialize a Project

```bash
axiom init [DIRECTORY] [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-l, --language <LANG>` | Target implementation language |
| `-v, --verification-system <SYSTEM>` | Verification system to use |
| `-d, --domain <DOMAIN>` | Application domain |

### Generate Specification

```bash
axiom spec --requirements <FILE> --domain <DOMAIN> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-r, --requirements <FILE>` | Path to requirements file |
| `-v, --verification-language <LANG>` | Verification language (default: fstar) |
| `-d, --domain <DOMAIN>` | Domain for the specification |
| `-o, --output <FILE>` | Output file |
| `-d, --detail-level <LEVEL>` | Detail level (default: standard) |

### Validate Specification

```bash
axiom validate --spec <FILE> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-s, --spec <FILE>` | Path to specification file |
| `-d, --depth <DEPTH>` | Validation depth (basic, typecheck, formal) |
| `-r, --requirements <FILE>` | Requirements file |

### Generate Implementation

```bash
axiom implement --spec <FILE> --language <LANG> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-s, --spec <FILE>` | Path to specification file |
| `-l, --language <LANG>` | Target language |
| `-o, --optimization <LEVEL>` | Optimization level |
| `-o, --output <FILE>` | Output file |
| `--comments` | Include comments (default: true) |

### Verify Implementation

```bash
axiom verify --implementation <FILE> --spec <FILE> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-i, --implementation <FILE>` | Path to implementation file |
| `-s, --spec <FILE>` | Path to specification file |
| `-s, --system <SYSTEM>` | Verification system |
| `-o, --output <DIR>` | Output directory |
| `-p, --proof-level <LEVEL>` | Proof level (default: standard) |
| `-t, --timeout <SECONDS>` | Timeout in seconds (default: 300) |

### End-to-End Processing

```bash
axiom process --requirements <FILE> --language <LANG> --domain <DOMAIN> --output <DIR> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-r, --requirements <FILE>` | Path to requirements file |
| `-l, --language <LANG>` | Target language |
| `-d, --domain <DOMAIN>` | Domain for specification |
| `-o, --output <DIR>` | Output directory |
| `-s, --system <SYSTEM>` | Verification system |
| `--verification-language <LANG>` | Verification language |
| `-i, --interactive` | Interactive mode (default: true) |

### Translate Specifications

```bash
axiom translate --source <FILE> --target-language <LANG> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-s, --source <FILE>` | Source specification file |
| `-t, --target-language <LANG>` | Target verification language |
| `-o, --output <FILE>` | Output file |

### List Supported Features

```bash
axiom list [WHAT]
```

Options for `WHAT`: languages, verification-systems, domains, verification-languages, all (default)

### Check Tool Integration

```bash
axiom check [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-s, --system <SYSTEM>` | Verification system to check |
| `-l, --language <LANG>` | Implementation language to check |
| `-i, --install` | Install missing dependencies |

## Examples

### Verify a Cryptographic Implementation

```bash
# Generate spec from requirements
axiom spec --requirements crypto_reqs.txt --domain cryptography --output aes.fst

# Generate implementation
axiom implement --spec aes.fst --language rust --output aes.rs

# Verify implementation
axiom verify --implementation aes.rs --spec aes.fst
```

### End-to-End Processing

```bash
# Process requirements through the entire pipeline
axiom process --requirements reqs.txt --language rust --domain cryptography --output ./out
```

### Environment Variables

- `AXIOM_CONFIG`: Path to configuration file
- `AXIOM_LOG_LEVEL`: Default log level
- `AXIOM_CACHE_DIR`: Cache directory for verification artifacts