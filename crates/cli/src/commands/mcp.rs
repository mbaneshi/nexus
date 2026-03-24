//! `nexus mcp` — Model Context Protocol server over stdio.
//!
//! Reads JSON-RPC messages from stdin, writes responses to stdout.
//! Implements the MCP protocol (initialize, tools/list, tools/call)
//! so Claude Code can interact with nexus directly.

use color_eyre::eyre;
use rusqlite::Connection;
use serde_json::{Value, json};
use std::io::{BufRead, Write};

/// Run the MCP server, reading JSON-RPC from stdin and writing responses to stdout.
pub fn run(conn: &Connection) -> eyre::Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let reader = stdin.lock();
    let mut writer = stdout.lock();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                let err_resp = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {e}")
                    }
                });
                writeln!(writer, "{}", serde_json::to_string(&err_resp)?)?;
                writer.flush()?;
                continue;
            }
        };

        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("");
        let params = request.get("params").cloned().unwrap_or(json!({}));

        let response = match method {
            "initialize" => handle_initialize(&id),
            "initialized" => {
                // Notification, no response needed
                continue;
            }
            "tools/list" => handle_tools_list(&id),
            "tools/call" => handle_tools_call(&id, &params, conn),
            "notifications/cancelled" | "notifications/initialized" => {
                // Notifications, no response needed
                continue;
            }
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {method}")
                }
            }),
        };

        writeln!(writer, "{}", serde_json::to_string(&response)?)?;
        writer.flush()?;
    }

    Ok(())
}

/// Handle the `initialize` method.
fn handle_initialize(id: &Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "nexus",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    })
}

/// Handle the `tools/list` method.
fn handle_tools_list(id: &Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": [
                {
                    "name": "nexus_search",
                    "description": "Search indexed files in the home directory via FTS5 full-text search.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query text"
                            },
                            "category": {
                                "type": "string",
                                "description": "Filter by category (config, code, document, image, video, audio, archive, cache, project, download, other)",
                                "enum": ["config", "code", "document", "image", "video", "audio", "archive", "cache", "project", "download", "other"]
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of results (default: 20)",
                                "default": 20
                            }
                        },
                        "required": ["query"]
                    }
                },
                {
                    "name": "nexus_stats",
                    "description": "Get home directory statistics including file counts, sizes, and category breakdown.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "nexus_config_list",
                    "description": "List all discovered config tools in ~/.config with file counts and sizes.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "nexus_config_show",
                    "description": "Show config files for a specific tool with paths, sizes, and languages.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "tool_name": {
                                "type": "string",
                                "description": "Tool name (e.g., nvim, git, fish, zsh)"
                            }
                        },
                        "required": ["tool_name"]
                    }
                },
                {
                    "name": "nexus_config_backup",
                    "description": "Backup a tool's config files as a snapshot. If no tool specified, backs up all tools.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "tool_name": {
                                "type": "string",
                                "description": "Tool name to backup (omit for all tools)"
                            }
                        }
                    }
                },
                {
                    "name": "nexus_changes",
                    "description": "Show recent filesystem changes detected by the nexus daemon.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of changes to return (default: 50)",
                                "default": 50
                            }
                        }
                    }
                }
            ]
        }
    })
}

/// Handle the `tools/call` method by dispatching to the appropriate tool handler.
fn handle_tools_call(id: &Value, params: &Value, conn: &Connection) -> Value {
    let tool_name = params
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("");
    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    let result = match tool_name {
        "nexus_search" => tool_search(&arguments, conn),
        "nexus_stats" => tool_stats(conn),
        "nexus_config_list" => tool_config_list(conn),
        "nexus_config_show" => tool_config_show(&arguments, conn),
        "nexus_config_backup" => tool_config_backup(&arguments, conn),
        "nexus_changes" => tool_changes(&arguments, conn),
        _ => Err(eyre::eyre!("Unknown tool: {tool_name}")),
    };

    match result {
        Ok(content) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": content
                    }
                ]
            }
        }),
        Err(e) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": format!("Error: {e}")
                    }
                ],
                "isError": true
            }
        }),
    }
}

/// Search indexed files via FTS5.
fn tool_search(args: &Value, conn: &Connection) -> eyre::Result<String> {
    let query = args
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or_else(|| eyre::eyre!("Missing required parameter: query"))?;
    let category = args.get("category").and_then(|c| c.as_str());
    let limit = args
        .get("limit")
        .and_then(|l| l.as_u64())
        .unwrap_or(20) as usize;

    let search_query = nexus_core::models::SearchQuery {
        text: query.to_string(),
        category: category.map(nexus_core::models::FileCategory::from_str_lossy),
        limit,
        ..Default::default()
    };

    let results = nexus_discovery::search(conn, &search_query)?;
    Ok(serde_json::to_string_pretty(&results)?)
}

/// Get home directory statistics.
fn tool_stats(conn: &Connection) -> eyre::Result<String> {
    let stats = nexus_discovery::home_stats(conn)?;
    Ok(serde_json::to_string_pretty(&stats)?)
}

/// List all discovered config tools.
fn tool_config_list(conn: &Connection) -> eyre::Result<String> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let tools = nexus_configs::discover_tools(&home)?;

    // Upsert tools into database (same pattern as CLI)
    for tool in &tools {
        conn.execute(
            "INSERT OR REPLACE INTO config_tools (name, config_dir, description, language)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                tool.name,
                tool.config_dir.to_string_lossy().as_ref(),
                tool.description,
                tool.language,
            ],
        )?;
    }

    Ok(serde_json::to_string_pretty(&tools)?)
}

/// Show config files for a specific tool.
fn tool_config_show(args: &Value, conn: &Connection) -> eyre::Result<String> {
    let tool_name = args
        .get("tool_name")
        .and_then(|t| t.as_str())
        .ok_or_else(|| eyre::eyre!("Missing required parameter: tool_name"))?;

    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let tools = nexus_configs::discover_tools(&home)?;

    let tool = tools
        .iter()
        .find(|t| t.name == tool_name)
        .ok_or_else(|| eyre::eyre!("Tool '{}' not found", tool_name))?;

    let tool_id: i64 = conn
        .query_row(
            "SELECT id FROM config_tools WHERE name = ?1",
            [tool_name],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let files = nexus_configs::scan_tool_files(tool_id, tool_name, &tool.config_dir)?;
    Ok(serde_json::to_string_pretty(&files)?)
}

/// Backup a tool's config (or all tools).
fn tool_config_backup(args: &Value, conn: &Connection) -> eyre::Result<String> {
    let tool_name = args.get("tool_name").and_then(|t| t.as_str());

    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let tools = nexus_configs::discover_tools(&home)?;

    // Upsert tools into database
    for tool in &tools {
        conn.execute(
            "INSERT OR REPLACE INTO config_tools (name, config_dir, description, language)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                tool.name,
                tool.config_dir.to_string_lossy().as_ref(),
                tool.description,
                tool.language,
            ],
        )?;
    }

    let targets: Vec<_> = if let Some(name) = tool_name {
        tools.into_iter().filter(|t| t.name == name).collect()
    } else {
        tools
    };

    if targets.is_empty() {
        return Ok(json!({"message": "No tools to backup"}).to_string());
    }

    let mut results = Vec::new();
    for tool in &targets {
        let tool_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM config_tools WHERE name = ?1",
                [&tool.name],
                |row| row.get(0),
            )
            .ok();

        let snap_id = nexus_configs::create_snapshot(conn, tool_id, None, &tool.config_dir)?;
        results.push(json!({
            "tool": tool.name,
            "snapshot_id": snap_id
        }));
    }

    Ok(serde_json::to_string_pretty(&results)?)
}

/// Show recent filesystem changes.
fn tool_changes(args: &Value, conn: &Connection) -> eyre::Result<String> {
    let limit = args
        .get("limit")
        .and_then(|l| l.as_u64())
        .unwrap_or(50) as usize;

    let changes = nexus_core::db::list_changes(conn, limit)?;
    Ok(serde_json::to_string_pretty(&changes)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Connection {
        nexus_core::db::open_in_memory().unwrap()
    }

    #[test]
    fn initialize_returns_server_info() {
        let id = json!(1);
        let response = handle_initialize(&id);
        assert_eq!(response["id"], 1);
        assert_eq!(response["result"]["serverInfo"]["name"], "nexus");
        assert!(response["result"]["capabilities"]["tools"].is_object());
    }

    #[test]
    fn tools_list_returns_all_tools() {
        let id = json!(2);
        let response = handle_tools_list(&id);
        let tools = response["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 6);

        let names: Vec<&str> = tools
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"nexus_search"));
        assert!(names.contains(&"nexus_stats"));
        assert!(names.contains(&"nexus_config_list"));
        assert!(names.contains(&"nexus_config_show"));
        assert!(names.contains(&"nexus_config_backup"));
        assert!(names.contains(&"nexus_changes"));
    }

    #[test]
    fn tools_list_has_valid_schemas() {
        let id = json!(3);
        let response = handle_tools_list(&id);
        let tools = response["result"]["tools"].as_array().unwrap();

        for tool in tools {
            assert!(tool["name"].is_string(), "tool must have a name");
            assert!(tool["description"].is_string(), "tool must have a description");
            assert!(tool["inputSchema"].is_object(), "tool must have an inputSchema");
            assert_eq!(
                tool["inputSchema"]["type"].as_str().unwrap(),
                "object",
                "inputSchema type must be object"
            );
        }
    }

    #[test]
    fn search_requires_query_param() {
        let conn = setup_db();
        let args = json!({});
        let result = tool_search(&args, &conn);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("query"));
    }

    #[test]
    fn search_with_empty_db_returns_empty() {
        let conn = setup_db();
        let args = json!({"query": "nonexistent"});
        let result = tool_search(&args, &conn).unwrap();
        let parsed: Vec<Value> = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn stats_with_empty_db() {
        let conn = setup_db();
        let result = tool_stats(&conn).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["total_files"], 0);
    }

    #[test]
    fn changes_with_empty_db() {
        let conn = setup_db();
        let args = json!({"limit": 10});
        let result = tool_changes(&args, &conn).unwrap();
        let parsed: Vec<Value> = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn changes_default_limit() {
        let conn = setup_db();
        let args = json!({});
        let result = tool_changes(&args, &conn).unwrap();
        let parsed: Vec<Value> = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn config_show_requires_tool_name() {
        let conn = setup_db();
        let args = json!({});
        let result = tool_config_show(&args, &conn);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("tool_name"));
    }

    #[test]
    fn unknown_tool_returns_error() {
        let conn = setup_db();
        let id = json!(10);
        let params = json!({"name": "nonexistent_tool", "arguments": {}});
        let response = handle_tools_call(&id, &params, &conn);
        assert!(response["result"]["isError"].as_bool().unwrap_or(false));
    }

    #[test]
    fn tools_call_dispatch_search() {
        let conn = setup_db();
        let id = json!(5);
        let params = json!({
            "name": "nexus_search",
            "arguments": {"query": "test"}
        });
        let response = handle_tools_call(&id, &params, &conn);
        assert_eq!(response["id"], 5);
        // Should succeed even with empty DB
        assert!(response["result"]["content"].is_array());
        assert!(!response["result"]["isError"].as_bool().unwrap_or(false));
    }

    #[test]
    fn tools_call_dispatch_stats() {
        let conn = setup_db();
        let id = json!(6);
        let params = json!({"name": "nexus_stats", "arguments": {}});
        let response = handle_tools_call(&id, &params, &conn);
        assert_eq!(response["id"], 6);
        assert!(response["result"]["content"].is_array());
    }

    #[test]
    fn tools_call_dispatch_changes() {
        let conn = setup_db();
        let id = json!(7);
        let params = json!({"name": "nexus_changes", "arguments": {"limit": 5}});
        let response = handle_tools_call(&id, &params, &conn);
        assert_eq!(response["id"], 7);
        assert!(response["result"]["content"].is_array());
    }
}
