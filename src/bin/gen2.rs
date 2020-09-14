use command_run::Command;
use std::fs;
use std::path::Path;

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

fn indent<S: AsRef<str>>(line: S) -> String {
    format!("    {}", line.as_ref())
}

#[derive(Clone, Copy, Debug)]
enum ErrorType {
    Io,
    Anyhow,
}

impl ErrorType {
    fn all() -> Vec<ErrorType> {
        vec![ErrorType::Io, ErrorType::Anyhow]
    }

    fn as_str(&self) -> &str {
        match self {
            ErrorType::Io => "std::io::Error",
            ErrorType::Anyhow => "anyhow::Error",
        }
    }

    fn short_name(&self) -> &str {
        match self {
            ErrorType::Io => "io",
            ErrorType::Anyhow => "anyhow",
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
}

fn gen_program(error_type: ErrorType, operation: Operation) -> Program {
    let mut program = Program::default();

    // Add make_error function
    program.add_line(format!(
        "fn make_error() -> Result<(), {}> {{",
        error_type.as_str()
    ));
    let bad_path = "/this/file/does/not/exist";
    let io_error = format!("std::fs::remove_file(\"{}\")", bad_path);
    program.add_line(indent(match error_type {
        ErrorType::Io => io_error,
        ErrorType::Anyhow => format!("Ok({}?)", io_error),
    }));
    program.add_line("}");
    program.add_empty();

    if operation == Operation::Return {
        program.add_line(format!(
            "fn main() -> Result<(), {}> {{",
            error_type.as_str()
        ));
    } else {
        program.add_line("fn main() {");
    }
    program.add_line(indent(match operation {
        Operation::Debug => "eprintln!(\"{:?}\", make_error().unwrap_err())",
        Operation::Display => "eprintln!(\"{}\", make_error().unwrap_err())",
        Operation::Unwrap => "make_error().unwrap();",
        Operation::Expect => "make_error().expect(\"oh no\");",
        Operation::Return => "make_error()",
    }));
    program.add_line("}");
    program
}

fn main() -> Result<(), anyhow::Error> {
    for err_type in ErrorType::all() {
        for operation in Operation::all() {
            let prog = gen_program(err_type, operation);
            let file_name =
                format!("{}_{}.rs", err_type.short_name(), operation.as_str());
            let contents = prog.lines.join("\n");
            fs::write(Path::new("gen/src/bin").join(file_name), contents)?;
        }
    }

    Command::new("cargo").set_dir("gen").add_arg("fmt").run()?;
    Command::new("cargo")
        .set_dir("gen")
        .add_arg("build")
        .run()?;

    Ok(())
}
