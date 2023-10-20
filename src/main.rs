extern crate clap;
use clap::{App, Arg};


use reqwest::{self, Response};
use serde::{Deserialize, Serialize, Deserializer};
use std::process::exit;
use std::{io, fs, os};
use std::io::{BufRead, BufReader, BufWriter, Write};

#[derive(Debug, Deserialize)]
struct Anime {
    id: i64,
    name: String,
    #[serde(rename="type")]
    anime_type: String,
}
#[derive(Debug, Deserialize)]
struct Clip {
        id: i64,
        caption: String,
        link: String,
}   

#[derive(Debug, Deserialize)]
struct Clips {
    data: Vec<Clip>,
    currentPage: i64,
    totalPages: i64
}

#[derive(Debug, Deserialize)]
struct Data {
    data: Vec<Anime>,
}

async fn download_video(link: String) -> Result<String, reqwest::Error> {
    let mut r = reqwest::get(link).await?; 
    let filename = r.headers().get("content-disposition").unwrap().to_str().unwrap().split("filename=").nth(1).unwrap();


    let mut f = match fs::File::create(&filename) {
        Ok(file) => file,
        Err(_) => {
            fs::remove_file(&filename).ok(); 

            fs::File::create(&filename).expect("Failed to create file")
        },
    };

    while let Some(c) = r.chunk().await? {
        f.write_all(&c); 
    }

    f.flush(); 
    drop(f);

    Ok("".to_string())
}

fn print_clips(resp: &Clips) {
    println!("\nFound: {} | Current Page: {} | Total pages: {}\n-----------------------------------------------------------------------------------------------------------\n", resp.data.len(), resp.currentPage, resp.totalPages);
    resp.data.iter().for_each(|each| {
        println!("{}. {}", each.id, each.caption);
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new("Anime Climax CLI")
    .version("1.0")
    .author("Pichsereyvattana Chan")
    .about("A Rust CLI to help you get your favorite anime climax scene in seconds")
    .arg(
        Arg::with_name("search")
        .short("s")
        .long("search") // allow --name
        .takes_value(true)
        .help("Anime name to search")
        .required(false)
    ).arg(
        Arg::with_name("clips")
        .short("c")
        .long("clips")
        .takes_value(true)
        .required(false)
    )
    .arg(
        Arg::with_name("page")
        .short("p")
        .long("page")
        .takes_value(true)
        .required(false)
    ).arg(
        Arg::with_name("download_all_clips")
        .short("d")
        .long("download_all")
        .takes_value(true)
        .required(false)
    ).arg(
        Arg::with_name("single_clip")
        .long("sc")
        .takes_value(true)
        .conflicts_with_all(&["search", "clips", "page", "download_all"])
    )
    
    ;

    // .arg(
    //     Arg::with_name("verbose")
    //         .short("v")
    //         .long("verbose")
    //         .help("Enable verbose mode"),
    // )
    // .arg(
    //     Arg::with_name("force")
    //         .short("f")
    //         .long("force")
    //         .help("Force operation"),
    // )

    let matches = app.get_matches();
    
    let mut all_animes: Vec<Anime> = reqwest::get("https://anime-climax-api.onrender.com/anime/animes?all=true").await?.json().await?;
    
    if let Some(sc) = matches.value_of("single_clip") {
        let clip_metadata: Clip = reqwest::get(format!("https://anime-climax-api.onrender.com/clip/{}", sc)).await?.json().await?;
        download_video(clip_metadata.link).await;
        exit(0);
    }


    if let Some(s) = matches.value_of("search") {
        
        for anime in all_animes {
            if anime.name.to_lowercase().contains(&s.to_lowercase()) {
                println!("{}. {}", &anime.id, &anime.name);
            }
        }
        println!();
        exit(0);
    }

    if let Some(c) = matches.value_of("clips") {
        if let Some(p) = matches.value_of("page") {
            let resp: Clips = reqwest::get(format!("https://anime-climax-api.onrender.com/anime/{}/clips?page={}", &c, &p)).await?.json().await?;
            print_clips(&resp);
        } else {
            let resp: Clips = reqwest::get(format!("https://anime-climax-api.onrender.com/anime/{}/clips?page=1", &c)).await?.json().await?;
            print_clips(&resp);
        }

        exit(0);

    }

    if let Some(da) = matches.value_of("download_all_clips")  {
        if matches.is_present("page") {
            if let Some(p) = matches.value_of("page") {
                let clips: Clips= reqwest::get(format!("https://anime-climax-api.onrender.com/anime/{}/clips?page={}",  da, p)).await.unwrap().json().await.unwrap();
                let mut handles = Vec::new();
                for clip in clips.data {
                    // println!("{}. {}", &clip.id, &clip.caption);
    
                    let handle = tokio::spawn(download_video(clip.link.clone()));
                    handles.push(handle);
                }
                
                for handle in handles {
                    handle.await.unwrap();
                }

                exit(0);
            }

            

        }

        let mut current = 1 as i64;

        let clips: Clips= reqwest::get(format!("https://anime-climax-api.onrender.com/anime/{}/clips?page=1",  da)).await.unwrap().json().await.unwrap();
        let mut total_pages = clips.totalPages;
        
        while current <= total_pages {
            let clips: Clips= reqwest::get(format!("https://anime-climax-api.onrender.com/anime/{}/clips?page={}",  &da, &current)).await.unwrap().json().await.unwrap();
            let mut handles = Vec::new();
            for clip in clips.data {
                // println!("{}. {}", &clip.id, &clip.caption);

                let handle = tokio::spawn(download_video(clip.link.clone()));
                handles.push(handle);
            }
            
            for handle in handles {
                handle.await.unwrap();
            }

            current += 1;
        }
        // for clip in clips {
        //     println!("{}. {}", &clip.id, &clip.caption);
            
        //     download_video(&clip.link).await?; 
        // }
    }


      //     let resp: Data = reqwest::get("https://anime-climax-api.onrender.com/anime/animes?page=1")
    //     .await?
    //     .json()
    //     .await?;

    // let formated: Vec<String> = resp.data.iter().enumerate().map(|(idx, anime)| {
    //     format!("{}. {} ({})", idx, &anime.name, &anime.anime_type)
    // }).collect();

    // println!("{:#?}", formated);


    // let mut input = String::new();

    // std::io::stdin().read_line(&mut input).expect("error".into());

    // let choice = input.trim().parse::<i32>().unwrap();

    // println!("Choice {}", choice);
    
    // let clips: Clips = reqwest::get("https://anime-climax-api.onrender.com/anime/1/clips?page=1").await.unwrap().json().await.unwrap();


    // for clip in clips.data {
    //     println!("{}. {}", &clip.id, &clip.caption);
        
    //     download_video(&clip.link).await?; 
    // }

  
    Ok(())
}
