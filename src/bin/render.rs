use anyhow::{anyhow, Error};
use askama::Template;
use command_run::Command;
use std::path::{Path, PathBuf};
use std::{env, fs};
use syntect::highlighting::{Color, ThemeSet};
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

#[derive(Default)]
struct Program {
    lines: Vec<String>,
}

impl Program {
    fn add_line<S: AsRef<str>>(&mut self, line: S) {
        self.lines.push(line.as_ref().into());
    }

    fn add_empty(&mut self) {
        self.lines.push("".into());
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ErrorType {
    Io,
    Anyhow,
    AnyhowContext,
    AnyhowContext2x,
    StableEyre,
    ThisError,
}

impl ErrorType {
    fn all() -> Vec<ErrorType> {
        vec![
            ErrorType::Io,
            ErrorType::Anyhow,
            ErrorType::AnyhowContext,
            ErrorType::AnyhowContext2x,
            ErrorType::StableEyre,
            ErrorType::ThisError,
        ]
    }

    fn full_type_path(&self) -> &str {
        match self {
            ErrorType::Io => "std::io::Error",
            ErrorType::Anyhow
            | ErrorType::AnyhowContext
            | ErrorType::AnyhowContext2x => "anyhow::Error",
            ErrorType::StableEyre => "stable_eyre::eyre::Report",
            ErrorType::ThisError => "CustomError",
        }
    }

    fn short_name(&self) -> &str {
        match self {
            ErrorType::Io => "io",
            ErrorType::Anyhow => "anyhow",
            ErrorType::AnyhowContext => "anyhow_context",
            ErrorType::AnyhowContext2x => "anyhow_context_2x",
            ErrorType::StableEyre => "stable_eyre",
            ErrorType::ThisError => "thiserror",
        }
    }

    fn as_title(&self) -> &str {
        match self {
            ErrorType::Io => "std::io::Error",
            ErrorType::Anyhow => "anyhow",
            ErrorType::AnyhowContext => "anyhow with context",
            ErrorType::AnyhowContext2x => "anyhow with context 2x",
            ErrorType::StableEyre => "stable_eyre",
            ErrorType::ThisError => "thiserror",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Operation {
    Debug,
    Display,
    Unwrap,
    Expect,
    Return,
}

impl Operation {
    fn all() -> Vec<Operation> {
        vec![
            Operation::Debug,
            Operation::Display,
            Operation::Unwrap,
            Operation::Expect,
            Operation::Return,
        ]
    }

    fn as_str(&self) -> &str {
        match self {
            Operation::Debug => "debug",
            Operation::Display => "display",
            Operation::Unwrap => "unwrap",
            Operation::Expect => "expect",
            Operation::Return => "return",
        }
    }

    fn as_title(&self) -> &str {
        match self {
            Operation::Debug => "Debug",
            Operation::Display => "Display",
            Operation::Unwrap => "Unwrap",
            Operation::Expect => "Expect",
            Operation::Return => "Return",
        }
    }
}

fn gen_program(error_type: ErrorType, operation: Operation) -> Program {
    let mut program = Program::default();

    if matches!(
        error_type,
        ErrorType::AnyhowContext | ErrorType::AnyhowContext2x
    ) {
        program.add_line("use anyhow::Context;");
        program.add_empty();
    } else if error_type == ErrorType::ThisError {
        program.add_line("#[derive(Debug, thiserror::Error)]");
        program.add_line("enum CustomError {");
        program.add_line("#[error(\"bad thing: {0}\")]");
        program.add_line("BadThing(#[from] std::io::Error)");
        program.add_line("}");
        program.add_empty();
    }

    // Add make_error function
    program.add_line(format!(
        "fn make_error() -> Result<(), {}> {{",
        error_type.full_type_path()
    ));
    let bad_path = "/this/file/does/not/exist";
    let io_error = format!("std::fs::remove_file(\"{}\")", bad_path);
    program.add_line(match error_type {
        ErrorType::Io => io_error,
        ErrorType::AnyhowContext => {
            format!("Ok({}.context(\"oh no\")?)", io_error)
        }
        ErrorType::AnyhowContext2x => format!(
            "Ok({}.context(\"oh no\").context(\"second context\")?)",
            io_error
        ),
        ErrorType::Anyhow | ErrorType::StableEyre | ErrorType::ThisError => {
            format!("Ok({}?)", io_error)
        }
    });
    program.add_line("}");
    program.add_empty();

    // Add install_hook function
    if error_type == ErrorType::StableEyre {
        program.add_line(
            "fn install_hook() -> stable_eyre::eyre::Result<()> {
            let hook = stable_eyre::HookBuilder::default()
                 .capture_backtrace_by_default(true);
            hook.install()
        }",
        );
    }

    if operation == Operation::Return {
        program.add_line(format!(
            "fn main() -> Result<(), {}> {{",
            error_type.full_type_path()
        ));
    } else {
        program.add_line("fn main() {");
    }

    if error_type == ErrorType::StableEyre {
        let handle_err = if operation == Operation::Return {
            "?"
        } else {
            ".unwrap()"
        };
        program.add_line(format!("install_hook(){};", handle_err));
    }

    program.add_line(match operation {
        Operation::Debug => "eprintln!(\"{:?}\", make_error().unwrap_err())",
        Operation::Display => "eprintln!(\"{}\", make_error().unwrap_err())",
        Operation::Unwrap => "make_error().unwrap();",
        Operation::Expect => "make_error().expect(\"oh no\");",
        Operation::Return => "make_error()",
    });
    program.add_line("}");
    program
}

fn get_source_file_name(error_type: ErrorType, operation: Operation) -> String {
    format!("{}_{}.rs", error_type.short_name(), operation.as_str())
}

fn get_source_path(error_type: ErrorType, operation: Operation) -> PathBuf {
    Path::new("gen/src/bin").join(get_source_file_name(error_type, operation))
}

/// Normalize eyre backtrace output. This makes the output match the
/// github CI.
fn normalize_backtrace_paths(stderr: String) -> String {
    let runner_home = "/home/runner";

    // First normalize the source paths
    let stderr = stderr.replace(
        env!("CARGO_MANIFEST_DIR"),
        &(runner_home.to_owned() + "/work/rust-error-output/rust-error-output"),
    );

    // Then normalize the cargo paths
    let home = env::var("HOME").expect("HOME is not set");
    stderr.replace(&home, runner_home)
}

struct SourceAndOutput {
    initial: String,
    rest: String,
    output: String,
}

impl SourceAndOutput {
    fn new(path: &Path) -> Result<SourceAndOutput, Error> {
        let src = fs::read_to_string(path)?;
        let initial;
        let rest;
        if let Some(index) = src.find("fn main") {
            initial = &src[..index];
            rest = &src[index..];
        } else {
            return Err(anyhow!("missing fn main"));
        }

        let file_name = path.file_name().unwrap().to_str().unwrap();
        let mut cmd =
            Command::new(Path::new(".").join(file_name.replace(".rs", "")));
        cmd.set_dir("gen/target/debug");
        cmd.check = false;
        cmd.capture = true;
        cmd.log_command = false;
        let cmdout = cmd.run().unwrap();
        if !cmdout.stdout.is_empty() {
            panic!("unexpected stdout from {}", file_name);
        }
        let stderr = cmdout.stderr_string_lossy();

        let stderr = normalize_backtrace_paths(stderr.into());

        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let mut theme = ts.themes["InspiredGitHub"].clone();

        theme.settings.background = Some(Color {
            r: 243,
            g: 246,
            b: 250,
            a: 255,
        });

        let initial = highlighted_html_for_string(
            initial,
            &ss,
            ss.find_syntax_by_extension("rs").unwrap(),
            &theme,
        );

        let rest = highlighted_html_for_string(
            rest,
            &ss,
            ss.find_syntax_by_extension("rs").unwrap(),
            &theme,
        );

        let output = format!("<pre>{}</pre>", stderr);

        Ok(SourceAndOutput {
            initial,
            rest,
            output,
        })
    }
}

fn gen_nav(current: &str) -> String {
    let link = |text: &str, target: &str| -> String {
        let class = if target == current {
            "class=\"active\""
        } else {
            ""
        };
        format!(
            "<li><a {} href=\"{}.html\">{}</a></li>",
            class, target, text
        )
    };

    let mut nav = "<ul>".to_string();
    for error_type in ErrorType::all() {
        nav += &link(error_type.as_title(), error_type.short_name());
    }
    nav += &link("panic", "panic");
    nav += "</ul>";
    nav
}

fn get_rust_version() -> Result<String, Error> {
    let mut cmd = Command::new("rustc");
    cmd.add_arg("--version");
    cmd.capture = true;
    cmd.log_command = false;
    let out = cmd.run()?;

    // Output looks like this: "cargo 1.46.0 (149022b1d 2020-07-17)"
    let full = out.stdout_string_lossy();

    // Return the middle part, e.g. "1.46.0"
    if let Some(version) = full.split_whitespace().nth(1) {
        Ok(version.into())
    } else {
        Err(anyhow!("invalid version output"))
    }
}

#[derive(Template)]
#[template(path = "error.html", escape = "none")]
struct ErrorTemplate {
    error_name: String,
    nav: String,
    content: String,
    version: String,
}

impl ErrorTemplate {
    fn write(&self, name: &str) -> Result<(), Error> {
        let path = Path::new("docs").join(format!("{}.html", name));
        fs::write(path, self.render()?)?;
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let version = get_rust_version()?;

    for err_type in ErrorType::all() {
        for operation in Operation::all() {
            let prog = gen_program(err_type, operation);
            let path = get_source_path(err_type, operation);
            let contents = prog.lines.join("\n");
            fs::write(path, contents)?;
        }
    }

    let panic_path = Path::new("gen/src/bin/panic.rs");
    fs::write(
        panic_path,
        "fn main() {
    panic!(\"oh no\");
}",
    )?;

    Command::new("cargo").set_dir("gen").add_arg("fmt").run()?;
    Command::new("cargo")
        .set_dir("gen")
        .add_arg("build")
        .run()?;

    for error_type in ErrorType::all() {
        let mut content = String::new();
        for (index, operation) in Operation::all().iter().enumerate() {
            let path = get_source_path(error_type, *operation);
            let output = SourceAndOutput::new(&path)?;

            if index == 0 {
                content += &format!(
                    "<h2>Setup code for {}</h2>",
                    error_type.as_title()
                );
                content += &output.initial;
            }

            content += &format!("<h2>{}</h2>", operation.as_title());
            content += &output.rest;
            content += &output.output;
        }

        ErrorTemplate {
            nav: gen_nav(error_type.short_name()),
            error_name: error_type.short_name().into(),
            content,
            version: version.clone(),
        }
        .write(error_type.short_name())?;
    }

    // Create panic example
    let output = SourceAndOutput::new(panic_path)?;
    let content = format!("<h2>Panic</h2>{}{}", output.rest, output.output);
    ErrorTemplate {
        nav: gen_nav("panic"),
        error_name: "panic".into(),
        content,
        version,
    }
    .write("panic")?;

    Ok(())
}
