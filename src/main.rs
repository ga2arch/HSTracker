#![feature(slice_patterns)]
#![feature(convert)]

extern crate yaml;
extern crate term;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::fmt;
use std::env;
use std::ascii::{AsciiExt, OwnedAsciiExt};

use yaml::constructor::*;
use yaml::constructor::YamlStandardData::{YamlMapping, YamlString, YamlInteger};

struct Season {
    num: isize,
    matches: Vec<Match>,
}

impl Season {
    fn new(num: isize, matches: Vec<Match>) -> Season {
        Season { num: num, matches: matches }
    }

    fn winrate(&self) -> f32 {
        let wins: Vec<&Match> = self.matches.iter()
            .filter(|m| m.result == MatchResult::Win).collect();

        let total   = self.matches.len() as f32;
        let winrate = (wins.len() as f32) / total * 100.0;

        winrate
    }

    fn winrate_by_deck(&self, name: &String, class: &String) -> f32 {
        let deck = lowercase(&format!("{} {}", name, class));

        let dms: Vec<&Match> = self.matches.iter()
            .filter(|m| lowercase(&m.deck) == deck).collect();

        let wins: Vec<&&Match> = dms.iter()
            .filter(|m| m.result == MatchResult::Win).collect();

        let total   = dms.len() as f32;
        let winrate = (wins.len() as f32) / total * 100.0;

        winrate
    }
}


struct Match {
    id: isize,
    deck: String,
    opponent: String,
    result: MatchResult,
    kind: MatchType,
}

impl Match {
    fn new(id: isize) -> Match {
        Match { id: id,
                deck: String::new(),
                opponent: String::new(),
                result: MatchResult::Win,
                kind: MatchType::Casual }
    }
}

impl fmt::Display for Season {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //let mut result = vec![];
        writeln!(f, "{}", format!("Season {}", self.num));
        let mut t = term::stdout().unwrap();

        for m in &self.matches {
            if (m.result == MatchResult:: Win) {
                t.fg(term::color::GREEN).unwrap();
            } else {
                t.fg(term::color::RED).unwrap();
            }
            //result.push(
            writeln!(f, "{}",
                format!(" {}) {} vs {} \t\t{}",
                    m.id, m.deck, m.opponent, m.kind));

            t.reset().unwrap();
        }

        writeln!(f, "{}", "")
        //.reset().unwrap();
    }
}

#[derive(PartialEq, Eq)]
enum MatchResult {
    Win,
    Loss,
}

impl fmt::Display for MatchResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            &MatchResult::Win  => "Win",
            &MatchResult::Loss => "Loss",
        };

        write!(f, "{}", s)
    }
}

enum MatchType {
    Ranked,
    Casual,
    Friendly,
}

impl fmt::Display for MatchType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            &MatchType::Ranked   => "Rankend",
            &MatchType::Casual   => "Casual",
            &MatchType::Friendly => "Friendly",
        };

        write!(f, "{}", s)
    }
}

fn parse_seasons(map: &Vec<(YamlStandardData, YamlStandardData)>) -> Result<Vec<Season>, &'static str> {
    let mut seasons: Vec<Season> = Vec::new();

    match map.as_slice() {
        [(YamlString(ref name), YamlMapping(ref map))] => {
            let mut matches: Vec<Match>  = Vec::new();

            for m in map.as_slice() {

                match m {
                    &(YamlInteger(ref id), YamlMapping(ref content)) =>
                        matches.push(parse_match(*id, content).unwrap()),

                    _ => continue,
                }
            }

            let s_name = name.to_string();
            let temp: Vec<&str> = s_name.split(' ').collect();
            let season_num = temp[1].to_string().parse::<isize>().unwrap();

            seasons.push(Season::new(season_num, matches));
            Ok(seasons)
        },

        _ => Err("Error"),
    }
}

fn lowercase(s: &String) -> String {
    s.chars().map(|c| c.to_lowercase().next().unwrap()).collect::<String>()
}

fn capitalize(s: &String) -> String {
    s.chars().enumerate()
        .map(|(i, c)| if (i == 0) {
                c.to_uppercase().next().unwrap()
            } else {
                c.to_lowercase().next().unwrap()
            } ).collect::<String>()
}

fn parse_match(id: isize, map: &Vec<(YamlStandardData, YamlStandardData)>) -> Result<Match, &'static str> {
    let mut m = Match::new(id);

    for e in map.as_slice() {
        match e {
            &(YamlString(ref key), YamlString(ref value)) =>

                match key.as_str() {
                    "deck"     => m.deck = value.trim().to_string(),
                    "opponent" => m.opponent = value.trim().to_string(),
                    "result"   =>
                        match value.as_str() {
                            "Win"  => m.result = MatchResult::Win,
                            "Loss" => m.result = MatchResult::Loss,
                            _      => continue,
                        },
                    "type"     =>
                        match value.as_str() {
                            "Ranked"   => m.kind = MatchType::Ranked,
                            "Casual"   => m.kind = MatchType::Casual,
                            "Friendly" => m.kind = MatchType::Friendly,
                            _          => continue,
                        },
                    _ => continue,
                },

            _ => continue
        }
    }

    Ok(m)
}

fn parse(data: Vec<YamlStandardData>) -> Result<Vec<Season>, &'static str> {
    for doc in data.iter() {
        match doc {
            &YamlMapping(ref map) => return parse_seasons(map),

            _ => return Err("No docs"),
        }
    }

    Err("No docs")
}



fn main() {
    let path = "/Users/Gabriele/Dropbox/hearthstone.yaml";

    let mut reader = BufReader::new(File::open(path).unwrap());
    let data = yaml::parse_io_utf8(&mut reader).unwrap();

    let seasons = parse(data);

    for s in seasons.unwrap() {
        println!("{}", s);

        if (env::args().count() == 3) {
            let name  = env::args().nth(1).unwrap();
            let class = env::args().nth(2).unwrap();

            println!("{} {} Winrate: {}%", capitalize(&name),
                capitalize(&class), s.winrate_by_deck(&name, &class));
        }

        println!("Total Winrate: {}%", s.winrate());
    }

}
