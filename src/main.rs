#![feature(advanced_slice_patterns, slice_patterns, convert)]

extern crate argparse;
extern crate yaml;
extern crate term;
extern crate chrono;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::fmt;

use yaml::constructor::*;
use yaml::constructor::YamlStandardData::{YamlMapping, YamlString, YamlInteger};

use std::collections::{HashMap, HashSet};
use argparse::{ArgumentParser, List};
use chrono::*;

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

    fn deck(&self, name: &String, class: &String) -> Season  {
        let deck = lowercase(&format!("{} {}", name, class));

        let dms: Vec<Match> = self.matches.clone().into_iter()
            .filter(|m| lowercase(&m.deck) == deck).collect();

        Season::new(self.num, dms)
    }

    fn vs(&self) -> HashMap<String, (usize, f32)> {
        let mut wrates = HashMap::new();
        let mut opps = HashSet::new();

        let mut ms = self.matches.to_vec();
        ms.sort_by(|a, b| a.opponent.cmp(&b.opponent));

        for m in &ms {
            opps.insert(m.opponent.clone());
        }

        for o in opps {
            let dms: Vec<&Match> = self.matches.iter()
                .filter(|m| lowercase(&m.opponent) == lowercase(&o)).collect();

            let wins: Vec<&&Match> = dms.iter()
                .filter(|m| m.result == MatchResult::Win).collect();

            let total   = dms.len() as f32;
            let winrate = (wins.len() as f32) / total * 100.0;

            wrates.insert(o, (dms.len(), winrate,));
        }

        wrates
    }

    fn time(&self, ts: DateTime<Local>) -> Season {
        let dms: Vec<Match> = self.matches.clone().into_iter()
            .filter(|m| (m.datetime.day(), m.datetime.month()) == (ts.day(), ts.month())).collect();

        Season::new(self.num, dms)
    }
}

impl fmt::Display for Season {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", format!("Season {}", self.num));

        for m in &self.matches {
            writeln!(f, "{}", m);
        }

        writeln!(f, "{}", "")
    }
}

#[derive(Debug, Clone)]
struct Match {
    id: isize,
    deck: String,
    opponent: String,
    result: MatchResult,
    kind: MatchType,
    datetime: DateTime<Local>,
}

impl Match {
    fn new(id: isize) -> Match {
        Match { id:        id,
                deck:      String::new(),
                opponent:  String::new(),
                result:    MatchResult::Win,
                kind:      MatchType::Casual,
                datetime:  Local::now() }
    }
}

impl fmt::Display for Match {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut t = term::stdout().unwrap();

        if self.result == MatchResult:: Win {
            t.fg(term::color::GREEN).unwrap();
        } else {
            t.fg(term::color::RED).unwrap();
        }

        write!(f, "{}",
            format!(" {}) {} vs {} \t\t{}",
                self.id, self.deck, self.opponent, self.kind));

        t.reset().unwrap();

        write!(f, "{}", "")
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
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

#[derive(Debug, Clone)]
enum MatchType {
    Ranked,
    Casual,
    Friendly,
}

impl fmt::Display for MatchType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            &MatchType::Ranked   => "Ranked",
            &MatchType::Casual   => "Casual",
            &MatchType::Friendly => "Friendly",
        };

        write!(f, "{}", s)
    }
}

fn parse_season(map: &Vec<(YamlStandardData, YamlStandardData)>) -> Result<Season, &'static str> {
    match map.as_slice() {
        [(YamlString(ref name), YamlMapping(ref map))] => {
            let mut matches: Vec<Match>  = Vec::new();

            for m in map.as_slice() {
                match m {
                    &(YamlInteger(ref id), YamlMapping(ref content)) =>
                        matches.push(parse_match(*id, content)),

                    _ => continue,
                }
            }

            let s_name = name.to_string();
            let temp: Vec<&str> = s_name.split(' ').collect();
            let season_num = temp[1].to_string().parse::<isize>().unwrap();


            Ok(Season::new(season_num, matches))
        },

        _ => Err("Error"),
    }
}

fn lowercase(s: &String) -> String {
    s.chars().map(|c| c.to_lowercase().next().unwrap()).collect::<String>()
}

fn capitalize(s: &String) -> String {
    s.chars().enumerate()
        .map(|(i, c)| if i == 0 {
                c.to_uppercase().next().unwrap()
            } else {
                c.to_lowercase().next().unwrap()
            }).collect::<String>()
}

fn parse_match(id: isize, map: &Vec<(YamlStandardData, YamlStandardData)>) -> Match {
    let mut m = Match::new(id);

    for e in map.as_slice() {
        match e {
            &(YamlString(ref key), YamlString(ref value)) =>
                match key.as_str() {
                    "deck"     => m.deck     = value.trim().to_string(),
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
                    "datetime" =>
                        m.datetime = Local.datetime_from_str(value, "%Y-%m-%d %H:%M:%S").unwrap(),

                    _ => continue,
                },

            _ => continue
        }
    }

    m
}

fn parse(data: Vec<YamlStandardData>) -> Result<Vec<Season>, &'static str> {
    let mut seasons: Vec<Season> = Vec::new();

    for doc in data.iter() {
        match doc {
            &YamlMapping(ref map) =>
                seasons.push(parse_season(map).unwrap()),

            _ => return Err("No docs"),
        }
    }

    Ok(seasons)
}

fn main() {
    let path = "hearthstone.yaml";

    let mut args: Vec<String> = Vec::new();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Calc HS stats");
        ap.refer(&mut args)
            .add_argument("args", List,
                          "Deck to calc winrate for");
        ap.parse_args_or_exit();
    }

    let mut reader = BufReader::new(File::open(path).unwrap());
    let data = yaml::parse_io_utf8(&mut reader).unwrap();

    let seasons = parse(data).unwrap();

    if args.len() >= 1 {
        let season = seasons.last().unwrap();

        match args[0].as_str() {
            "deck" => {
                let deck   = &args[1..3];
                let name   = deck[0].to_string();
                let class  = deck[1].to_string();

                let data = season.deck(&name, &class);

                println!("Matches: ");

                for m in &data.matches {
                    println!("{}", m);
                }

                println!("\nMatches: {}", data.matches.len());
                println!("Winrate: {}%\n", data.winrate());

                let wrates = season.vs();

                for (k, v) in wrates {
                    println!("{}\t\t {}% {}", k, v.1, v.0);
                }
            },

            "show" => {
                print!("{}", season);
                println!("Total Winrate: {}%", season.winrate());
            },

            "today" => {
                let data = season.time(Local::now());

                println!("Matches: ");

                for m in &data.matches {
                    
                    println!("{}", m);
                }

                println!("\nMatches: {}", data.matches.len());
                println!("Winrate: {}%\n", data.winrate());
            }

            "yesterday" => {
                let date = Local::now().checked_sub(Duration::days(1)).unwrap();
                let data = season.time(date);

                println!("Matches: ");

                for m in &data.matches {
                    println!("{}", m);
                }

                println!("\nMatches: {}", data.matches.len());
                println!("Winrate: {}%\n", data.winrate());
            }

            _ => println!("Command unknown"),
        }
    }
}
