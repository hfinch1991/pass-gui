use std::env;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/pass-gui-main.log")
    {
        let _ = writeln!(file, "[{}] STARTUP - Args: {:?}", chrono::Local::now(), args);
    }

    let mut is_native_mode = false;
    for arg in &args {
        let arg_lower = arg.to_lowercase();
        if arg_lower.contains("com.github.browserpass.native") || 
           arg_lower.contains("chrome-extension://") ||
           arg_lower == "--native-messaging" || 
           arg_lower == "show" || 
           arg_lower == "list" ||
           arg_lower.ends_with(".json") {
            is_native_mode = true;
            break;
        }
    }

    if is_native_mode {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/pass-gui-main.log")
        {
            let _ = writeln!(file, "[{}] Entering BACKGROUND mode", chrono::Local::now());
        }
        pass_gui_lib::run_native_messaging();
        std::process::exit(0);
    } else {
        pass_gui_lib::run();
    }
}