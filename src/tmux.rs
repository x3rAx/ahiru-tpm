use cmd_lib::run_fun;

pub fn get_option(name: &str) -> Option<String> {
    run_fun!(tmux show-option -vg $name).ok()
}
