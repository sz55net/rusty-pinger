use std::{time::Duration};

use async_minecraft_ping::{ConnectionConfig, ServerDescription};
use futures::future::join_all;
// use s3::{creds::Credentials, Bucket, Region};
// use sha1::{Digest, Sha1};
use tokio::{fs::File, io::AsyncReadExt, time::timeout};
// use tokio_postgres::{Client, NoTls};
// use uuid::Uuid;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // let s3_region: Region = Region::Custom {
    //     region: "storj-gateway".to_owned(),
    //     endpoint: "https://gateway.storjshare.io".to_owned(),
    // };
    // let s3_credentials: Credentials = Credentials {
    //     access_key: Some("jvlj23nmsoe5wtlgunes7c37fjka".to_string()),
    //     secret_key: Some("j3lumlcb5jsu54hwax6idaurud3jztumxhhzleovqvp3lbjuwtt3w".to_string()),
    //     security_token: None,
    //     session_token: None,
    //     expiration: None,
    // };
    // let bucket = Arc::new(Bucket::new("minecraft-icons", s3_region, s3_credentials).unwrap());
    let mut ips: Vec<String> = Vec::new();
    let mut buffer = Vec::new();

    // let (client, connection) = tokio_postgres::connect(
    //     "host=/var/run/postgresql user=george password=ab34EF&*cabbage21 dbname=george",
    //     NoTls,
    // )
    // .await
    // .unwrap();
    // let client = Arc::new(client);
    // tokio::spawn(async move {
    //     if let Err(e) = connection.await {
    //         eprintln!("connection error: {}", e);
    //     }
    // });

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
    let chunks: Vec<Vec<String>> = ips.chunks(ips.len() / 4096).map(|s| s.into()).collect();
    let mut handles = Vec::new();
    for chunk in chunks {
        // let client_clone = Arc::clone(&client);
        // let bucket_clone = Arc::clone(&bucket);
        handles.push(tokio::spawn(async move {
            process_chunk(chunk).await;
        }));
    }
    join_all(handles).await;
    Ok(())
}

async fn process_chunk(chunk: Vec<String>) {
    println!("Spawned thread");
    for ip in chunk.into_iter() {
        // println!("{}", &ip);
        let connection = timeout(
            Duration::from_millis(1000),
            ConnectionConfig::build(&ip).connect(),
        )
        .await;
        if connection.is_ok() && connection.as_ref().unwrap().is_ok() {
            let status = timeout(
                Duration::from_millis(2000),
                connection.unwrap().unwrap().status(),
            )
            .await;
            if status.is_ok() && status.as_ref().unwrap().is_ok() {
                let status = &status.as_ref().unwrap().as_ref().unwrap().status;
                if let ServerDescription::Object { text: motd } = &status.description {
                    if motd.contains("LiveOverflow") {
                        println!("IP: {:#?}\nStatus: {:#?}\n\n", &ip, &status);
                    }
                    // if !motd.contains("tcpshield") {
                    //     if status.players.sample.is_some() {
                    //         let player_sample: Vec<Uuid> = status
                    //             .players
                    //             .sample
                    //             .as_ref()
                    //             .unwrap()
                    //             .into_iter()
                    //             .map(|player| Uuid::parse_str(&player.id).unwrap())
                    //             .collect();
                    //         client.execute("INSERT INTO results (ip,motd, max_players, online_players, version_name, protocol_version, player_sample) VALUES ($1, $2, $3, $4, $5, $6, $7)", &[&ip, motd, &i64::from(status.players.max),  &i64::from(status.players.online), &status.version.name,  &i64::from(status.version.protocol), &player_sample]).await.expect("Error writing to database");
                    //     } else {
                    //         client.execute("INSERT INTO results (ip,motd, max_players, online_players, version_name, protocol_version) VALUES ($1, $2, $3, $4, $5, $6)", &[&ip, motd, &i64::from(status.players.max),  &i64::from(status.players.online), &status.version.name,  &i64::from(status.version.protocol)]).await.expect("Error writing to database");
                    //     }
                    }
                // if status.favicon.is_some() {
                //     let mut hasher = Sha1::new();
                //     let url = DataUrl::process(status.favicon.as_ref().unwrap()).unwrap();
                //     let (vec_favicon, _) = &url.decode_to_vec().unwrap();
                //     hasher.update(vec_favicon);
                //     let vec_2 = &vec_favicon;
                //     let file_path = hex::encode(hasher.finalize()) + ".png";
                //     bucket
                //         .put_object(&file_path, &vec_2)
                //         .await
                //         .expect("Favicon upload failure");
                //     client
                //         .execute(
                //             "UPDATE results SET favicon=$1 WHERE ip=$2",
                //             &[&file_path, &ip],
                //         )
                //         .await
                //         .unwrap();
                // }
            }
        }
    }
}
