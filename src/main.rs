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
    result: Result,
    kind: Kind,
}

impl Match {
    fn new(id: isize) -> Match {
        Match { id: id,
                deck: String::new(),
                opponent: String::new(),
                result: Result::Win,
                kind: Kind::Casual }
    }
}

#[derive(Debug)]
enum Result {
    Win,
    Loss,
}

#[derive(Debug)]
enum Kind {
    Ranked,
    Casual,
    Friendly,
}

fn parse_seasons(map: &Vec<(YamlStandardData, YamlStandardData)>) -> Result<Vec<Season>, &'static str> {
    let mut seasons: Vec<Season> = Vec::new();

    match map.as_slice() {
        [(YamlString(ref name), YamlMapping(ref content))] => {
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
        }
    }

    Ok(seasons)
}

fn parse_match(id: isize, map: &Vec<(YamlStandardData, YamlStandardData)>) -> Result<Match, &'static str> {
    let mut m = Match::new(id);

    for m in map.as_slice() {
        match map {
            &(YamlString(ref key), YamlString(ref value)) =>

                match key.as_str() {
                    "deck"     => m.deck = value.to_string(),
                    "opponent" => m.opponent = value.to_string(),
                    "result"   =>
                        match value.as_str() {
                            "Win"  => m.result = Result::Win,
                            "Loss" => m.result = Result::Loss,
                            _      => Err("Unknown result"),
                        },
                    "type"     =>
                        match value.as_str() {
                            "Ranked"   => m.kind = Kind::Ranked,
                            "Casual"   => m.kind = Kind::Casual,
                            "Friendly" => m.kind = Kind::Friendly,
                            _          => Err("Unknown match type"),
                        },
                    _ => continue,
                },

            _ => continue
        }
    }

    Ok(m)
}

fn parse(doc: Vec<YamlStandardData>) -> Vec<Season> {

    for doc in data.iter() {
        match doc {
            &YamlMapping(ref map) => parse_seasons(map),

            _ => panic!("Error"),
        }
    }
}

fn main() {
    let path = "/Users/Gabriele/Dropbox/hearthstone.yaml";

    let mut reader = BufReader::new(File::open(path).unwrap());
    let data = yaml::parse_io_utf8(&mut reader).unwrap();

    let mut seasons: Vec<Season> = Vec::new();

    match data.first().unwrap() {

        &YamlMapping(ref content) =>

            match content.as_slice() {
                [(YamlString(ref name), YamlMapping(ref content))] => {
                    let mut matches: Vec<Match>  = Vec::new();

                    for e in content.as_slice() {

                        match e {
                            &(YamlInteger(ref id), YamlMapping(ref content)) => {
                                let mut m = Match::new(*id);

                                for e in content.as_slice() {

                                    match e {
                                        &(YamlString(ref key), YamlString(ref value)) =>
                                            match key.as_str() {
                                                "deck"     => m.deck = value.to_string(),
                                                "opponent" => m.opponent = value.to_string(),
                                                "result"   =>
                                                    match value.as_str() {
                                                        "Win"  => m.result = Result::Win,
                                                        "Loss" => m.result = Result::Loss,
                                                        _      => continue,
                                                    },
                                                "type"     =>
                                                    match value.as_str() {
                                                        "Ranked"   => m.kind = Kind::Ranked,
                                                        "Casual"   => m.kind = Kind::Casual,
                                                        "Friendly" => m.kind = Kind::Friendly,
                                                        _          => continue,
                                                    },
                                                _ => continue,
                                            },

                                        _ => continue
                                    }
                                }

                                matches.push(m);
                            },

                            _ => continue,
                        }

                    }

                    let s_name = name.to_string();
                    let temp: Vec<&str> = s_name.split(' ').collect();
                    let season_num = temp[1].to_string().parse::<isize>().unwrap();

                    seasons.push(Season::new(season_num, matches));
                },

                _ => panic!("Error"),
            },

        _ => panic!("Error"),
    }

    println!("{:?}", seasons);

}
