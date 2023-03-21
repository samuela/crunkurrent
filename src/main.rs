use clap::Parser;
use colored::Color::{self, TrueColor};
use colored::Colorize;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

/// Run programs concurrently.
///
/// For example,
///
///    $ crunkurrent --cmd "npm run dev" --cmd "cd api && flask run"
///
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
  /// Commands to run, use multiple times for concurrent processes
  #[arg(long, required = true)]
  cmd: Vec<String>,
}

// https://coolors.co/ff595e-ff924c-ffca3a-c5ca30-8ac926-52a675-1982c4-4267ac-6a4c93
static COLORS: &'static [Color] = &[
  TrueColor {
    r: 106,
    g: 76,
    b: 147,
  },
  TrueColor {
    r: 138,
    g: 201,
    b: 38,
  },
  TrueColor {
    r: 255,
    g: 202,
    b: 58,
  },
  TrueColor {
    r: 25,
    g: 130,
    b: 196,
  },
  TrueColor {
    r: 255,
    g: 89,
    b: 84,
  },
  TrueColor {
    r: 82,
    g: 166,
    b: 117,
  },
  TrueColor {
    r: 197,
    g: 202,
    b: 48,
  },
  TrueColor {
    r: 255,
    g: 146,
    b: 67,
  },
  TrueColor {
    r: 66,
    g: 103,
    b: 172,
  },
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args = Args::parse();

  let (stdout_tx, mut stdout_rx) = mpsc::unbounded_channel();
  let (stderr_tx, mut stderr_rx) = mpsc::unbounded_channel();
  let (waits_tx, mut waits_rx) = mpsc::unbounded_channel();

  // See https://github.com/samuela/crunkurrent/issues/1.
  let running = Arc::new(AtomicBool::new(true));
  let running_ = running.clone();
  ctrlc::set_handler(move || {
    eprintln!("Sending kill signal to subprocesses");
    running_.store(false, Ordering::SeqCst);
  })
  .expect("Error setting Ctrl-C handler");

  for (i, cmd) in args.cmd.iter().enumerate() {
    let stdout_tx_ = stdout_tx.clone();
    let stderr_tx_ = stderr_tx.clone();
    let waits_tx_ = waits_tx.clone();
    let cmd_ = cmd.clone();
    let running_ = running.clone();
    tokio::spawn(async move {
      let mut child: Child = Command::new("sh")
        .arg("-c")
        .arg(&cmd_)
        .stdin(Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn");
      let pid: u32 = child.id().expect("could not get child pid");

      // Pick color based on the command order instead of the pid so that colors are consistent across invocations.
      let pid_with_color = pid.to_string().color(COLORS[i % COLORS.len()]);

      eprintln!("{:<8} ├ [started '{}']", &pid_with_color, cmd_);

      let mut stdout_reader = BufReader::new(
        child
          .stdout
          .take()
          .expect("child did not have a handle to stdout"),
      )
      .lines();
      let mut stderr_reader = BufReader::new(
        child
          .stderr
          .take()
          .expect("child did not have a handle to stderr"),
      )
      .lines();

      loop {
        if !running_.load(Ordering::SeqCst) {
          child.kill().await.expect("failed to kill child process");
          // We don't `break` here; instead we let `child.wait()` resolve in the `tokio::select!` below and proceed as
          // usual from there.
        }
        tokio::select! {
          // Always take stdout/stderr before the exit status.
          biased;

          Ok(Some(line)) = stdout_reader.next_line() => { stdout_tx_.send((pid_with_color.clone(), line)).expect("failed to send"); }
          Ok(Some(line)) = stderr_reader.next_line() => { stderr_tx_.send((pid_with_color.clone(), line)).expect("failed to send"); }
          Ok(exitcode) = child.wait() => { waits_tx_.send((pid_with_color.clone(), exitcode)).expect("failed to send"); break; }
        }
      }
    });
  }

  let mut max_status = 0;
  let mut finished = 0;
  loop {
    tokio::select! {
      // Always take stdout/stderr before the exit status.
      biased;

      Some((pid, line)) = stdout_rx.recv() => { println!("{:<8} │ {}", pid, line); }
      Some((pid, line)) = stderr_rx.recv() => { eprintln!("{:<8} │ {}", pid, line); }
      Some((pid, exitcode)) = waits_rx.recv() => {
        if let Some(code) = exitcode.code() {
          eprintln!("{:<8} ├ [exited with status {}]", pid, code);
          max_status = std::cmp::max(max_status, code);
        } else {
          eprintln!("{:<8} ├ [exited with {}]", pid, exitcode);
          // Note: we don't update max_status here since, technically speaking, we don't have an exit code.
        }
        finished += 1;
        if finished == args.cmd.len() { break; }
      }
    }
  }

  std::process::exit(max_status);
}
