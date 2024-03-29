use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::{
    env,
    fs::File,
    io::{self, ErrorKind},
    path::{Path},
    string::String
};

#[derive(Debug, Serialize, Deserialize)]
struct City {
    name : String,
    code: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Country {
    name : String,
    flag: String,
    cities: Vec<City>
}

#[derive(Debug, Serialize, Deserialize)]
struct Weather {
    #[serde(rename = "Headline")]
    headline: Headline,
    #[serde(rename = "DailyForecasts")]
    daily_forecasts: Vec<DailyForecasts>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Headline {
    #[serde(rename = "Text")]
    text: String
}

#[derive(Debug, Serialize, Deserialize)]
struct DailyForecasts {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Day")]
    day: Day,
    #[serde(rename = "Night")]
    night: Day,
    #[serde(rename = "Temperature")]
    temperature : Temperature,
    #[serde(rename = "Link")]
    link: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Day {
    #[serde(rename = "Icon")]
    icon: i32,
    #[serde(rename = "IconPhrase")]
    icon_phrase : String
}

#[derive(Debug, Serialize, Deserialize)]
struct Temperature {
    #[serde(rename = "Minimum")]
    minimum: TemperatureValue,
    #[serde(rename = "Maximum")]
    maximum: TemperatureValue
}

#[derive(Debug, Serialize, Deserialize)]
struct TemperatureValue {
    #[serde(rename = "Value")]
    value: f32,
    #[serde(rename = "Unit")]
    unit: String,
    #[serde(rename = "UnitType")]
    unit_type: u32,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let key = &args[1];
    let path = &args[2];

    let countries: Result<Vec<Country>, io::Error> = get_list(&path);

    let now: DateTime<Utc> = Utc::now();

    for country in countries.unwrap() {
        println!("{} {}", country.name, country.flag);
        for city in country.cities {
            println!("{}", city.name);

            // This can not be ok. 
            // @todo investigate how format! can do the work here.
            let mut path = "share/".to_owned();
            path.push_str(&now.format("%Y").to_string());
            path.push_str("/");
            path.push_str(&now.format("%m").to_string());
            path.push_str("/");
            path.push_str(&now.format("%d").to_string());
            path.push_str("/");
            path.push_str(&city.code);
            path.push_str(".json");
            
            if  Path::new(&path).exists() {
                let file = File::open(path).expect("File should be readable.");
                let w: Weather = serde_json::from_reader(file).expect("Bad format of file.");
                print_the_weather(&w);
            } else {                
                if let Ok(w) = call_api(&city.code, &key).await {
                    print_the_weather(&w);
                    let destination = std::path::Path::new(&path);
                    let prefix = destination.parent().unwrap();
                    std::fs::create_dir_all(prefix).unwrap();
                    std::fs::write(
                        path.to_string(),
                        serde_json::to_string_pretty(&w).expect("msg"),
                    ).unwrap();
                } else {
                    println!("Something happend when calling API, Possible due to bad credentials.");
                }
            }
        }
        println!();
    }
}

fn print_the_weather(w: &Weather) {
    println!("{}", w.headline.text);
    println!("Máxima: {} C| Mínima: {} C",w.daily_forecasts[0].temperature.maximum.value, w.daily_forecasts[0].temperature.minimum.value);
    println!("Día {} ({}) | Noche {} ({})",
        convert_icon(w.daily_forecasts[0].day.icon),
        w.daily_forecasts[0].day.icon_phrase,
        convert_icon(w.daily_forecasts[0].night.icon),
            w.daily_forecasts[0].night.icon_phrase,
    );
    println!("Más información: {}", w.daily_forecasts[0].link);
    println!("***************************");
}

// Get all the city and countrie data outside of the program
fn get_list(path: &String) -> Result<Vec<Country>, io::Error> {

    let real_path = Path::new(path);

    if !real_path.is_file() {
        let not_a_file_error = io::Error::new(
            ErrorKind::InvalidInput,
            format!("Not a file: {}", real_path.display()),
        );
        return Err(not_a_file_error);
    }

    let file = File::open(path).expect("File should be readable.");
    let json: Vec<Country> = serde_json::from_reader(file).expect("Bad format of file.");
    Ok(json)
}

// This is the call to the api.
async fn call_api(code:&String, api_key:&String) -> Result<Weather, reqwest::Error> {

    let url = format!(
        "http://dataservice.accuweather.com/forecasts/v1/daily/1day/{}?apikey={}&language=es-ES&metric=true",
        code,
        api_key);

    let data = reqwest::get(url)
        .await?
        .json::<Weather>()
        .await?;

    Ok(data)
}

fn convert_icon(icon: i32) -> String {

    let final_icon =  match icon {
        1..=5 => ":soleado:",
        6 => ":casi_todo_soleado:",
        7..=8 => ":nube:",
        35..=38 => ":nube:",
        11 => ":nube:",
        22 => ":nube:",
        12 => ":nube_de_lluvia:",
        13..=14 => ":sol_tras_nubes_lluvia:",
        19 => ":nube_de_nieve:",
        15 => ":nube_de_truenos_y_lluvia:",
        20..=21 => ":sol_con_nubes:",
        23 => ":sol_con_nubes:",
        24 => ":cubito_de_hielo:",
        18 => ":nube_de_lluvia:",
        25 => ":nube_de_lluvia:",
        39..=40 => ":nube_de_lluvia:",
        16..=17 => ":nube_de_truenos_y_lluvia:",
        41..=42 => ":nube_de_truenos_y_lluvia:",
        26 => ":nube_de_nieve:",
        29 => ":nube_de_nieve:",
        30 => ":cara_con_calor:",
        31 => ":cara_con_frio:",
        32 => ":guión:",
        33..=34 => ":luna_llena_con_cara:",
        _=>""
    };

    return final_icon.to_string();
}

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn convert_icon_return_correctly() {
        let icon = convert_icon(4);
        let icon2 = convert_icon(36);
        assert_eq!(icon, ":soleado:");
        assert_eq!(icon2, ":nube:");
    }

    #[test]
    fn throw_error_if_file_path_is_incorrect() {
        let path = String::from("/what/is/this/route");
        let result = get_list(&path);
        assert!(result.is_err());
    }

    #[test]
    fn check_path() {
        let now: DateTime<Utc> = Utc::now();
        let mut path = "share/".to_owned();
            path.push_str(&now.format("%Y").to_string());
            path.push_str("/");
            path.push_str(&now.format("%a").to_string());
            path.push_str("/");
            path.push_str(&now.format("%d").to_string());
            path.push_str("/");
            path.push_str(".json");
        
        println!("{}", path.to_string());
    }
}
