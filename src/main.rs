extern crate satellite;
extern crate chrono;
use chrono::prelude::*;
use chrono::Duration;
use space_plot::Color;
use rand::Rng;

fn main() {
    let station_longitude = std::env::var("STATION_LONGITUDE").unwrap_or("0".to_string()).parse::<f64>().unwrap();
    let station_latitude = std::env::var("STATION_LATITUDE").unwrap_or("0".to_string()).parse::<f64>().unwrap();
    let station_altitude = std::env::var("STATION_ALTITUDE").unwrap_or("0".to_string()).parse::<f64>().unwrap();

    println!("Fetching TLE from space-track...");
    let colors = [
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::White
    ];

    let client = reqwest::Client::new();
    let params = [
        ("identity", std::env::var("SPACETRACK_IDENTITY").unwrap()),
        ("password", std::env::var("SPACETRACK_SECRET").unwrap()),
        ("query", "https://www.space-track.org/basicspacedata/query/class/tle_latest/ORDINAL/1/NORAD_CAT_ID/25544/orderby/TLE_LINE1%20ASC/format/3le".to_string())
    ];

    let mut additional_points: Vec<space_plot::Point> = vec![];
    let mut hourly_track: Vec<space_plot::Point> = vec![];

    let mut tles : Vec<String> = vec![];
    // Add TLE sources to this list.
    // An example for space-track and authentication is posted above. 
    // tles.push(client.post("https://www.space-track.org/ajaxauth/login").form(&params).send().unwrap().text().unwrap());
    // tles.push(reqwest::get("https://www.celestrak.com/NORAD/elements/tdrss.txt").unwrap().text().unwrap());
    // tles.push(reqwest::get("https://www.celestrak.com/NORAD/elements/2019-006.txt").unwrap().text().unwrap());

    let mut combined = String::new();
    for tle in tles {
        combined.push_str(&tle);
    }

    println!("Performing orbital calculations...");
    // println!("`{}`", tle);
    // let mut satrecs = vec![satellite::io::parse(&tle).unwrap()];
    let (mut satrecs, errors) = satellite::io::parse_multiple(&combined);
    println!("succsfully parsed {} recs, failed {} recs", satrecs.len(), errors.len());
    // let mut satrecs = satellite::io::parse_multiple(&tles).unwrap();
    // let mut satrecs = vec![ satellite::io::parse_multiple(&tles).unwrap()[0].clone() ];

    let mut rng = rand::thread_rng();
    satrecs.reverse();

    for mut satrec in satrecs {
        let rec = satrec.clone();

        let time = Utc::now();
        let result = match satellite::propogation::propogate_datetime(&mut satrec, time) {
                Ok(result) => result,
                Err(e) => {
                    println!("Failed to propgate: {:?}", e);
                    continue
                }
            };

        if let Some(name) = satrec.name {
            println!("Name: {}", name)
        }
        println!("Position {:#?}", result.position);
        println!("Velocity {:#?}", result.velocity);
        
        let observer = satellite::Geodedic {
            longitude: station_longitude * satellite::constants::DEG_2_RAD,
            latitude: station_latitude * satellite::constants::DEG_2_RAD,
            height: station_altitude
        };


        // Perform calculations to find current position
        let gmst = satellite::propogation::gstime::gstime_datetime(time);
        let sat_pos = satellite::transforms::eci_to_geodedic(&result.position, gmst);
        let position_ecf = satellite::transforms::eci_to_ecf(&result.position, 0.0);
        let look_angles = satellite::transforms::ecf_to_look_angles(&observer, &position_ecf);

        println!("longitude = {}", sat_pos.longitude * satellite::constants::RAD_TO_DEG);
        println!("latitude = {}", sat_pos.latitude * satellite::constants::RAD_TO_DEG);
        println!("alt = {}", sat_pos.height * satellite::constants::KM_TO_MI);
        println!("aizmuth = {}", look_angles.azimuth * satellite::constants::RAD_TO_DEG);
        println!("elevation = {}", look_angles.elevation * satellite::constants::RAD_TO_DEG);
        println!("range = {}", look_angles.range * satellite::constants::KM_TO_MI);

        let track_color = rng.choose(&colors).unwrap();
        additional_points.push(space_plot::Point {
            x: sat_pos.longitude * satellite::constants::RAD_TO_DEG,
            y: sat_pos.latitude * satellite::constants::RAD_TO_DEG,
            color: Some(*track_color),
            name: rec.name.clone(),
            point: Some("+".to_string())
        });

        // Print hourly track.
        let track_duration : i64 = 20;
        for i in 0..track_duration {
            // println!("{:?}: {}", rec.name, i );
            let time = Utc::now() + Duration::minutes(i - track_duration / 2);
            let result = match satellite::propogation::propogate_datetime(&mut rec.clone(), time) {
                Ok(result) => result,
                Err(e) => {
                    println!("Failed to propgate track: {:?}", e);
                    continue
                }
            };

            // Set the Observer at 122.03 West by 36.96 North, in RADIANS
            let observer = satellite::Geodedic {
                longitude: station_longitude * satellite::constants::DEG_2_RAD,
                latitude: station_latitude * satellite::constants::DEG_2_RAD,
                height: station_altitude
            };

            let gmst = satellite::propogation::gstime::gstime_datetime(time);
            let sat_pos = satellite::transforms::eci_to_geodedic(&result.position, gmst);

            // println!("latitude: {}\nlongitude: {}", sat_pos.longitude * satellite::constants::RAD_TO_DEG, sat_pos.latitude * satellite::constants::RAD_TO_DEG);

            hourly_track.push(space_plot::Point {
                x:  sat_pos.longitude * satellite::constants::RAD_TO_DEG,
                y: sat_pos.latitude * satellite::constants::RAD_TO_DEG,
                name: None,
                color: Some(*track_color),
                point: Some("*".to_string())
            });
        }
    }

    // Add ground-station and current position points.
    additional_points.push(space_plot::Point {
        x: station_longitude,
        y: station_latitude,
        color: Some(Color::Red),
        name: Some("GND".to_string()),
        point: Some("+".to_string())
    });

    hourly_track.append(&mut additional_points);

    println!("{}", space_plot::render_point(hourly_track, space_plot::Plot::make_blank(200, 60)));
}