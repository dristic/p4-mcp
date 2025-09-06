# Test Data

This directory contains JSON test files for manually testing the p4-mcp server functionality.

## How to Use

### Prerequisites
- Build the project: `cargo build`
- For mock testing: Set environment variable `P4_MOCK_MODE=1`
- For real P4 testing: Ensure you have a Perforce client configured

### Running Tests

#### Mock Mode (Recommended for Development)
```powershell
$env:P4_MOCK_MODE=1; Get-Content test_data\test_p4_info.json | .\target\debug\p4-mcp.exe
```

#### Real Perforce Mode
```powershell
Get-Content test_data\test_p4_info.json | .\target\debug\p4-mcp.exe
```

## Test Files

### Basic MCP Protocol
- `test_initialize.json` - Initialize the MCP server
- `test_list_tools.json` - List all available tools
- `test_ping.json` - Ping the server (if exists)

### Perforce Commands

#### Information Commands
- `test_p4_info.json` - Get Perforce client and server information
- `test_p4_status.json` - Get workspace status for a specific path
- `test_p4_opened.json` - List files opened for edit
- `test_p4_changes.json` - List recent changes

#### File Operations
- `test_p4_edit.json` - Open files for edit
- `test_p4_add.json` - Add new files to Perforce
- `test_p4_sync_example.json` - Sync files from depot

## Example Usage

1. **Test server initialization:**
   ```powershell
   Get-Content test_data\test_initialize.json | .\target\debug\p4-mcp.exe
   ```

2. **Check available tools:**
   ```powershell
   Get-Content test_data\test_list_tools.json | .\target\debug\p4-mcp.exe
   ```

3. **Get P4 server info (safe command):**
   ```powershell
   $env:P4_MOCK_MODE=1; Get-Content test_data\test_p4_info.json | .\target\debug\p4-mcp.exe
   ```

4. **Test file operations (mock mode recommended):**
   ```powershell
   $env:P4_MOCK_MODE=1; Get-Content test_data\test_p4_edit.json | .\target\debug\p4-mcp.exe
   ```

## Creating Custom Tests

You can create your own test files using this JSON structure:

```json
{
  "method": "tools/call",
  "id": "unique-test-id",
  "params": {
    "name": "tool_name",
    "arguments": {
      "parameter1": "value1",
      "parameter2": "value2"
    }
  }
}
```

## Safety Notes

- Always use mock mode (`P4_MOCK_MODE=1`) when testing file modification commands
- The `p4_info` command is safe to run against real servers as it's read-only
- Be cautious with commands like `p4_edit`, `p4_add`, `p4_submit` in real mode as they modify your workspace
- Test files use example paths like `//depot/main/...` - adjust for your actual depot structure

## Exit Codes

The server will exit with code 1 after processing stdin input, which is normal behavior for MCP servers.