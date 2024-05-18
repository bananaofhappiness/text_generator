use std::env;
use std::fs;
use std::error::Error;
pub struct Config {
    // pub language: String,
    pub depth_level: u8,
    pub dev_mode: bool,
    // pub dev_text: Option<String>, //used only when dev mode is on
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str>{
        if args.len() < 2 {
            return Err("\
Недостаточно аргументов.
Используйте следующие аргументы:
число от 1 до 5 для выбора уровня глубины алгоритма.

Not enough arguments.
Use following arguments:
a number from 1 to 5 to choose depth level of algorithm.");
        }

        let depth_level: u8;
        // let language = args[1].clone();
        if let Ok(val) = args[1].clone().parse::<u8>() {
            depth_level = val;
        }
        else {
            return Err("Не удалось преобразовать аргумент в целое число.\nCouldn't convert argument into an integer.");
        }

        if depth_level < 1 {
            return Err("Уровень глубины не может быть меньше 1.\nDepth level can't be less than 1.");
        } else if depth_level > 5 {
            return Err("Уровень глубины не может быть больше 5.\nDepth level can't be more than 5.");
        }

        let dev_mode = env::var("DEV_MODE").is_ok();
        if dev_mode {
            return Ok(Config{
                // language,
                depth_level,
                dev_mode,
                // dev_text: Some(args[2].clone())
            })
        }
        Ok(Config{
            // language,
            depth_level,
            dev_mode,
            // dev_text: None
        })

    }
}

pub mod dev_fn {
    use super::*;
    use std::{collections::BTreeMap, io::Write};
    use unicode_segmentation::UnicodeSegmentation;

    pub fn prepare_text() -> Result<(), Box<dyn Error>> {
        let whitelisted = ['а', 'б', 'в', 'г', 'д', 'е', 'ё', 'ж', 'з', 'и', 'й', 'к', 'л', 'м', 'н', 'о', 'п', 'р', 'с', 'т', 'у', 'ф', 'х', 'ц', 'ч', 'ш', 'щ', 'ъ', 'ы', 'ь', 'э', 'ю', 'я', ' ', '\n'];
        let paths = match fs::read_dir("texts") {
            Ok(paths) => paths,
            Err(err) => return Err(Box::new(err)),
        };
        for path in paths {
            let contents = fs::read_to_string(format!("{}",path.as_ref().unwrap().path().display()))?.to_lowercase();
            let mut prep_contents: String = contents
                .chars()
                .filter(|c| whitelisted.contains(c))
                .collect();
            prep_contents = prep_contents.replace("-", " ")
                .replace("\n", " ")
                .trim()
                .split(' ')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            if let Err(err) = fs::write(format!("prep_{}",path.as_ref().unwrap().path().display()), prep_contents) {
                return Err(Box::new(err))
            };
        }
        Ok(())
    }

    pub fn create_model() -> Result<(), Box<dyn Error>> {
        for level in 1..=5 {
            let mut map = BTreeMap::new();
            let paths = match fs::read_dir("prep_texts") {
                Ok(paths) => paths,
                Err(err) => return Err(Box::new(err)),
            };
            for path in paths {
                let contents = fs::read_to_string(format!("{}",path.as_ref().unwrap().path().display()))?;
                let contents: Vec<&str> = contents.graphemes(true).collect();
                for i in 0..=contents.len()-level {
                    let mut word = contents[i].to_string();
                    for j in 1..level {
                        word += contents[i+j];
                    }
                    let count: &mut u32 = map.entry(word.to_string()).or_insert(0);
                    *count += 1;
                }
            }
            let mut file = fs::File::create(format!("models/level_{}.json",level))?;
            let serialized = serde_json::to_string(&map)?;
            file.write_all(serialized.as_bytes())?;
        }
        Ok(())
    }
}

pub mod user_fn {
    use super::*;
    use std::{collections::BTreeMap, fs};
    use rand::prelude::*;

    pub fn generate_level_1_text() -> Result<String, Box<dyn Error>> {
        let file = fs::read_to_string(format!("models/level_1.json"))?;
        let map: BTreeMap<String, u32> = serde_json::from_str(&file).unwrap();

        let mut choices = Vec::new();
        for (key, value) in map.iter() {
            choices.push((key.clone(), value.clone()))
        }
        drop(map);

        let mut text = String::new();
        let mut rng = thread_rng();

        for _ in 0..200 {
            text += &choices.choose_weighted(&mut rng, |item| item.1).unwrap().0;
        }

        Ok(text)
    }

    pub fn generate_text(level: u8) -> Result<String, Box<dyn Error>>{
        let file = fs::read_to_string(format!("models/level_{}.json",level))?;
        let map: BTreeMap<String, u32> = serde_json::from_str(&file).unwrap();

        let mut text = String::from(" ");
        text += &make_first_choice(&map, " ");

        dbg!(&text);
        for _ in 0..200 {
            let split_pos = text.char_indices().nth_back((level-2) as usize).unwrap().0;
            dbg!(&text[split_pos..]);
            text += &make_choices(&map, &text[split_pos..]);
        }

        Ok(text)
    }

    fn make_choices(map: &BTreeMap<String, u32>, find: &str) -> String {
        let mut rng = thread_rng();
        let mut choices = Vec::new();
        for (key, value) in map.range(find.to_owned()..).take_while(|(k, _)| k.starts_with(find)) {
            choices.push((key,value))
        }
        choices.choose_weighted(&mut rng, |item| item.1).unwrap().0.chars().last().unwrap().to_string()
    }

    fn make_first_choice<'a> (map: &'a BTreeMap<String, u32>, find: &str) -> &'a str {
        let mut rng = thread_rng();
        let mut choices = Vec::new();
        for (key, value) in map.range(find.to_owned()..).take_while(|(k, _)| k.starts_with(find)) {
            choices.push((key,value))
        }
        choices.choose_weighted(&mut rng, |item| item.1).unwrap().0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_counts_right() {
        todo!()
    }

    #[test]
    fn it_gets_to_the_end_of_file() {
        todo!()
    }
}