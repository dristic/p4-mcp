# P4-MCP: Perforce Model Context Protocol Server

A Model Context Protocol (MCP) server that provides AI assistants like Claude with the ability to interact with Perforce (P4) version control systems.

## Features

This MCP server exposes the following Perforce commands as tools:

- **p4_status** - Get Perforce workspace status
- **p4_sync** - Sync files from Perforce depot
- **p4_edit** - Open file(s) for edit in Perforce
- **p4_add** - Add new file(s) to Perforce
- **p4_submit** - Submit changes to Perforce
- **p4_revert** - Revert files in Perforce
- **p4_opened** - List files opened for edit
- **p4_changes** - List recent changes

## Prerequisites

- Rust (1.70 or later)
- Perforce CLI (`p4`) installed and configured (for real P4 operations)
- MCP-compatible client (like Claude Desktop)

## Installation

1. Clone this repository:
```bash
git clone <repository-url>
cd p4-mcp
```

2. Build the project:
```bash
cargo build --release
```

## Configuration

### Mock Mode (for testing)

Set the `P4_MOCK_MODE` environment variable to enable mock responses without requiring a real Perforce connection:

```bash
export P4_MOCK_MODE=1  # On Unix/Linux/macOS
set P4_MOCK_MODE=1     # On Windows
```

In mock mode, all P4 commands return simulated responses for testing purposes.

### Real Perforce Mode

For real Perforce integration, ensure:

1. The `p4` CLI tool is installed and in your PATH
2. Your Perforce workspace is properly configured
3. You have valid Perforce credentials and connection settings

## Usage

### Running the Server

```bash
cargo run --release
```

Or run the built binary directly:

```bash
./target/release/p4-mcp
```

### Command Line Options

- `--debug` or `-d`: Enable debug logging

### Integration with Claude Desktop

Add the following to your Claude Desktop MCP configuration:

```json
{
  "mcpServers": {
    "p4-mcp": {
      "command": "path/to/p4-mcp/target/release/p4-mcp",
      "args": [],
      "env": {
        "P4_MOCK_MODE": "1"
      }
    }
  }
}
```

Replace `path/to/p4-mcp` with the actual path to your compiled binary.

## Available Tools

### p4_status
Get the status of files in your Perforce workspace.

**Parameters:**
- `path` (optional): Specific path to check status for

**Example:**
```json
{
  "name": "p4_status",
  "arguments": {
    "path": "//depot/main/src/..."
  }
}
```

### p4_sync
Synchronize files from the Perforce depot to your workspace.

**Parameters:**
- `path` (optional): Path to sync (defaults to "...")
- `force` (optional): Force sync, overwriting local changes

**Example:**
```json
{
  "name": "p4_sync",
  "arguments": {
    "path": "//depot/main/...",
    "force": false
  }
}
```

### p4_edit
Open files for editing in Perforce.

**Parameters:**
- `files` (required): Array of file paths to open for edit

**Example:**
```json
{
  "name": "p4_edit",
  "arguments": {
    "files": ["src/main.rs", "src/lib.rs"]
  }
}
```

### p4_add
Add new files to Perforce.

**Parameters:**
- `files` (required): Array of file paths to add

**Example:**
```json
{
  "name": "p4_add",
  "arguments": {
    "files": ["new_file.txt", "another_file.rs"]
  }
}
```

### p4_submit
Submit changes to Perforce.

**Parameters:**
- `description` (required): Change description
- `files` (optional): Specific files to submit

**Example:**
```json
{
  "name": "p4_submit",
  "arguments": {
    "description": "Fix bug in authentication module",
    "files": ["src/auth.rs", "tests/auth_test.rs"]
  }
}
```

### p4_revert
Revert files in Perforce, discarding local changes.

**Parameters:**
- `files` (required): Array of file paths to revert

**Example:**
```json
{
  "name": "p4_revert",
  "arguments": {
    "files": ["src/temp.rs"]
  }
}
```

### p4_opened
List files currently opened for edit.

**Parameters:**
- `changelist` (optional): Specific changelist number to query

**Example:**
```json
{
  "name": "p4_opened",
  "arguments": {
    "changelist": "12345"
  }
}
```

### p4_changes
List recent changes in Perforce.

**Parameters:**
- `max` (optional): Maximum number of changes to return (default: 10)
- `path` (optional): Path to filter changes

**Example:**
```json
{
  "name": "p4_changes",
  "arguments": {
    "max": 20,
    "path": "//depot/main/src/..."
  }
}
```

## Development

### Project Structure

```
src/
├── main.rs           # Entry point and server setup
├── mcp/
│   ├── mod.rs        # MCP server implementation
│   └── types.rs      # MCP protocol types
└── p4/
    ├── mod.rs        # P4 command handler
    └── commands.rs   # P4 command definitions
```

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Debug Mode

Run with debug logging:

```bash
cargo run -- --debug
```

## Troubleshooting

### Common Issues

1. **P4 command not found**: Ensure the `p4` CLI is installed and in your PATH
2. **Permission denied**: Check your Perforce credentials and workspace permissions
3. **Connection issues**: Verify your P4PORT, P4USER, and P4CLIENT environment variables

### Logging

Use the `--debug` flag to enable detailed logging:

```bash
./p4-mcp --debug
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Roadmap

- [ ] Support for P4 branches and streams
- [ ] File diff capabilities
- [ ] Integration with P4 changelists
- [ ] Support for P4 shelving operations
- [ ] Enhanced error handling and reporting