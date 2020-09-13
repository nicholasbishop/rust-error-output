use anyhow::{Context, Error};
use command_run::Command;
use std::fs;
use std::path::Path;

#[derive(Default)]
struct Output {
    lines: Vec<String>,
}

impl Output {
    fn add_rust_block<S: AsRef<str>>(&mut self, contents: S) {
        self.add_code_block_impl(contents, "rust");
    }

    fn add_code_block<S: AsRef<str>>(&mut self, contents: S) {
        self.add_code_block_impl(contents, "");
    }

    fn add_code_block_impl<S: AsRef<str>>(&mut self, contents: S, mode: &str) {
        let block = "```";
        self.lines.push(block.to_string() + mode);
        self.lines
            .extend(contents.as_ref().lines().map(|line| line.to_string()));
        self.lines.push(block.into());
    }

    fn finalize(&self) -> String {
        self.lines.join("\n")
    }

    fn add_line(&mut self, line: &str) {
        self.lines.push(line.into());
    }
}

fn main() -> Result<(), Error> {
    let template = include_str!("../../template.md");

    let mut output = Output::default();
    for line in template.lines() {
        let prefix = "!!! ";
        if line.starts_with(prefix) {
            let file_name = &line[prefix.len()..];
            let path = Path::new("src/bin").join(file_name);

            let contents = fs::read_to_string(path).context(line)?;
            output.add_rust_block(contents);

            let mut cmd = Command::new(
                Path::new("target/debug").join(file_name.replace(".rs", "")),
            );
            cmd.check = false;
            cmd.capture = true;
            cmd.log_command = false;
            let cmdout = cmd.run().unwrap();
            if !cmdout.stdout.is_empty() {
                panic!("unexpected stdout from {}", file_name);
            }

            output.add_code_block(cmdout.stderr_string_lossy());
        } else {
            output.add_line(line);
        }
    }

    fs::write("docs/index.md", output.finalize())?;

    Ok(())
}
