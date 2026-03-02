//! MCP server entry point for AgenticPlanning.
//!
//! Runs the MCP server over stdin/stdout using Content-Length framed JSON-RPC 2.0.

use std::io::{BufRead, BufReader, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use agentic_planning::PlanningEngine;
use agentic_planning_mcp::{ghost_bridge, PlanningMcpServer, MAX_CONTENT_LENGTH_BYTES};

fn main() {
    let engine = PlanningEngine::in_memory();
    let mut server = PlanningMcpServer::new(engine);

    // Ghost bridge: sync planning context to AI coding assistants.
    let mut ghost = ghost_bridge::GhostBridge::new();

    // Install signal handler for graceful shutdown.
    let shutdown = Arc::new(AtomicBool::new(false));
    {
        let flag = Arc::clone(&shutdown);
        if let Err(e) = ctrlc::set_handler(move || {
            flag.store(true, Ordering::SeqCst);
        }) {
            eprintln!("warning: could not install signal handler: {e}");
        }
    }

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();

    run_stdio_loop(
        &mut reader,
        &mut writer,
        &mut server,
        &shutdown,
        &mut ghost,
    );

    // Final ghost sync before exit.
    if let Some(ref mut g) = ghost {
        g.sync(server.engine());
    }

    // Graceful shutdown: persist engine state before exit.
    if let Err(e) = server.save() {
        eprintln!("warning: failed to save on shutdown: {e}");
    }
}

fn run_stdio_loop<R: BufRead + Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    server: &mut PlanningMcpServer,
    shutdown: &AtomicBool,
    ghost: &mut Option<ghost_bridge::GhostBridge>,
) {
    let mut line = String::new();
    let mut content_length: Option<usize> = None;

    loop {
        if shutdown.load(Ordering::SeqCst) {
            break;
        }

        line.clear();
        let bytes = match reader.read_line(&mut line) {
            Ok(n) => n,
            Err(_) => break,
        };
        if bytes == 0 {
            break; // EOF
        }

        let trimmed = line.trim_end_matches(['\r', '\n']);

        // Support Content-Length framed messages.
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("content-length:") {
            let rest = trimmed.split_once(':').map(|(_, rhs)| rhs).unwrap_or("");
            match rest.trim().parse::<usize>() {
                Ok(n) if n <= MAX_CONTENT_LENGTH_BYTES => content_length = Some(n),
                Ok(n) => {
                    eprintln!(
                        "Content-Length {n} exceeds max frame size of {MAX_CONTENT_LENGTH_BYTES} bytes"
                    );
                    break;
                }
                Err(_) => {
                    eprintln!("Invalid Content-Length header: {trimmed}");
                    content_length = None;
                }
            }
            continue;
        }

        if let Some(n) = content_length {
            // Skip optional header separator line.
            if trimmed.is_empty() {
                let mut buf = vec![0u8; n];
                if reader.read_exact(&mut buf).is_err() {
                    break;
                }
                let raw = String::from_utf8_lossy(&buf).to_string();
                let response = server.handle_raw(raw.trim());
                if !response.is_empty() && write_framed(writer, &response).is_err() {
                    break;
                }
                // Ghost sync after each request.
                if let Some(ref mut g) = ghost {
                    g.sync(server.engine());
                }
                content_length = None;
                continue;
            }
            // Ignore extra header lines (e.g. Content-Type).
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        // Bare JSON line (no Content-Length framing).
        let response = server.handle_raw(trimmed);
        if response.is_empty() {
            continue;
        }
        if writeln!(writer, "{}", response).is_err() {
            break;
        }
        if writer.flush().is_err() {
            break;
        }
        // Ghost sync after each request.
        if let Some(ref mut g) = ghost {
            g.sync(server.engine());
        }
    }
}

fn write_framed<W: Write>(writer: &mut W, response: &str) -> std::io::Result<()> {
    let len = response.len();
    write!(writer, "Content-Length: {}\r\n\r\n{}", len, response)?;
    writer.flush()
}
