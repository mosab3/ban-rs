// responsible for banning and unbanning IP's
// uses redis to store banned IP's and theres timestamps

use std::collections::HashMap;
use std::process::Command;
use std::io::{self, Write};
use std::net::IpAddr;
use std::str::FromStr;
use redis::Commands;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use tokio::time::{sleep, Duration};
use tokio::{sync, task};
use crate::watchers::Record;
use crate::conf::CONFIG;
use crate::helpers::is_red_hat_based;

pub async fn banner(record: Record, bantime: u64, main_rx: sync::mpsc::Receiver<Record>) {

    let timestamp = get_timestamp(&record.datetime).expect(
        format!("Failed to get timestamp from datetime: {}", record.datetime).as_str()
    );

    let config = &CONFIG;
    let redis_config = &config.redis;

    // redis://[<username>][:<password>@]<hostname>[:port][/[<db>][?protocol=<protocol>]]
    let url = format!(
        "redis://{}:{}@{}:{}/{}",
        redis_config.username,
        redis_config.password,
        redis_config.host,
        redis_config.port,
        redis_config.db
    );

    let client = redis::Client::open(url).unwrap();
    let mut con = redis::AsyncCommands::new(client.get_connection().unwrap());

    // let con_clone = client.get_connection()?;
    task::spawn(check_banned_ips(con));
    
    while let Some(record) =  main_rx.recv().await {
        if record.status_code.is_none() || record.status_code.unwrap().to_string().starts_with("4") {
            fail(&mut con, record);
        }
    }


    // match record.status_code {
    //     Some(code) => {
    //         if code.to_string().starts_with("4") {
    //             fail(&mut con, record);
    //         }
    //     },
    //     // if status code is not 4xx, it's mean that the request came from a SSH service
    //     None => {
    //         fail(&mut con, record);
    //     }
    // }

    // Ok(())
}

fn fail(con: &mut redis::Connection, record: Record) -> redis::RedisResult<()> {

    // let config = match &CONFIG. {

    // };

    // time between fail attempts in seconds
    let tbf = 10;

    let fail_key = format!("ban-rs::fail::{}", record.ip);
    let result: HashMap<String, String> = con.hgetall(&fail_key)?;
    if result.is_empty() {
        con.hset(&fail_key, "attempts", 1)?;
        con.hset(fail_key, "last_fail", record.datetime)?;
    } else {
        let mut attempts: u8 = result.get("attempts").unwrap().parse().unwrap();
        let last_fail: String = result.get("last_fail").unwrap().to_string();

        // new - old = diff

        let old_timestamp = get_timestamp(&last_fail).expect(
            &format!("Failed to get timestamp from datetime: {}", last_fail)
        );
        let new_timestamp = get_timestamp(&record.datetime).expect(
            &format!("Failed to get timestamp from datetime: {}", record.datetime)
        );
        let duration = new_timestamp - old_timestamp;

        
        if duration >= tbf {
            attempts += 1;
            con.hset(&fail_key, "attempts", attempts)?;
            ban(record.ip, con);
        }

        // TODO:
        //     1. Check if it has been equal or more than tbf seconds since the last fail, if so, add 1 to the number of attempts
        //     2. Check if the number of attempts is equal or more than the max number of attempts
        //     3. ban the ip
        //     4. register the ip as banned in redis `ban-rs::banned`(hash of banned ips and datetime of ban)
    }
        
        Ok(())

    // ban-rs::fail::<IP>{attempts: num, last_fail: time}

}

async fn check_banned_ips(con: &mut redis::Connection) {
    loop {
        sleep(Duration::from_secs(1)).await;
        // Get all banned IPs
        let ban_ips: Vec<String> = con.lrange("ban-rs::banned::ips", 0, -1).unwrap();

        for ip in ban_ips {
            let now_timestamp = Utc::now().timestamp();
            let ban_timestamp = con.hget(format!("ban-rs::banned::{}", ip), "timestamp").unwrap();
            let ban_duration = now_timestamp - ban_timestamp.parse::<i64>().unwrap();
            if ban_duration >= bantime {
                unban(ip, con);
            }
        }

    }
}

fn ban(ip: String, con: &mut redis::Connection) -> io::Result<()>{
    con.lpush("ban-rs::banned::ips", ip);

    con.hset(
        format!("ban-rs::banned::{}", ip),
        "timestamp",
        Utc::now().timestamp().to_string()
    );


    if is_red_hat_based() {
        let ip_type = match IpAddr::from_str(&ip).unwrap() {
            IpAddr::V4(_) => String::from("ipv4"),
            IpAddr::V6(_) => String::from("ipv6"),
        };
    
        let output = Command::new("firewall-cmd")
            .arg("--permanent")
            .arg("--add-rich-rule")
            .arg(
                format!(
                    "rule family={} source address={} reject",
                    ip_type, ip
                )
            )
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to add firewall rule: {:?}", output)));
        }

        let output = Command::new("firewall-cmd")
            .arg("--reload")
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to reload firewall: {:?}", output)));
        }
    } else {
        let output = Command::new("ufw")
            .arg("deny")
            .arg("from")
            .arg(ip)
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to add ufw rule: {:?}", output)));
        }
        
        let output = Command::new("ufw")
            .arg("reload")
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to reload ufw: {:?}", output)));
        }
    }

    Ok(())

}

fn unban(ip: String, con: &mut redis::Connection) -> io::Result<()>{

    con.lrem("ban-rs::banned::ips", 0, ip)?;

    con.del(
        format!("ban-rs::banned::{}", ip),
    )?;

    if is_red_hat_based() {
        let ip_type = match IpAddr::from_str(&ip).unwrap() {
            IpAddr::V4(_) => String::from("ipv4"),
            IpAddr::V6(_) => String::from("ipv6"),
        };
    
        let output = Command::new("firewall-cmd")
            .arg("--permanent")
            .arg("--remove-rich-rule")
            .arg(
                format!(
                    "rule family={} source address={} reject",
                    ip_type, ip
                )
            )
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to remove firewall rule: {:?}", output)));
        }

        let output = Command::new("firewall-cmd")
            .arg("--reload")
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to reload firewall: {:?}", output)));
        }
    } else {
        let output = Command::new("ufw")
            .arg("delete")
            .arg("deny")
            .arg("from")
            .arg(ip)
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to remove ufw rule: {:?}", output)));
        }
        
        let output = Command::new("ufw")
            .arg("reload")
            .output()?;

        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to reload ufw: {:?}", output)));
        }
    }

    Ok(())

}

/// Detect the type of datetime format and return a timestamp
fn get_timestamp(datetime: &str) -> Option<i64> {
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d",
        "%H:%M:%S",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%d",
        "%Y-%m-%dT%H:%M:%S%.fZ",
        "%Y-%m-%dT%H:%M:%S%.f%:z",
        "%Y-%m-%dT%H:%M:%S%:z",
    ];

    for &format in &formats {
        if let Ok(parsed) = DateTime::parse_from_str(datetime, format) {
            Some(parsed.timestamp());
        }
        if let Ok(parsed) = NaiveDateTime::parse_from_str(datetime, format) {
            // Assuming the NaiveDateTime is in UTC
            Some(Utc.from_utc_datetime(&parsed).timestamp());
        }
    }

    None

}

#[test]
fn test_timestamp() {
    let datetime = "2022-01-01 00:00:00 +3000";
    let ts = get_timestamp(datetime);
    println!("{:?}", ts);
}
