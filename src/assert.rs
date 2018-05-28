use std::fmt;
use std::process;
use std::str;

use predicates;

use errors::output_fmt;

/// Extend `process::Output` with assertions.
///
/// # Examples
///
/// ```rust
/// use assert_cmd::*;
///
/// use std::process::Command;
///
/// Command::main_binary()
///     .unwrap()
///     .assert()
///     .success();
/// ```
pub trait OutputAssertExt {
    /// Wrap with an interface for that provides assertions on the `process::Output`.
    fn assert(self) -> Assert;
}

impl OutputAssertExt for process::Output {
    fn assert(self) -> Assert {
        Assert::new(self)
    }
}

impl<'c> OutputAssertExt for &'c mut process::Command {
    fn assert(self) -> Assert {
        let output = self.output().unwrap();
        Assert::new(output).set_cmd(format!("{:?}", self))
    }
}

/// `process::Output` assertions.
#[derive(Debug)]
pub struct Assert {
    output: process::Output,
    cmd: Option<String>,
    stdin: Option<Vec<u8>>,
}

impl Assert {
    /// Convert `std::process::Output` into a `Fail`.
    pub fn new(output: process::Output) -> Self {
        Self {
            output,
            cmd: None,
            stdin: None,
        }
    }

    /// Add the command line for additional context.
    pub fn set_cmd(mut self, cmd: String) -> Self {
        self.cmd = Some(cmd);
        self
    }

    /// Add the `stdn` for additional context.
    pub fn set_stdin(mut self, stdin: Vec<u8>) -> Self {
        self.stdin = Some(stdin);
        self
    }

    /// Access the contained `std::process::Output`.
    pub fn get_output(&self) -> &process::Output {
        &self.output
    }

    // How does user interact with assertion API?
    // - On Assert class, using error chaining
    //   - "Builder" or not?  If yes, then do we extend Result?
    //   - How do we give a helpful unwrap?
    // - Build up assertion data and "execute" it, like assert_cli used to?  But that was mostly
    //   from building up before executing the command happened.  Now we're doing it
    //   after-the-fact.
    // - Immediately panic in each assertion? Let's give that a try.

    /// Ensure the command succeeded.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use assert_cmd::*;
    ///
    /// use std::process::Command;
    ///
    /// Command::main_binary()
    ///     .unwrap()
    ///     .assert()
    ///     .success();
    /// ```
    pub fn success(self) -> Self {
        if !self.output.status.success() {
            panic!("Unexpected failure\n{}", self);
        }
        self
    }

    /// Ensure the command failed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use assert_cmd::*;
    ///
    /// use std::process::Command;
    ///
    /// Command::main_binary()
    ///     .unwrap()
    ///     .env("exit", "1")
    ///     .assert()
    ///     .failure();
    /// ```
    pub fn failure(self) -> Self {
        if self.output.status.success() {
            panic!("Unexpected success\n{}", self);
        }
        self
    }

    /// Ensure the command returned the expected code.
    pub fn interrupted(self) -> Self {
        if self.output.status.code().is_some() {
            panic!("Unexpected completion\n{}", self);
        }
        self
    }

    /// Ensure the command returned the expected code.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use assert_cmd::*;
    ///
    /// use std::process::Command;
    ///
    /// Command::main_binary()
    ///     .unwrap()
    ///     .env("exit", "42")
    ///     .assert()
    ///     .code(predicates::ord::eq(42));
    /// ```
    pub fn code(self, pred: &predicates::Predicate<i32>) -> Self {
        let actual_code = self.output
            .status
            .code()
            .unwrap_or_else(|| panic!("Command interrupted\n{}", self));
        if !pred.eval(&actual_code) {
            panic!("Unexpected return code\n{}", self);
        }
        self
    }

    /// Ensure the command wrote the expected data to `stdout`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use assert_cmd::*;
    ///
    /// use std::process::Command;
    ///
    /// Command::main_binary()
    ///     .unwrap()
    ///     .env("stdout", "hello")
    ///     .env("stderr", "world")
    ///     .assert()
    ///     .stdout(predicates::ord::eq(b"hello"));
    /// ```
    pub fn stdout(self, pred: &predicates::Predicate<Vec<u8>>) -> Self {
        {
            let actual = &self.output.stdout;
            if !pred.eval(actual) {
                panic!("Unexpected stdout\n{}", self);
            }
        }
        self
    }

    /// Ensure the command wrote the expected data to `stderr`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use assert_cmd::*;
    ///
    /// use std::process::Command;
    ///
    /// Command::main_binary()
    ///     .unwrap()
    ///     .env("stdout", "hello")
    ///     .env("stderr", "world")
    ///     .assert()
    ///     .stderr(predicates::ord::eq(b"world"));
    /// ```
    pub fn stderr(self, pred: &predicates::Predicate<Vec<u8>>) -> Self {
        {
            let actual = &self.output.stderr;
            if !pred.eval(actual) {
                panic!("Unexpected stderr\n{}", self);
            }
        }
        self
    }
}

impl fmt::Display for Assert {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref cmd) = self.cmd {
            writeln!(f, "command=`{}`", cmd)?;
        }
        if let Some(ref stdin) = self.stdin {
            if let Ok(stdin) = str::from_utf8(stdin) {
                writeln!(f, "stdin=```{}```", stdin)?;
            } else {
                writeln!(f, "stdin=```{:?}```", stdin)?;
            }
        }
        output_fmt(&self.output, f)
    }
}