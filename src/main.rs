use text_generator::Config;
use std::env;
use std::process;
use text_generator::dev_fn;
use text_generator::user_fn;
fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Проблема с парсингом аргументов: | Problem with argument parsing:\n{err}");
        process::exit(1);
    });

    if config.dev_mode {
        if let Err(err) = dev_fn::prepare_text() {
            eprintln!("{err}");
            process::exit(1);
        };
        if let Err(err) = dev_fn::create_model() {
            eprintln!("{err}");
            process::exit(1);
        };
    }

    match config.depth_level {
        1 => {
            let text = user_fn::generate_level_1_text().unwrap_or_else(|err| {
                eprintln!("Не удалось сгенерировать текст: | Couldn't generate text:\n{err}");
                process::exit(1);
            });
            println!("{}", text);
        },

        other => {
            let text = user_fn::generate_text(other).unwrap_or_else(|err| {
                eprintln!("Не удалось сгенерировать текст: | Couldn't generate text:\n{err}");
                process::exit(1);
            });
            println!("{}", text);
        },

    }
}