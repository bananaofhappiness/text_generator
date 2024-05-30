use std::env;
use std::fs;
use std::error::Error;

const DEPTH: usize = 20;

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
число от 1 до 10 для выбора уровня глубины алгоритма.

Not enough arguments.
Use following arguments:
a number from 1 to 10 to choose depth level of algorithm.");
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
        } else if depth_level > DEPTH as u8 {
            return Err("Уровень глубины не может быть больше 20.\nDepth level can't be more than 20.");
        }

        let dev_mode = env::var("DEV_MODE").is_ok();
        // if dev_mode {
        //     return Ok(Config{
        //         // language,
        //         depth_level,
        //         dev_mode,
        //         // dev_text: Some(args[2].clone())
        //     })
        // }
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
    use std::{collections::BTreeMap, io::Write, sync::mpsc, thread::{self, JoinHandle}};
    use unicode_segmentation::UnicodeSegmentation;

    pub fn prepare_text() -> Result<(), Box<std::io::Error>> {
        let whitelisted = ['а', 'б', 'в', 'г', 'д', 'е', 'ё', 'ж', 'з', 'и', 'й', 'к', 'л', 'м', 'н', 'о', 'п', 'р', 'с', 'т', 'у', 'ф', 'х', 'ц', 'ч', 'ш', 'щ', 'ъ', 'ы', 'ь', 'э', 'ю', 'я', ' ', '\n'];
        let paths = match fs::read_dir("texts") {
            Ok(paths) => paths,
            Err(err) => return Err(Box::new(err)),
        };
        let mut v = Vec::<JoinHandle<Result<(), Box<std::io::Error>>>>::new();
        paths.into_iter().for_each(|path| {
            let handler = thread::spawn(move || {
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
                Ok(())
            });
            v.push(handler);
        });
        for jh in v.into_iter() {
            if let Err(err) = jh.join().unwrap() {
                return Err(err);
            }
        };
        Ok(())
    }
    

    pub fn create_model() -> Result<(), Box<std::io::Error>> {
        let mut v = Vec::<JoinHandle<Result<(), Box<std::io::Error>>>>::new();
        for level in 1..=DEPTH {
            let handler = thread::spawn(move || {
                let (tx, rx) = mpsc::channel();
                let mut map: BTreeMap<String, u64> = BTreeMap::new();
                let paths = match fs::read_dir("prep_texts") {
                    Ok(paths) => paths,
                    Err(err) => return Err(Box::new(err)),
                };
                for path in paths {
                    let txc = tx.clone();
                    thread::spawn(move || {
                        let mut content_map = BTreeMap::new();
                        let contents = fs::read_to_string(format!("{}",path.as_ref().unwrap().path().display())).unwrap();
                        let contents: Vec<&str> = contents.graphemes(true).collect();
                        for i in 0..=contents.len()-level {
                            let mut word = contents[i].to_string();
                            for j in 1..level {
                                word += contents[i+j];
                            }
                            let count: &mut u64 = content_map.entry(word.to_string()).or_insert(0);
                            *count += 1;
                        }
                        txc.send(content_map).unwrap();
                    });
                }
    
                drop(tx);
    
                for received in rx {
                    add_to_map(&mut map, received);
                }
    
                let mut file = fs::File::create(format!("models/level_{}.json",level))?;
                let serialized = serde_json::to_string(&map).unwrap();
                file.write_all(serialized.as_bytes())?;
                Ok(())
            });
            v.push(handler);
        }
        for jh in v.into_iter() {
            if let Err(err) = jh.join().unwrap() {
                return Err(err);
            }
        };
        Ok(())
    }

    pub fn add_to_map(map: &mut BTreeMap<String, u64>, map2: BTreeMap<String, u64>) {
        map2.into_iter().for_each(|x| {
            let count = map.entry(x.0).or_insert(0);
            *count += x.1;
        })
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

        for _ in 0..2000 {
            text += &choices.choose_weighted(&mut rng, |item| item.1).unwrap().0;
        }

        Ok(text)
    }

    pub fn generate_text(level: u8) -> Result<String, Box<dyn Error>>{
        let file = fs::read_to_string(format!("models/level_{}.json",level))?;
        let map: BTreeMap<String, u32> = serde_json::from_str(&file).unwrap();

        let mut text = String::from(" ");
        text += &make_first_choice(&map, " ");

        for _ in 0..2000 {
            let split_pos = text.char_indices().nth_back((level-2) as usize).unwrap().0;
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
        choices.choose_weighted(&mut rng, |item| item.1)
                .unwrap()
                .0
                .chars()
                .last()
                .unwrap()
                .to_string()
    }

    fn make_first_choice<'a> (map: &'a BTreeMap<String, u32>, find: &str) -> &'a str {
        let mut rng = thread_rng();
        let mut choices = Vec::new();
        for (key, value) in map.range(find.to_owned()..).take_while(|(k, _)| k.starts_with(find)) {
            choices.push((key,value))
        }
        choices.choose_weighted(&mut rng, |item| item.1)
                .unwrap()
                .0
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;
    use super::*;

    #[test]
    fn it_adds_to_map() {
        let mut map = BTreeMap::new();
        map.insert("a".to_owned(), 1);
        map.insert("b".to_owned(), 2);
        map.insert("c".to_owned(), 3);
        let mut map2 = BTreeMap::new();
        map2.insert("a".to_owned(), 1);
        map2.insert("b".to_owned(), 2);
        map2.insert("c".to_owned(), 3);
        let mut map3 = BTreeMap::new();
        map3.insert("a".to_owned(), 2);
        map3.insert("b".to_owned(), 4);
        map3.insert("c".to_owned(), 6);
        dev_fn::add_to_map(&mut map,map2);
        assert_eq!(map3, map);

        let mut map = BTreeMap::new();
        map.insert("a".to_owned(), 100);
        map.insert("b".to_owned(), 21923);
        map.insert("c".to_owned(), 3123);
        let mut map2 = BTreeMap::new();
        map2.insert("a".to_owned(), 146);
        map2.insert("b".to_owned(), 254);
        map2.insert("c".to_owned(), 3123);
        let mut map3 = BTreeMap::new();
        map3.insert("a".to_owned(), 246);
        map3.insert("b".to_owned(), 22177);
        map3.insert("c".to_owned(), 6246);
        dev_fn::add_to_map(&mut map,map2);
        assert_eq!(map3, map);
    }
}