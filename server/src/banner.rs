/// GG-Retro ASCII banner with Gadu-Gadu sun
/// Colors match the classic GG orange/yellow theme

// ANSI color codes for GG theme
const ORANGE: &str = "\x1b[38;5;208m";  // GG Orange
const YELLOW: &str = "\x1b[38;5;220m";  // GG Yellow
const WHITE: &str = "\x1b[38;5;255m";   // White
const RESET: &str = "\x1b[0m";

pub fn print_banner() {
    let version = env!("CARGO_PKG_VERSION");

    println!(r#"
{o}
{o}   ██████╗  ██████╗       ██████╗ ███████╗████████╗██████╗  ██████╗
{o}  ██╔════╝ ██╔════╝       ██╔══██╗██╔════╝╚══██╔══╝██╔══██╗██╔═══██╗
{y}  ██║  ███╗██║  ███╗█████╗██████╔╝█████╗     ██║   ██████╔╝██║   ██║
{y}  ██║   ██║██║   ██║╚════╝██╔══██╗██╔══╝     ██║   ██╔══██╗██║   ██║
{o}  ╚██████╔╝╚██████╔╝      ██║  ██║███████╗   ██║   ██║  ██║╚██████╔╝
{o}   ╚═════╝  ╚═════╝       ╚═╝  ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝
{w}
{w}  Gadu-Gadu 6.0 Protocol Server                          v{version}
{y}  ─────────────────────────────────────────────────────────────────
{r}"#,
        y = YELLOW,
        o = ORANGE,
        w = WHITE,
        r = RESET,
        version = version
    );
}
