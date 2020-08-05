extern crate reqwest;
extern crate select;

use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};

#[command]
fn decision(ctx: &mut Context, msg: &Message) -> CommandResult {
	
	let name_vec: Vec<String> = names((*msg.content).to_string());

	let fighter_1 = name_vec[0].to_owned() + " " + &name_vec[1];

	let fighter_2 = name_vec[3].to_owned() + " " + &name_vec[4];

	let f1_url = fighter_url(fighter_1);
	let f2_url = fighter_url(fighter_2);
	println!("F1: {}", f1_url);
	println!("F2: {}", f2_url);
    let _ = msg.channel_id.say(&ctx.http, "Decision");

    Ok(())
}

fn names(mut content: String) -> std::vec::Vec<String> {
	
	//Used to remove the box command prefix
	for _n in 0..10{
		content.remove(0);
	}

	//Splits all the input, using whitespace as a delimiter
	let name_split = content.split_whitespace();
	//Collects all input into a vector
	let name_vec: Vec<String> = name_split.map(str::to_string).collect();

	println!("{:?}", name_vec);

	return name_vec;
}

fn fighter_url(fighter: String) -> String{

	//Sets up http client
	let client = reqwest::blocking::Client::new();

	//Base search url
	let origin_url = "http://www.mmadecisions.com/search?s=";
	
	//Search url for the fighter user specified
	let fighter_se = origin_url.to_owned() + &fighter.replace(" ", "+");
	let res = client.get(&fighter_se).send().unwrap();

	//Returns status for thge requested page
	println!("Status for {}: {}", &fighter_se, res.status());

	let webpage = Document::from_read(res).unwrap();

	//Webpage scrape for all linls
	let results: Vec<_> = webpage
							.find(Name("a"))
							.filter_map(|n| n.attr("href"))
							.collect();

	let mut f_url = String::new();
	for n in results{
		if n.starts_with("fighter"){
			f_url = "http://www.mmadecisions.com/".to_owned() + &n.to_string();
		}
	}
	return f_url;
}