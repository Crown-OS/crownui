use serde::Serialize;

/// One step's outcome.
#[derive(Debug, Serialize)]
pub(crate) struct Step {
    pub(crate) name: String,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

/// A command's outcome, printed as text or JSON.
#[derive(Debug, Serialize)]
pub(crate) struct Report {
    pub(crate) command: &'static str,
    pub(crate) ok: bool,
    pub(crate) steps: Vec<Step>,
}

impl Report {
    pub(crate) fn new(command: &'static str, steps: Vec<Step>) -> Self {
        Self {
            command,
            ok: steps.iter().all(|step| step.ok),
            steps,
        }
    }

    pub(crate) fn print(&self, json: bool) {
        if json {
            match serde_json::to_string_pretty(self) {
                Ok(text) => println!("{text}"),
                Err(error) => eprintln!("could not serialize report: {error}"),
            }
            return;
        }
        for step in &self.steps {
            let mark = if step.ok { "ok  " } else { "FAIL" };
            println!("[{mark}] {}", step.name);
            let show_detail = !step.ok || self.command == "preview";
            if show_detail && !step.detail.is_empty() {
                let limit = if self.command == "preview" {
                    usize::MAX
                } else {
                    40
                };
                for line in step.detail.lines().take(limit) {
                    println!("       {line}");
                }
            }
        }
        println!(
            "{}: {}",
            self.command,
            if self.ok { "passed" } else { "failed" }
        );
    }
}
