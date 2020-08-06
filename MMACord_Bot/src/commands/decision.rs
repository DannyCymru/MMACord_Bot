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

		let fight_search = fight_url(f1, f2);

		if fight_search == "Fight does not exist"{
			let _ = msg.channel_id.say(&ctx.http, "Fight does not exist");
		}

		else {

			let round = fight_scrape(fight_search, "round".to_string());

			for x in round{
				println!("{:?}", x);
			}
		}		
		
	}
	else {
		 let _ = msg.channel_id.say(&ctx.http, "You must only enter a fighters first and second name");
	}


    Ok(())
}

//Grabs the names from the user input
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

//Scrapes web pages for links
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

//Obtains the fighters specific URL pages
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

//Loops through vectors to help determine if two fighters have fought
//and reduce code duplication
fn fight_loop(f_links: Vec<String>, f2: String) -> String{

	let mut result = String::new();

	for n in f_links {
		if n.contains(&f2.replace(" ", "-")){
			result = n.clone();
			println!("{:?}", n);
			break;
		}
		else{
			println!("They have not fought");

			result = "Fight does not exist".to_string();
		}
	}

	return result;
}

//Scrapes pages for a matching fight
fn fight_url(f1: String, f2: String) -> String{
	
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

	//If both fighters exist
	else if url_1 != failure && url_2 != failure {
		
		//Scrape the first fight
		let mut links = link_scrape(url_1);
		
		//Check if F1 and F2 have fought
		let check = fight_loop(links, f2);

		//If it fails, scrape F2's links and check for the fight
		if check == "Fight does not exist".to_string() {
			links = link_scrape(url_2);
			result = fight_loop(links, f1);
		}
		else{
			result = check;
		}
	}

	else {
		println!("complete failure");
		result = "Fight does not exist".to_string();
	}

	return result;
}


fn fight_scrape(fight_url: String, dtype: String ) -> Vec<i32>{
	
	//Sets up http client
	let client = reqwest::blocking::Client::new();

	let fight_url = "http://www.mmadecisions.com/".to_owned() + &fight_url;
	
	let res = client.get(&fight_url).send().unwrap();

	let webpage = Document::from_read(res).unwrap();

	let mut data: Vec<i32> = Vec::new();

	//Scrapes for the information we would like
	for n in webpage.find(Class("decision")){
		
		let mut results: Vec<String> = n.find(Class("list")).map(|n| n.text()).collect();
		
		if dtype == "round" {
			let new_data = round_scrape(data_check(&mut results));

			for x in new_data{
				data.push(x);
			}
		}

		else if dtype == "score"{
			data = score_scrape(data_check(&mut results));
		}
	}
	return data;
}

//checks the webpage results so we can 
fn data_check(scrape_data: &mut Vec<String>) -> Vec<String>{
	
	let mut good_data: Vec<String> = Vec::new();

	//Checks scraped results
	for x in scrape_data{
		//or if its a long string of characters
		if x.len() > 2 || x.len() < 1 || x.is_empty() {
			println!("This triggered data check if: {:?}", x);
		}

		else if x.len() <= 2 {
			let data = x.to_string();
			good_data.push(data);
		}
	}
	return good_data;
}

//Returns the round data from the fight page
fn round_scrape(scrape_data :  Vec<String>) ->  Vec<i32> {
	
	let mut round: Vec<i32> = Vec::new();

	println!("Scrape data: {:?}", scrape_data);
	for n in scrape_data	{
		let int_n = n.parse::<i32>().unwrap();

		if int_n >= 1 && int_n < 6{
			round.push(int_n);
		}

		else {
		}
	}

	return round;
}

//Returns the score from the fight page
fn score_scrape(scrape_data: Vec<String>) ->  Vec<i32>{
	
	let mut score: Vec<i32> = Vec::new();

	for n in scrape_data.clone(){
		
		let int_n = n.parse::<i32>().unwrap();
		
		if int_n > 5 {
			score.push(int_n);
		}

		else {
			println!("{:?}",scrape_data);
		}
	}

	println!("{}", score.len());
	return score;
}