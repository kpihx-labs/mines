use crate::game::GameConfig;

pub enum Action {
    Run(GameConfig),
    Print(String),
}

pub fn parse(args: impl Iterator<Item = String>) -> Result<Action, String> {
    let mut difficulty = "intermediate".to_string();
    let mut width = None;
    let mut height = None;
    let mut mines = None;
    let mut seed = None;
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Action::Print(help())),
            "-V" | "--version" => {
                return Ok(Action::Print(format!(
                    "mines {}",
                    env!("CARGO_PKG_VERSION")
                )));
            }
            "-d" | "--difficulty" => difficulty = value(&mut args, &arg)?,
            "--width" => width = Some(number(&mut args, &arg)?),
            "--height" => height = Some(number(&mut args, &arg)?),
            "--mines" => mines = Some(number(&mut args, &arg)?),
            "--seed" => seed = Some(number(&mut args, &arg)?),
            unknown => return Err(format!("unknown option: {unknown}")),
        }
    }

    let mut config = match difficulty.as_str() {
        "beginner" => GameConfig::beginner(),
        "intermediate" => GameConfig::intermediate(),
        "expert" => GameConfig::expert(),
        "custom" => GameConfig::new(
            width.ok_or("custom difficulty requires --width")?,
            height.ok_or("custom difficulty requires --height")?,
            mines.ok_or("custom difficulty requires --mines")?,
            "custom",
        )?,
        _ => return Err("difficulty must be beginner, intermediate, expert, or custom".into()),
    };
    if (width.is_some() || height.is_some() || mines.is_some()) && difficulty != "custom" {
        return Err("--width, --height, and --mines require --difficulty custom".into());
    }
    config.seed = seed;
    Ok(Action::Run(config))
}

fn value(args: &mut impl Iterator<Item = String>, option: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{option} requires a value"))
}

fn number<T: std::str::FromStr>(
    args: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<T, String> {
    value(args, option)?
        .parse()
        .map_err(|_| format!("invalid value for {option}"))
}

pub fn help() -> String {
    format!(
        "mines {}\nA fast, responsive Minesweeper for the terminal.\n\nUSAGE:\n  mines [OPTIONS]\n\nOPTIONS:\n  -d, --difficulty <LEVEL>  beginner, intermediate, expert, or custom [default: intermediate]\n      --width <COLUMNS>      Custom board width (5..=100)\n      --height <ROWS>        Custom board height (5..=100)\n      --mines <COUNT>        Custom mine count\n      --seed <NUMBER>        Reproducible board seed\n  -h, --help                 Print help\n  -V, --version              Print version\n\nEXAMPLES:\n  mines -d beginner\n  mines -d expert\n  mines -d custom --width 40 --height 20 --mines 130",
        env!("CARGO_PKG_VERSION")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_intermediate() {
        let Action::Run(config) = parse(std::iter::empty()).expect("default config") else {
            panic!("expected run");
        };
        assert_eq!((config.width, config.height, config.mines), (16, 16, 40));
    }

    #[test]
    fn parses_custom_board() {
        let args = [
            "-d", "custom", "--width", "40", "--height", "20", "--mines", "130",
        ];
        let Action::Run(config) = parse(args.into_iter().map(str::to_string)).expect("custom")
        else {
            panic!("expected run");
        };
        assert_eq!((config.width, config.height, config.mines), (40, 20, 130));
    }

    #[test]
    fn rejects_partial_custom_board() {
        let args = ["-d", "custom", "--width", "10"];
        assert!(parse(args.into_iter().map(str::to_string)).is_err());
    }
}
