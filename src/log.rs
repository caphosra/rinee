#[macro_export]
macro_rules! write_log {
    (ERROR, $($arg:tt)*) => {
        use colored::Colorize;
        println!("{} {}", "[ERROR]".red(), format!($($arg)*));
    };
    (WARN, $($arg:tt)*) => {
        if cfg!(debug_assertions) {
            use colored::Colorize;
            println!(" {} {}", "[WARN]".yellow(), format!($($arg)*));
        }
    };
    (LOG, $($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!("  [LOG] {}",  format!($($arg)*));
        }
    };
    (DEBUG, $($arg:tt)*) => {
        if cfg!(debug_assertions) {
            use colored::Colorize;
            println!("{}", format!("[DEBUG] {}", format!($($arg)*)).color(colored::Color::TrueColor { r: 100, g: 100, b: 100 }));
        }
    }
}
