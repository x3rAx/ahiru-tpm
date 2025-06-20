use std::io;

use crate::plugin::Plugin;

pub struct ActionResult {
    pub plugin: Plugin,
    pub result: Result<(), io::Error>,
    pub stdout: String,
    pub stderr: String,
}

impl ActionResult {
    pub fn new(plugin: Plugin, output: (Result<(), io::Error>, String, String)) -> ActionResult {
        ActionResult {
            plugin,
            result: output.0,
            stdout: output.1,
            stderr: output.2,
        }
    }
}
