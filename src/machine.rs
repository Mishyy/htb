use std::env;
use std::path::Path;
use std::str::FromStr;

use json::JsonValue;

#[derive(Debug, Clone)]
pub struct Machine {
    pub id: u64,
    pub name: String,
    pub os: OperatingSystem,
    pub difficulty: Difficulty,
    pub ip: String,
    pub home: String,
}

impl From<&JsonValue> for Machine {
    fn from(info: &JsonValue) -> Self {
        let name = info["name"].to_string();
        let machine = Machine {
            id: info["id"].as_u64().unwrap(),
            name: info["name"].to_string(),
            os: info["os"]
                .to_string()
                .parse::<OperatingSystem>()
                .expect("os"),
            difficulty: info["difficultyText"]
                .to_string()
                .parse::<Difficulty>()
                .expect("difficultyText"),
            ip: info["ip"].to_string(),
            home: Path::new(&env::var("CS_OPT").expect("You must define CS_OPT!"))
                .join("htb")
                .join("lab")
                .join(name.to_lowercase())
                .to_str()
                .expect("home")
                .to_string(),
        };
        return machine;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OperatingSystem {
    Linux,
    Windows,
}

impl FromStr for OperatingSystem {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Linux" => Ok(Self::Linux),
            "Windows" => Ok(Self::Windows),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Difficulty {
    Easy,
    VeryEasy,
    Medium,
    Hard,
    Insane,
}

impl FromStr for Difficulty {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Easy" => Ok(Self::Easy),
            "VeryEase" => Ok(Self::VeryEasy),
            "Medium" => Ok(Self::Medium),
            "Hard" => Ok(Self::Hard),
            "Insane" => Ok(Self::Insane),
            _ => Err(()),
        }
    }
}
