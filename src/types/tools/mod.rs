//! Object-oriented tool framework.
//!
//! Standardizes every tool's input and output and makes each execution
//! trivially benchmarkable. The design has three pieces:
//!
//! - [`ToolInput`]  — a single, uniform way to pass arguments to any tool
//!   (the text plus optional `action` / `find` / `replace` params).
//! - [`ToolOutput`] — a single, uniform way to read a tool's result, including
//!   an error flag and the wall-clock execution time.
//! - [`Tool`]       — the trait every tool implements. A tool only declares its
//!   slug/label and a pure `run`; the framework supplies `execute`, which times
//!   the run and wraps it in a [`ToolOutput`].
//!
//! [`ToolRegistry`] collects all tools and lets callers dispatch by slug and
//! benchmark any tool (or the whole catalog) with a single call.

use std::time::Instant;

// ---------------------------------------------------------------------------
// Standardized input
// ---------------------------------------------------------------------------

/// Uniform input accepted by every tool.
///
/// Most tools only read [`ToolInput::input`]. The optional fields exist so the
/// parameterised tools (`case`, `replace`, `replace-all`, `lorem`) can receive
/// their extra arguments through the same channel, keeping the trait signature
/// small and the dispatcher uniform.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolInput {
    /// The primary text to transform.
    pub input: String,
    /// Optional action (e.g. `case` -> camel/snake/kebab/pascal).
    pub action: Option<String>,
    /// Optional find string (find-and-replace tools).
    pub find: Option<String>,
    /// Optional replace string (find-and-replace tools).
    pub replace: Option<String>,
}

impl ToolInput {
    /// Build an input from just the text.
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            action: None,
            find: None,
            replace: None,
        }
    }

    /// Set the `action` parameter.
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Set the `find` parameter.
    pub fn with_find(mut self, find: impl Into<String>) -> Self {
        self.find = Some(find.into());
        self
    }

    /// Set the `replace` parameter.
    pub fn with_replace(mut self, replace: impl Into<String>) -> Self {
        self.replace = Some(replace.into());
        self
    }

    /// Convenience: `find` + `replace` together.
    pub fn with_find_replace(mut self, find: impl Into<String>, replace: impl Into<String>) -> Self {
        self.find = Some(find.into());
        self.replace = Some(replace.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Standardized output
// ---------------------------------------------------------------------------

/// Uniform output produced by every tool execution.
///
/// Carries the result, an optional error, the slug of the tool that ran, and
/// the wall-clock execution time in seconds — so callers (routes, benchmarks,
/// tests) all read the same shape.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolOutput {
    /// The transformed result (empty when the tool errored).
    pub result: String,
    /// Error message when the tool failed, otherwise `None`.
    pub error: Option<String>,
    /// Wall-clock execution time in seconds.
    pub execution_time: f32,
    /// Slug of the tool that produced this output.
    pub tool: String,
}

impl ToolOutput {
    /// Build a successful output.
    pub fn ok(tool: impl Into<String>, result: impl Into<String>, execution_time: f32) -> Self {
        Self {
            result: result.into(),
            error: None,
            execution_time,
            tool: tool.into(),
        }
    }

    /// Build a failed output.
    pub fn err(tool: impl Into<String>, error: impl Into<String>, execution_time: f32) -> Self {
        Self {
            result: String::new(),
            error: Some(error.into()),
            execution_time,
            tool: tool.into(),
        }
    }

    /// Whether the tool completed without error.
    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }

    /// Whether the tool failed.
    pub fn is_err(&self) -> bool {
        self.error.is_some()
    }
}

// ---------------------------------------------------------------------------
// The Tool trait
// ---------------------------------------------------------------------------

/// The interface every tool implements.
///
/// Implementors only provide a slug, a label, and a pure, deterministic
/// [`Tool::run`]. The framework's default [`Tool::execute`] times the run and
/// wraps it in a [`ToolOutput`], so benchmarking comes for free.
///
/// `Send + Sync` is required so a [`ToolRegistry`] can be shared across
/// threads (e.g. stored in a `static` behind a `OnceLock`).
pub trait Tool: Send + Sync {
    /// Unique slug used in URLs and dispatch (e.g. `"uppercase"`).
    fn slug(&self) -> &str;

    /// Human-readable label for the sidebar / catalog.
    fn label(&self) -> &str;

    /// Perform the transform. Must be pure and deterministic: no I/O, no
    /// network, no global state. Returns the transformed text.
    fn run(&self, input: &ToolInput) -> String;

    /// Execute the tool and benchmark it.
    ///
    /// Times [`Tool::run`] and wraps the result in a [`ToolOutput`]. Override
    /// only if a tool needs custom error handling or timing semantics.
    fn execute(&self, input: &ToolInput) -> ToolOutput {
        let start = Instant::now();
        let result = self.run(input);
        let execution_time = start.elapsed().as_secs_f32();
        ToolOutput::ok(self.slug(), result, execution_time)
    }
}

// ---------------------------------------------------------------------------
// Registry + benchmarking
// ---------------------------------------------------------------------------

/// A single benchmark sample for one tool execution.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolBenchmark {
    /// Slug of the tool that was benchmarked.
    pub tool: String,
    /// Number of executions sampled.
    pub iterations: u32,
    /// Total wall-clock time across all iterations (seconds).
    pub total_time: f32,
    /// Average wall-clock time per iteration (seconds).
    pub avg_time: f32,
}

impl ToolBenchmark {
    /// Average time per iteration in milliseconds.
    pub fn avg_ms(&self) -> f32 {
        self.avg_time * 1000.0
    }
}

/// Collects every tool and dispatches by slug.
///
/// Tools are registered once at startup. [`ToolRegistry::run`] looks up a tool
/// by slug and executes it against a [`ToolInput`], returning a [`ToolOutput`].
/// [`ToolRegistry::benchmark`] runs a tool many times and reports timing.
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// Register a tool. Later registrations with the same slug replace earlier
    /// ones.
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.retain(|t| t.slug() != tool.slug());
        self.tools.push(tool);
    }

    /// Look up a tool by slug.
    pub fn get(&self, slug: &str) -> Option<&dyn Tool> {
        self.tools.iter().find(|t| t.slug() == slug).map(|t| t.as_ref())
    }

    /// All registered slugs, in registration order.
    pub fn slugs(&self) -> Vec<&str> {
        self.tools.iter().map(|t| t.slug()).collect()
    }

    /// Number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Execute the tool for `slug` against `input`.
    ///
    /// Returns `None` when the slug is unknown (the caller decides how to
    /// surface that — e.g. a 404 or an "unknown tool" message).
    pub fn run(&self, slug: &str, input: &ToolInput) -> Option<ToolOutput> {
        self.get(slug).map(|tool| tool.execute(input))
    }

    /// Benchmark `slug` by running it `iterations` times against `input`.
    ///
    /// Returns `None` when the slug is unknown.
    pub fn benchmark(
        &self,
        slug: &str,
        input: &ToolInput,
        iterations: u32,
    ) -> Option<ToolBenchmark> {
        let tool = self.get(slug)?;
        let start = Instant::now();
        for _ in 0..iterations {
            tool.execute(input);
        }
        let total_time = start.elapsed().as_secs_f32();
        Some(ToolBenchmark {
            tool: slug.to_string(),
            iterations,
            total_time,
            avg_time: total_time / iterations as f32,
        })
    }

    /// Benchmark every registered tool once and return the samples.
    pub fn benchmark_all(&self, input: &ToolInput) -> Vec<ToolBenchmark> {
        self.tools
            .iter()
            .map(|tool| {
                let start = Instant::now();
                tool.execute(input);
                let total_time = start.elapsed().as_secs_f32();
                ToolBenchmark {
                    tool: tool.slug().to_string(),
                    iterations: 1,
                    total_time,
                    avg_time: total_time,
                }
            })
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trivial tool used to exercise the framework.
    struct Upper;

    impl Tool for Upper {
        fn slug(&self) -> &str {
            "uppercase"
        }
        fn label(&self) -> &str {
            "Uppercase"
        }
        fn run(&self, input: &ToolInput) -> String {
            input.input.to_uppercase()
        }
    }

    #[test]
    fn tool_input_builders() {
        let i = ToolInput::new("hi")
            .with_action("snake")
            .with_find_replace("a", "b");
        assert_eq!(i.input, "hi");
        assert_eq!(i.action.as_deref(), Some("snake"));
        assert_eq!(i.find.as_deref(), Some("a"));
        assert_eq!(i.replace.as_deref(), Some("b"));
    }

    #[test]
    fn tool_output_ok_and_err() {
        let ok = ToolOutput::ok("t", "res", 0.5);
        assert!(ok.is_ok());
        assert!(!ok.is_err());
        assert_eq!(ok.result, "res");
        assert_eq!(ok.execution_time, 0.5);

        let err = ToolOutput::err("t", "boom", 0.1);
        assert!(err.is_err());
        assert!(!err.is_ok());
        assert_eq!(err.error.as_deref(), Some("boom"));
    }

    #[test]
    fn execute_times_and_wraps() {
        let tool = Upper;
        let out = tool.execute(&ToolInput::new("hi"));
        assert_eq!(out.result, "HI");
        assert!(out.is_ok());
        assert_eq!(out.tool, "uppercase");
        assert!(out.execution_time >= 0.0);
    }

    #[test]
    fn registry_dispatch() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Upper));
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.slugs(), vec!["uppercase"]);

        let out = reg.run("uppercase", &ToolInput::new("hi")).unwrap();
        assert_eq!(out.result, "HI");

        assert!(reg.run("nope", &ToolInput::new("hi")).is_none());
    }

    #[test]
    fn registry_register_replaces_same_slug() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Upper));
        reg.register(Box::new(Upper));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn benchmark_reports_timing() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Upper));
        let b = reg
            .benchmark("uppercase", &ToolInput::new("hi"), 100)
            .unwrap();
        assert_eq!(b.tool, "uppercase");
        assert_eq!(b.iterations, 100);
        assert!(b.total_time >= 0.0);
        assert!(b.avg_time >= 0.0);
        assert!(b.avg_ms() >= 0.0);
        assert!(reg.benchmark("nope", &ToolInput::new("hi"), 1).is_none());
    }

    #[test]
    fn benchmark_all_covers_every_tool() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Upper));
        let samples = reg.benchmark_all(&ToolInput::new("hi"));
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].tool, "uppercase");
    }
}
