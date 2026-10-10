use super::command::LaunchCommand;
use super::log::{LogParser, ParsedLine};
use super::{LaunchEvent, LogStream};
use crate::error::AppResult;
use crate::java::hide_console;
use crate::state::RunRegistry;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

const PIPE_DRAIN_TIMEOUT: Duration = Duration::from_secs(2);

pub fn spawn(
    command: &LaunchCommand,
    run_id: u64,
    token: CancellationToken,
    runs: RunRegistry,
    emit: impl Fn(LaunchEvent) + Clone + Send + Sync + 'static,
) -> AppResult<()> {
    let mut process = Command::new(&command.program);
    process
        .args(&command.args)
        .current_dir(&command.working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    hide_console(&mut process);

    let mut child = process.spawn()?;
    emit(LaunchEvent::Running { pid: child.id() });

    let stdout = tokio::spawn(pump(child.stdout.take(), LogStream::Stdout, emit.clone()));
    let stderr = tokio::spawn(pump(child.stderr.take(), LogStream::Stderr, emit.clone()));

    tokio::spawn(async move {
        let status = tokio::select! {
            status = child.wait() => status,
            _ = token.cancelled() => {
                let _ = child.kill().await;
                child.wait().await
            }
        };
        let _ = tokio::time::timeout(PIPE_DRAIN_TIMEOUT, async {
            let _ = tokio::join!(stdout, stderr);
        })
        .await;
        runs.finish(run_id);
        emit(LaunchEvent::Exited {
            code: status.ok().and_then(|status| status.code()),
        });
    });
    Ok(())
}

async fn pump<R: AsyncRead + Unpin>(
    reader: Option<R>,
    stream: LogStream,
    emit: impl Fn(LaunchEvent),
) {
    let Some(reader) = reader else { return };
    let mut reader = BufReader::new(reader);
    let mut parser = LogParser::default();
    let mut buffer = Vec::new();
    let emit_lines = |lines: Vec<ParsedLine>| {
        for ParsedLine { level, text } in lines {
            emit(LaunchEvent::Log { stream, level, line: text });
        }
    };
    loop {
        buffer.clear();
        match reader.read_until(b'\n', &mut buffer).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let line = String::from_utf8_lossy(&buffer);
                emit_lines(parser.push(line.trim_end_matches(['\r', '\n'])));
            }
        }
    }
    emit_lines(parser.flush());
}
