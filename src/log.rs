#[macro_export]
macro_rules! write_log {
    (ERROR, $($arg:tt)*) => {
        use colored::Colorize;
        for line in format!($($arg)*).lines() {
            println!("{} {}", "[ERROR]".red(), line);
        }
    };
    (WARN, $($arg:tt)*) => {
        if cfg!(debug_assertions) {
            use colored::Colorize;
            for line in format!($($arg)*).lines() {
                println!("{} {}", " [WARN]".yellow(), line);
            }
        }
    };
    (LOG, $($arg:tt)*) => {
        if cfg!(debug_assertions) {
            for line in format!($($arg)*).lines() {
                println!("  [LOG] {}", line);
            }
        }
    };
    (DEBUG, $($arg:tt)*) => {
        if cfg!(debug_assertions) {
            use colored::Colorize;
            for line in format!($($arg)*).lines() {
                let color = colored::Color::TrueColor { r: 100, g: 100, b: 100 };
                println!("{}", format!("[DEBUG] {}", line).color(color));
            }
        }
    }
}
