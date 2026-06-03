use anyhow::{Context, Result};
use clap::Subcommand;
use std::fs;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum DevtoolsCommand {
    /// Print protocol capabilities supported by the current CLI/runtime schema.
    Capabilities {
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Capture a single FDTP frame snapshot from a running development app.
    Snapshot {
        /// Test/devtools control port exposed by `fission run --devtools`.
        #[arg(long, default_value_t = 9876)]
        port: u16,
        /// Write the JSON snapshot to a file instead of stdout.
        #[arg(long)]
        output: Option<PathBuf>,
        /// Pretty-print JSON.
        #[arg(long)]
        pretty: bool,
    },
    /// Print the live widget tree from a running development app.
    Tree {
        /// Test/devtools control port exposed by `fission run --devtools`.
        #[arg(long, default_value_t = 9876)]
        port: u16,
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Print the latest frame performance sample from a running development app.
    Perf {
        /// Test/devtools control port exposed by `fission run --devtools`.
        #[arg(long, default_value_t = 9876)]
        port: u16,
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Print the environment variables used by `fission run --devtools`.
    Env {
        /// Test/devtools control port to include in the output.
        #[arg(long, default_value_t = 9876)]
        port: u16,
        /// Include the performance overlay flag.
        #[arg(long)]
        performance_overlay: bool,
    },
}

pub fn run(command: DevtoolsCommand) -> Result<()> {
    match command {
        DevtoolsCommand::Capabilities { json } => capabilities(json),
        DevtoolsCommand::Snapshot {
            port,
            output,
            pretty,
        } => snapshot(port, output, pretty),
        DevtoolsCommand::Tree { port, json } => tree(port, json),
        DevtoolsCommand::Perf { port, json } => perf(port, json),
        DevtoolsCommand::Env {
            port,
            performance_overlay,
        } => env(port, performance_overlay),
    }
}

fn capabilities(json: bool) -> Result<()> {
    let capabilities = fission_devtools_protocol::DevtoolsCapabilities::runtime_baseline();
    if json {
        println!("{}", serde_json::to_string_pretty(&capabilities)?);
    } else {
        println!(
            "Fission devtools protocol {}",
            fission_devtools_protocol::FDTP_SCHEMA_VERSION
        );
        println!("widget tree: {}", capabilities.widget_tree);
        println!("core ir: {}", capabilities.core_ir);
        println!("layout: {}", capabilities.layout);
        println!("semantics: {}", capabilities.semantics);
        println!("performance: {}", capabilities.performance);
        println!("screenshots: {}", capabilities.screenshots);
    }
    Ok(())
}

fn snapshot(port: u16, output: Option<PathBuf>, pretty: bool) -> Result<()> {
    let snapshot = read_snapshot(port)?;
    let json = if pretty {
        serde_json::to_string_pretty(&snapshot)?
    } else {
        serde_json::to_string(&snapshot)?
    };
    if let Some(path) = output {
        fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))?;
    } else {
        println!("{json}");
    }
    Ok(())
}

fn tree(port: u16, json: bool) -> Result<()> {
    let snapshot = read_snapshot(port)?;
    let Some(tree) = snapshot.widget_tree else {
        println!("No widget tree snapshot is available.");
        return Ok(());
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&tree)?);
        return Ok(());
    }
    if let Some(root) = tree.root {
        print_widget_node(&tree, root, 0);
    } else {
        println!("Widget tree is empty.");
    }
    Ok(())
}

fn perf(port: u16, json: bool) -> Result<()> {
    let snapshot = read_snapshot(port)?;
    let Some(perf) = snapshot.performance else {
        println!("No frame performance sample is available yet.");
        return Ok(());
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&perf)?);
        return Ok(());
    }
    let frame_interval = perf
        .frame_interval_ms
        .map(|ms| format_duration(ms))
        .unwrap_or_else(|| "--".to_string());
    let fps = perf
        .fps()
        .map(|fps| format!("{fps:.0} fps"))
        .unwrap_or_else(|| "fps --".to_string());
    println!(
        "frame {}: {fps}, interval {frame_interval}, render {}",
        perf.sequence,
        format_duration(perf.total_ms),
    );
    if let Some(renderer) = &perf.renderer {
        println!("renderer: {renderer}");
    }
    println!(
        "widgets: {}, core ir nodes: {}, layout nodes: {}",
        perf.widget_count, perf.core_node_count, perf.layout_node_count
    );
    if let Some((stage, ms)) = perf.slowest_known_stage() {
        println!("slowest stage: {stage} {ms:.2}ms");
    }
    Ok(())
}

fn format_duration(ms: f64) -> String {
    if ms.is_finite() && ms > 0.0 && ms < 1.0 {
        format!("{:.0}us", ms * 1000.0)
    } else {
        format!("{ms:.2}ms")
    }
}

fn read_snapshot(port: u16) -> Result<fission_devtools_protocol::DevtoolsFrameSnapshot> {
    let client = fission_test_driver::LiveTestClient::connect(port);
    client
        .wait_for_ready(5_000)
        .with_context(|| format!("failed to attach to devtools control port {port}"))?;
    client.get_devtools_snapshot()
}

fn print_widget_node(
    tree: &fission_devtools_protocol::WidgetTreeSnapshot,
    ordinal: u64,
    depth: usize,
) {
    let Some(node) = tree.nodes.get(ordinal as usize) else {
        return;
    };
    let indent = "  ".repeat(depth);
    let id = node
        .widget_id
        .as_ref()
        .map(|id| format!(" #{id}"))
        .unwrap_or_default();
    let label = node
        .debug_label
        .as_ref()
        .map(|label| format!(" {label:?}"))
        .unwrap_or_default();
    println!("{indent}{}{}{}", node.kind, id, label);
    for child in &node.children {
        print_widget_node(tree, *child, depth + 1);
    }
}

fn env(port: u16, performance_overlay: bool) -> Result<()> {
    println!("FISSION_DEVTOOLS=1");
    println!("FISSION_TEST_CONTROL_PORT={port}");
    if performance_overlay {
        println!("FISSION_DEVTOOLS_PERFORMANCE_OVERLAY=1");
    }
    Ok(())
}
