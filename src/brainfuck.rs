use std::{
    fmt::{self, Display},
    fs::{read_to_string, remove_file, File},
    io::{self, Write},
    path::Path,
    process::Stdio,
};

const TEMP_FILEPATH: &str = "temp.rs";
const TEMP_EXECPATH: &str = "temp";

#[derive(Debug)]
pub enum RunError {
    CouldNotCompile,
    PermissionDenied,
    Runtime(String),
    InvalidPath(String),
    InvalidChar(usize, char),
}

impl From<io::Error> for RunError {
    fn from(e: io::Error) -> Self {
        RunError::InvalidPath(e.to_string())
    }
}

impl Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RunError as M;
        match self {
            M::CouldNotCompile => write!(f, "Could not compile the generated Rust code"),
            M::InvalidPath(s) => write!(f, "Invalid path: {}", s),
            M::InvalidChar(i, c) => write!(f, "Invalid character at index {}: {}", i, c),
            M::PermissionDenied => write!(f, "Permission denied"),
            M::Runtime(s) => write!(f, "Runtime error: {}", s),
        }
    }
}

fn to_bf(src: &str) -> Result<String, RunError> {
    let mut indent = 4;
    let mut out = String::new();
    out.push_str("use std::io::Read;\n");
    out.push_str("use std::io::Write;\n\n");
    out.push_str("fn main() {\n");
    out.push_str("    let mut sp = 0;\n");
    out.push_str("    let mut stack = vec![0u8; 30000];\n");

    for (i, c) in src.chars().enumerate() {
        if c == ']' {
            indent -= 4;
        }
        out.push_str(&" ".repeat(indent));
        if c == '[' {
            indent += 4;
        }
        out.push_str(match c {
            '>' => "sp += 1;",
            '<' => "sp -= 1;",
            '+' => "stack[sp] = stack[sp].wrapping_add(1);",
            '-' => "stack[sp] = stack[sp].wrapping_sub(1);",
            '.' => "print!(\"{}\", stack[sp] as char); std::io::stdout().flush().unwrap();",
            ',' => "stack[sp] = std::io::stdin().bytes().next().unwrap().unwrap();",
            '[' => "while stack[sp] != 0 {",
            ']' => "}",
            ' ' | '\n' | '\t' => continue,
            _ => return Err(RunError::InvalidChar(i, c)),
        });
        out.push('\n');
    }
    out.push_str("}\n");
    Ok(out)
}

pub fn make(srcpath: impl AsRef<Path>, outpath: impl AsRef<Path>) -> Result<(), RunError> {
    let src = read_to_string(srcpath)?;
    let out = to_bf(&src)?;
    let mut temp_file = File::create(TEMP_FILEPATH)?;
    temp_file.write_all(out.as_bytes())?;
    let status = std::process::Command::new("rustc")
        .arg(TEMP_FILEPATH)
        .arg("-o")
        .arg(outpath.as_ref())
        // .arg("-C")
        // .arg("prefer-dynamic")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if !status.success() {
        return Err(RunError::CouldNotCompile);
    }

    match remove_file(TEMP_FILEPATH) {
        Ok(_) => (),
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => (),
            io::ErrorKind::PermissionDenied => return Err(RunError::PermissionDenied),
            _ => return Err(RunError::InvalidPath(e.to_string())),
        },
    }

    Ok(())
}

pub fn run_file(srcpath: impl AsRef<Path>) -> Result<(), RunError> {
    make(srcpath, TEMP_EXECPATH)?;
    let exec_path = Path::new(".").join(TEMP_EXECPATH);
    let status = std::process::Command::new(exec_path).status()?;
    if !status.success() {
        return Err(RunError::Runtime(format!(
            "Process exited with status code {}",
            status.code().unwrap_or(-1)
        )));
    }
    match remove_file(TEMP_EXECPATH) {
        Ok(_) => (),
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => (),
            io::ErrorKind::PermissionDenied => return Err(RunError::PermissionDenied),
            _ => return Err(RunError::InvalidPath(e.to_string())),
        },
    }
    Ok(())
}
