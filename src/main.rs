use std::{time::Duration, sync::Arc};

use async_minecraft_ping::{ConnectionConfig, ServerDescription};
use futures::future::join_all;
use tokio::{fs::File, io::AsyncReadExt, time::timeout};
use tokio_postgres::{NoTls, Client};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut ips: Vec<String> = Vec::new();
    let mut buffer = Vec::new();

    let (client, connection) = tokio_postgres::connect("host=localhost user=crunchy password=dd8b58ea9523d875c4472effea7f54e4ee678cdf79211c66ea3e3297c2fbc64f dbname=postgres", NoTls).await.unwrap();
    let client = Arc::new(client);
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    File::open("scan.json")
        .await?
        .read_to_end(&mut buffer)
        .await?;

    pjson::parse(&buffer, 0, |start: usize, end: usize, info: usize| -> i64 {
        if info & (pjson::STRING | pjson::VALUE) == pjson::STRING | pjson::VALUE {
            let mut el = String::from_utf8(buffer[start..end].to_vec()).unwrap();
            if el.contains(".") {
                el = el.replace("\"", "");
                ips.push(el);
            }
        }
        1
    });
    buffer = Vec::new(); // Deallocate? https://bit.ly/3Jd2pml
    println!("Total ips: {:?}", ips.len());
    let chunks: Vec<Vec<String>> = ips.chunks(ips.len() / 8).map(|s| s.into()).collect();
    let mut handles = Vec::new();
    for chunk in chunks {
        let client_clone =  Arc::clone(&client);
        handles.push(tokio::spawn(async move {
            process_chunk(chunk, &*client_clone).await;
        }));
    }
    join_all(handles).await;
    Ok(())
}

async fn process_chunk(chunk: Vec<String>, client: &Client) {
    println!("Spawned thread");
    for ip in chunk.into_iter() {
        println!("{}", &ip);
        let connection = timeout(
            Duration::from_millis(300),
            ConnectionConfig::build(&ip).connect(),
        )
        .await;
        if connection.is_ok() && connection.as_ref().unwrap().is_ok() {
            let status = timeout(
                Duration::from_millis(300),
                connection.unwrap().unwrap().status(),
            )
            .await;
            if status.is_ok() && status.as_ref().unwrap().is_ok() {
                let status = &status.as_ref().unwrap().as_ref().unwrap().status;
                if let ServerDescription::Object {text: motd } = &status.description {
                    if status.players.sample.is_some() {
                        let player_sample: Vec<&str> = status.players.sample.as_ref().unwrap().into_iter().map(|player| player.id.as_str()).collect();
                        client.execute("INSERT INTO results (ip,motd, max_players, online_players, version_name, protocol_version, player_sample) VALUES ($1, $2, $3, $4, $5, $6, $7)", &[&ip, motd, &i64::from(status.players.max),  &i64::from(status.players.online), &status.version.name,  &i64::from(status.version.protocol), &player_sample]).await.expect("Error writing to database"); 
                    }
                    client.execute("INSERT INTO results (ip,motd, max_players, online_players, version_name, protocol_version) VALUES ($1, $2, $3, $4, $5, $6)", &[&ip, motd, &i64::from(status.players.max),  &i64::from(status.players.online), &status.version.name,  &i64::from(status.version.protocol)]).await.expect("Error writing to database"); 
                }
            }
        }
    }
}
