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

	let fighter_1 = name_vec[0].to_owned() + "+" + &name_vec[1];

	let fighter_2 = name_vec[3].to_owned() + "+" + &name_vec[4];

	fighter_search(fighter_1, fighter_2);

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

fn fighter_search(fighter_1: String, fighter_2: String){

	//Sets up http client
	let client = reqwest::blocking::Client::new();

	//Base search url
	let origin_url = "http://www.mmadecisions.com/search?s=";
	
	//Search url for the first fighter user specified
	let fighter_1_se = origin_url.to_owned() + &fighter_1;
	let res = client.get(&fighter_1_se).send().unwrap();

	//Checks the page status eg 200 or 404
	println!("Status for {}: {}", &fighter_1_se, res.status());

	//Webpage scrape
	Document::from_read(res)
		.unwrap()
		.find(Name("a"))
		.filter_map(|n| n.attr("href"))
		.for_each(|x| println!("{}", x));
}