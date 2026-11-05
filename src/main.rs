use mines::{cli, ui::App};

fn main() {
    match cli::parse(std::env::args().skip(1)) {
        Ok(cli::Action::Run(config)) => {
            if let Err(error) = App::new(config).run() {
                eprintln!("mines: {error}");
                std::process::exit(1);
            }
        }
        Ok(cli::Action::Print(text)) => println!("{text}"),
        Err(error) => {
            eprintln!("mines: {error}\n\n{}", cli::help());
            std::process::exit(2);
        }
    }
}
