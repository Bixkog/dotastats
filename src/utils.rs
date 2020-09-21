use std::io::{BufRead, BufReader, Write};
use std::fs::{File, OpenOptions};
use tokio::time::{delay_until, Duration, Instant};

pub fn read_lines(filename : &String) -> std::io::Result<Vec<String>> {
    let file = File::open(filename);
    if let Err(_) = file {
        println!("Unable to open file {}. Creating it.", filename);
        File::create(filename)?;
        return Ok(vec![])
    }
    Ok(BufReader::new(file.unwrap()).lines().map(|l| l.unwrap()).collect())
}

pub fn append_lines(filename : &String, lines : &Vec<String>) -> std::io::Result<()> {
    let mut open_options = OpenOptions::new();
    open_options.append(true);
    let mut file = open_options.open(filename).or_else(|_| File::create(filename))?;
    for line in lines {
        let output = [line, "\n"].concat();
        file.write_all(output.as_bytes()).unwrap();
    }
    Ok(())
}

pub fn clear_file(filename : &String) -> std::io::Result<()> {
    OpenOptions::new().write(true).truncate(true).open(filename)?;
    Ok(())
}

/// Checks if match has parsing data, e.g. players lane, match skill
pub fn is_match_parsed(match_json : &serde_json::Value) -> bool {
    if match_json["skill"].is_null() {return false};
    if !match_json["players"].is_array() {return false};
    let players = match_json["players"].as_array().unwrap();
    for player in players {
        if player["lane"].is_null() {return false};
    };
    true
}

/// Sends get requests. Waits 1 second to ensure rpm <= 60.
pub async fn get_req_at60rpm(url : &String) -> reqwest::Result<serde_json::Value> {
    let start_inst = Instant::now();
    let response = reqwest::get(url)
            .await?
            .text()
            .await?;
    delay_until(start_inst + Duration::from_secs(1)).await;
    Ok(serde_json::from_str(&response).expect(format!("Can't parse response: {}", response).as_str()))
}