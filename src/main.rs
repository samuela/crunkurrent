use clap::Parser;
use colored::Color::{self, TrueColor};
use colored::Colorize;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
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
    r: 255,
    g: 89,
    b: 84,
  },
  TrueColor {
    r: 255,
    g: 146,
    b: 67,
  },
  TrueColor {
    r: 255,
    g: 202,
    b: 58,
  },
  TrueColor {
    r: 197,
    g: 202,
    b: 48,
  },
  TrueColor {
    r: 138,
    g: 201,
    b: 38,
  },
  TrueColor {
    r: 82,
    g: 166,
    b: 117,
  },
  TrueColor {
    r: 25,
    g: 130,
    b: 196,
  },
  TrueColor {
    r: 66,
    g: 103,
    b: 172,
  },
  TrueColor {
    r: 106,
    g: 76,
    b: 147,
  },
];

fn calculate_hash<T: Hash>(t: &T) -> u64 {
  let mut s = DefaultHasher::new();
  t.hash(&mut s);
  s.finish()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args = Args::parse();

  let (stdout_tx, mut stdout_rx) = mpsc::unbounded_channel();
  let (stderr_tx, mut stderr_rx) = mpsc::unbounded_channel();
  let (waits_tx, mut waits_rx) = mpsc::unbounded_channel();

  for cmd in &args.cmd {
    let stdout_tx_ = stdout_tx.clone();
    let stderr_tx_ = stderr_tx.clone();
    let waits_tx_ = waits_tx.clone();
    let cmd_ = cmd.clone();
    tokio::spawn(async move {
      let mut child: Child = Command::new("sh")
        .arg("-c")
        .arg(&cmd_)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn");
      let pid: u32 = child.id().expect("could not get child pid");

      // Pick color based on the command instead of the pid so that colors are consistent across invocations.
      let pid_with_color = pid
        .to_string()
        .color(COLORS[(calculate_hash(&cmd_) as usize) % COLORS.len()]);

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
        let code = exitcode.code().expect("failed to get exit code");
        eprintln!("{:<8} ├ [exited with status {}]", pid, code);
        finished += 1;
        max_status = std::cmp::max(max_status, code);
        if finished == args.cmd.len() { break; }
      }
    }
  }

  std::process::exit(max_status);
}
