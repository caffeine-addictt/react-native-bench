/// Creates one or more `Cmd` instances for shell-like command execution.
///
/// This macro allows defining commands in a syntax reminiscent of a shell file, including:
/// - Single commands with optional arguments
/// - Multiple commands separated by semicolons
/// - Optional current working directory via `cwd: PATH =>`
/// - Optional environment variables via `env: KEY=VALUE =>`
///
/// # Features
/// - Single command returns a `Cmd` instance.
/// - Multiple commands return a `Vec<Cmd>`. To run them conveniently, import the `Cmds` trait:
///   ```rust
///   use crate::cmds::Cmds;
///   let outputs = make_cmd!("git" "status"; "cargo" "build").run()?;
///   ```
/// - Arguments are space-separated literals:
///   ```rust
///   make_cmd!("git" "status" "a");
///   ```
/// - Optional `cwd` sets the working directory for the command(s):
///   ```rust
///   make_cmd!(cwd: "./app" => "git" "status");
///   ```
/// - Optional `env` sets environment variables for the command(s):
///   ```rust
///   make_cmd!(env: RUST_LOG=debug => "cargo" "run");
///   ```
///
/// # Parameters
/// - `$cmd`: The command to run, as a literal string (e.g., `"git"`).
/// - `$args`: Zero or more literal strings representing command arguments.
/// - Multiple commands are separated by semicolons `;`.
///
/// # Example
/// ```rust
/// // Single command
/// let cmd = make_cmd!("git" "status");
///
/// // Multiple commands
/// use crate::cmds::{CmdOutputs, Cmds};
/// let cmds = make_cmd!("git" "status"; "cargo" "build");
/// let outputs = cmds.run()?; // Cmds trait needed
/// ```
#[macro_export]
macro_rules! make_cmd {
    // "command", "args"* separated by ";"
    ($cmd:expr $(, $args:expr )* $(;)?) => {{
        $crate::cmds::Cmd::new($cmd)
            $(.arg($args))*
    }};

    // multiple of arm one
    ( $( $cmd:expr $(, $args:expr )* );+ $(;)?) => {{
        let mut cmds = Vec::new();
        $(
            cmds.push(make_cmd!($cmd $(, $args)*));
        )+
        cmds
    }};
}
