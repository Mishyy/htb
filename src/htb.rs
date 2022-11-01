extern crate attohttpc;
extern crate json;

use std::env;
use std::error::Error;

use attohttpc::{header, Method, RequestBuilder};
use json::JsonValue;

use crate::machine::Machine;
use crate::RES;

const API_URL: &str = "https://www.hackthebox.com/api/v4";

type Result<T> = std::result::Result<T, String>;

pub fn get_machines() -> Result<Vec<Machine>> {
    let response = fetch_get(&["machine", "list"]).map_err(|e| e.to_string())?;
    if response.has_key("info") {
        let info = &response["info"];
        let mut machines: Vec<Machine> = Vec::new();
        if info.is_array() {
            for machine in info.members() {
                machines.push(Machine::from(&*machine));
            }
        } else {
            machines.push(Machine::from(info))
        }
        return Ok(machines);
    }
    return Err(response.pretty(4));
}

pub fn get_machine(name: &str) -> Result<Machine> {
    let response = fetch_get(&["machine", "profile", name]).map_err(|e| e.to_string())?;
    if response.has_key("message") {
        return Err((&response["message"]).to_string());
    } else if response.has_key("info") {
        return Ok(Machine::from(&response["info"]));
    }
    return Err(response.pretty(4));
}

pub fn join_machine(machine: &Machine) -> Result<()> {
    let response =
        fetch_post(&["machine", "play", &machine.id.to_string()]).map_err(|e| e.to_string())?;
    if response.has_key("message") && (&response["message"].to_string()).eq("Playing machine.") {
        println!("Spawned {RES}1m{}{RES}0m on {}!", machine.name, machine.ip);
        return Ok(());
    }
    return Err(response.pretty(4));
}

pub fn leave_machine() -> Result<()> {
    let response = fetch_get(&["machine", "active"]).map_err(|e| e.to_string())?;
    if response.has_key("info") {
        if (&response["info"]).is_null() {
            println!("You have no active machine!");
            return Ok(());
        }

        let response = fetch_post(&["machine", "stpo"]).map_err(|e| e.to_string())?;
        if response.has_key("message") && *&response["message"].eq("Stopped playing machine.") {
            println!("Left {RES}1m{}{RES}0m!", &response["info"]["name"]);
            return Ok(());
        }
    }
    return Err(response.pretty(4));
}

pub fn own_machine(machine: &Machine, flag: &str, difficulty: u16) -> Result<()> {
    let mut json = JsonValue::new_object();
    json.insert("flag", flag)
        .and(json.insert("id", machine.id))
        .and(json.insert("difficulty", difficulty))
        .map_err(|e| e.to_string())?;

    let response = fetch_post_data(&["machine", "own"], json).map_err(|e| e.to_string())?;
    if response.has_key("message") {
        println!("{}", &response["message"].to_string());
        return Ok(());
    }
    return Err(response.pretty(4));
}

fn fetch_get(endpoint: &[&str]) -> std::result::Result<JsonValue, Box<dyn Error>> {
    let json = build_req(Method::GET, endpoint.join("/"))
        .send()?
        .text_utf8()?;
    return Ok(json::parse(&json.as_str())?);
}

fn fetch_post(endpoint: &[&str]) -> std::result::Result<JsonValue, Box<dyn Error>> {
    let json = build_req(Method::POST, endpoint.join("/"))
        .send()?
        .text_utf8()?;
    return Ok(json::parse(&json.as_str())?);
}

fn fetch_post_data(
    endpoint: &[&str],
    json: JsonValue,
) -> std::result::Result<JsonValue, Box<dyn Error>> {
    let json = build_req(Method::POST, endpoint.join("/"))
        .text(&json.to_string())
        .send()?
        .text_utf8()?;
    return Ok(json::parse(&json.as_str())?);
}

fn build_req(method: Method, endpoint: String) -> RequestBuilder {
    return match method {
        Method::GET => attohttpc::get(format!("{API_URL}/{endpoint}")),
        Method::POST => attohttpc::post(format!("{API_URL}/{endpoint}"))
            .header(header::CONTENT_TYPE, "application/json; charset=utf-8"),
        _ => panic!("Invalid method given: {}", method.to_string()),
    }
    .bearer_auth(&env::var("HTB_API_KEY").expect("You must define HTB_API_KEY!"))
    .header(header::CONNECTION, "Close")
    .header(header::REFERER, "https://app.hackthebox.com/")
    .header(header::ORIGIN, "https://app.hackthebox.com/")
    .header(header::ACCEPT, "application/json")
    .header(header::ACCEPT_ENCODING, "gzip, deflate");
}
