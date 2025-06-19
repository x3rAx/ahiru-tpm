use std::time::Duration;

use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub struct ProgressStatus {
    multi_progress: MultiProgress,
}

impl Default for ProgressStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressStatus {
    pub fn new() -> ProgressStatus {
        let multi_progress = MultiProgress::new();

        ProgressStatus { multi_progress }
    }

    pub fn add_task(&self, task_name: &str, initial_status: &str) -> Result<ProgressTask> {
        let progress_bar = self.multi_progress.add(ProgressBar::new_spinner());
        let progress_task = ProgressTask::new(progress_bar, task_name.to_string());
        progress_task.set_status(initial_status)?;
        Ok(progress_task)
    }
}

pub struct ProgressTask {
    task_name: String,
    progress_bar: ProgressBar,
}

impl ProgressTask {
    pub fn new(progress_bar: ProgressBar, task_name: String) -> ProgressTask {
        ProgressTask {
            task_name,
            progress_bar,
        }
    }

    pub fn set_status(&self, status: &str) -> Result<()> {
        self.progress_bar
            .set_message(format!("{:<64} {}", self.task_name, status.bold().cyan()));
        self.progress_bar
            .enable_steady_tick(Duration::from_millis(100));
        self.progress_bar.set_style(
            ProgressStyle::with_template("{spinner:.cyan} {msg}")
                .context("Failed to set progress style")?,
        );
        Ok(())
    }

    pub fn set_success(&self, status: &str) -> Result<()> {
        self.progress_bar.set_style(
            ProgressStyle::with_template(&format!("{} {}", "✔".bold().green(), "{msg}"))
                .context("Failed to set progress style")?,
        );
        self.progress_bar.finish_with_message(format!(
            "{:<64} {}",
            self.task_name,
            status.bold().green()
        ));
        Ok(())
    }

    pub fn set_failed(&self, status: &str) -> Result<()> {
        self.progress_bar.set_style(
            ProgressStyle::with_template(&format!("{} {}", "✘".bold().red(), "{msg}"))
                .context("Failed to set progress style")?,
        );
        self.progress_bar.finish_with_message(format!(
            "{:<64} {}",
            self.task_name,
            status.bold().red()
        ));
        Ok(())
    }
}
