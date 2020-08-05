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

	//Check to make sure that you insert the correct amount of inputs
	if name_vec.len() == 5{
		let f1 = name_vec[0].to_owned() + " " + &name_vec[1];

		let f2 = name_vec[3].to_owned() + " " + &name_vec[4];

		let _fight_search = fight_check(f1, f2);

		let _ = msg.channel_id.say(&ctx.http, "fight");

	}
	else {
		 let _ = msg.channel_id.say(&ctx.http, "You must only enter a fighters first and second name");
	}


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

fn link_scrape(url: String) -> std::vec::Vec<String>{

	//Sets up http client
	let client = reqwest::blocking::Client::new();

	let res = client.get(&url).send().unwrap();

	println!("Status for {}: {}", &url, res.status());

	let webpage = Document::from_read(res).unwrap();

	//Webpage scrape for all links
	let results: Vec<String> = webpage
							.find(Name("a"))
							.filter_map(|n| n.attr("href"))
							.map(str::to_string)
							.collect();

	println!("{:?}", results);

	return results;
}

fn fighter_url(fighter: String) -> String{
	
	//Search url
	let url = "http://www.mmadecisions.com/search?s=".to_owned() + &fighter.replace(" ", "+");

	//Vector of links scraped
	let results = link_scrape(url);

	let mut result = String::new();

	//For loop to look through the results till we get a fighter page
	for n in results{
		if n.starts_with("fighter"){
			result = "http://www.mmadecisions.com/".to_owned() + &n.to_string();
			break;
		}
		else {
			result = "No fighter found".to_string();
		}
	}
	return result;
}

fn fight_check(f1: String, f2: String) -> String{
	
	//Urls for both fighters provided
	let url_1 = fighter_url(f1.clone());
	let url_2 = fighter_url(f2.clone());
	
	let mut result = String::new();

	//Failure state
	let failure = "No fighter found";

	//Checks if either URL scrape fails
	if url_1 == failure && url_2 == failure {
		result = "Fighters not found".to_string();
	}
	else if url_1 != failure && url_2 == failure {
		result = "Fighter ".to_owned() + &f2 + " not found";
		println!("{:?}", result);
	}

	else if url_2 != failure && url_1 == failure {
		result = "Fighter ".to_owned() + &f1 + " not found";
		println!("{:?}", result);
	}

	else{
		
		let results = link_scrape(url_1);

		//For loop to look through the results till we get a fighter page
		for n in results{
			//Returns the fight url
			if n.contains(&f2.replace(" ", "-")){
				result = n.clone();
				println!("{:?}", n);
				break;
			}
			else {
				result = 'f'.to_string();
				println!("Fail");
			}
		}
	}	
	return result;
}