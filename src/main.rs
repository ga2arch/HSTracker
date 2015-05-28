#![feature(slice_patterns)]
#![feature(convert)]

extern crate yaml;

use std::io::BufReader;
use std::fs::File;

use yaml::constructor::*;
use yaml::constructor::YamlStandardData::{YamlMapping, YamlString, YamlInteger};

#[derive(Debug)]
struct Season {
    num: isize,
    matches: Vec<Match>,
}

impl Season {
    fn new(num: isize, matches: Vec<Match>) -> Season {
        Season { num: num, matches: matches }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
enum MatchResult {
    Win,
    Loss,
}

#[derive(Debug)]
enum MatchType {
    Ranked,
    Casual,
    Friendly,
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

fn parse_match(id: isize, map: &Vec<(YamlStandardData, YamlStandardData)>) -> Result<Match, &'static str> {
    let mut m = Match::new(id);

    for e in map.as_slice() {
        match e {
            &(YamlString(ref key), YamlString(ref value)) =>

                match key.as_str() {
                    "deck"     => m.deck = value.to_string(),
                    "opponent" => m.opponent = value.to_string(),
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

    println!("{:?}", seasons);

}
