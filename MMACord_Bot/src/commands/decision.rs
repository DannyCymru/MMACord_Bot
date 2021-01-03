extern crate reqwest;
extern crate select;
extern crate serenity;

use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};
use select::document::Document;
use select::predicate::{Class, Name};

#[command]
fn decision(ctx: &mut Context, msg: &Message) -> CommandResult {
	
	let name_vec: Vec<String> = ui_names((*msg.content).to_string());

	//Check to make sure that you insert the correct amount of inputs
	if name_vec.len() == 5{
		let f1 = (name_vec[0].to_owned() + " " + &name_vec[1]).to_lowercase();

		let f2 = (name_vec[3].to_owned() + " " + &name_vec[4]).to_lowercase();

		let fight_search = fight_url(f1.clone(), f2.clone());

		if fight_search == "Fight does not exist"{
			let _ = msg.channel_id.say(&ctx.http, "Fight does not exist");
		}

		else {

			let fight_data: (Vec<String>, 
							Vec<String>,
							Vec<String>, 
							Vec<i32>, 
							Vec<i32>) = fight_scrape(fight_search);

		let fight = f1.to_owned() + " vs " + &f2;
		embed(ctx, msg, fight, fight_data);
		}


	}
	else {
		 let _ = msg.channel_id.say(&ctx.http, "You must only enter a fighters first and second name");
	}


    Ok(())
}

//Grabs the names from the user input
fn ui_names(mut content: String) -> std::vec::Vec<String> {
	
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

	for mut n in f_links {

		n = n.to_lowercase();

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
	
	let result: String;

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


fn fight_scrape(fight_url: String) -> (Vec<String>, Vec<String>, Vec<String>, Vec<i32>, Vec<i32>){
	
	//Sets up http client
	let client = reqwest::blocking::Client::new();

	let fight_url = "http://www.mmadecisions.com/".to_owned() + &fight_url;
	
	let res = client.get(&fight_url).send().unwrap();

	let webpage = Document::from_read(res).unwrap();

	let mut judge_name: Vec<String> = Vec::new();
	let mut fighter_names: Vec<String> = Vec::new();
	let mut r_data: Vec<i32> = Vec::new();
	let mut s_data: Vec<i32> = Vec::new();

	let judge: Vec<String> = webpage.find(Class("judge")).map(|n| n.text()).collect();
	
	for n in judge{
		let temp_judge = n.split_whitespace();
		for i in temp_judge{
			judge_name.push(i.to_string());
		}
	}

	for n in webpage.find(Class("top-cell")){
		let results: Vec<String> = n.find(Name("b")).map(|n| n.text()).collect();
		let temp = name_scrape(results);
		for i in temp{
			fighter_names.push(i);
		}
	}

	let mut offic_dec: Vec<String> = Vec::new();

	for n in webpage.find(Name("tr")){
		let results: Vec<String> = n.find(Class("event2")).map(|n| n.text()).collect();
		for i in results{
			offic_dec.push(i);
		}
	}
	//Scrapes for the information we would like
	for n in webpage.find(Class("decision")){
		
		let mut results: Vec<String> = n.find(Class("list")).map(|n| n.text()).collect();
		let mut temp: Vec<i32> = round_scrape(data_check(&mut results));
		
		for n in temp{
			r_data.push(n);
		}

		temp = score_scrape(data_check(&mut results));

		for n in temp{
			s_data.push(n);
		}
	}

	 return (judge_name, fighter_names, offic_dec, r_data, s_data)
}

//checks the webpage results so we can 
fn data_check(scrape_data: &mut Vec<String>) -> Vec<String>{
	
	let mut good_data: Vec<String> = Vec::new();

	//Checks scraped results
	for x in scrape_data{
		//or if its a long string of characters
		if x.len() > 2 || x.len() < 1 || x.is_empty() {
		}

		else if x.len() <= 2 {
			let data = x.to_string();
			good_data.push(data);
		}
	}
	return good_data;
}

//Returns just the round data from the fight page
fn round_scrape(scrape_data :  Vec<String>) ->  Vec<i32> {
	
	let mut round: Vec<i32> = Vec::new();

	for n in scrape_data	{
		let int_n = n.parse::<i32>().unwrap();

		if int_n >= 1 && int_n < 6 && int_n != 0{
			round.push(int_n);
		}
	}
	return round;
}

//Returns just the wanted scores and values from the fight page
fn score_scrape(scrape_data: Vec<String>) ->  Vec<i32>{
	
	let mut score: Vec<i32> = Vec::new();

	for n in scrape_data.clone(){
		
		let int_n = n.parse::<i32>().unwrap();
		
		if int_n > 5 {
			score.push(int_n);
		}
	}

	return score;
}

fn name_scrape(scrape_data: Vec<String>) -> Vec<String>{

	let mut names: Vec<String> = Vec::new();

	for n in scrape_data.clone(){

		if !n.contains(" "){ 
			names.push(n); 
		}
	}
	return names
}

fn embed(ctx: &mut Context, msg: &Message, fight: String, fight_data: (Vec<String>, Vec<String>, Vec<String>, Vec<i32>, Vec<i32>)) {
		
	let judge_1 = fight_data.0[0].clone() + " " + &fight_data.0[1];
	let judge_2 = fight_data.0[2].clone() + " " + &fight_data.0[3];
	let judge_3 = fight_data.0[4].clone() + " " + &fight_data.0[5];
	let scraped_fight = fight_data.1[0].clone() + " defeats " + &fight_data.1[1];
	let offic_dec = fight_data.2[0].clone();

	let mut j1_rounds = String::new();
	let mut j2_rounds = String::new();
	let mut j3_rounds = String::new();

	if fight_data.3.iter().count() % 3 == 0{
		let j1_total = (fight_data.4[0] + fight_data.4[2] + fight_data.4[4]).to_string() + "-" + &(fight_data.4[1] + fight_data.4[3] + fight_data.4[5]).to_string();
		let j2_total = (fight_data.4[6] + fight_data.4[8] + fight_data.4[10]).to_string() + "-" + &(fight_data.4[7] + fight_data.4[9] + fight_data.4[11]).to_string();
		let j3_total = (fight_data.4[12] + fight_data.4[14] + fight_data.4[16]).to_string() + "-" + &(fight_data.4[13] + fight_data.4[15] + fight_data.4[17]).to_string();

		j1_rounds = "R".to_owned()+ &fight_data.3[0].to_string() +": " + &fight_data.4[0].to_string() +"-" + &fight_data.4[1].to_string()+ "
					R" + &fight_data.3[1].to_string() + ": " + &fight_data.4[2].to_string() +"-" + &fight_data.4[3].to_string()+" 
					R" + &fight_data.3[2].to_string() + ": " + &fight_data.4[4].to_string() +"-" + &fight_data.4[5].to_string()+"
					T: " + &j1_total;

		j2_rounds = "R".to_owned()+ &fight_data.3[0].to_string() +": " + &fight_data.4[6].to_string() +"-" + &fight_data.4[7].to_string()+ "
					R" + &fight_data.3[1].to_string() + ": " + &fight_data.4[8].to_string() +"-" + &fight_data.4[9].to_string()+" 
					R" + &fight_data.3[2].to_string() + ": " + &fight_data.4[10].to_string() +"-" + &fight_data.4[11].to_string()+"
					T: " + &j2_total;
		
		j3_rounds = "R".to_owned()+ &fight_data.3[0].to_string() +": " + &fight_data.4[0].to_string() +"-" + &fight_data.4[1].to_string()+ "
					R" + &fight_data.3[1].to_string() + ": " + &fight_data.4[2].to_string() +"-" + &fight_data.4[3].to_string()+" 
					R" + &fight_data.3[2].to_string() + ": " + &fight_data.4[4].to_string() +"-" + &fight_data.4[5].to_string()+"
					T: " + &j3_total;					
	}

	else if fight_data.3.iter().count() % 5 == 0 {
		let j1_total = (fight_data.4[0] + fight_data.4[2] + fight_data.4[4] + fight_data.4[6] + fight_data.4[8]).to_string() + 
						"-" + 
						&(fight_data.4[1] + fight_data.4[3] + fight_data.4[5] +fight_data.4[7] + fight_data.4[9]).to_string();

		let j2_total = (fight_data.4[10] + fight_data.4[12] + fight_data.4[14] + fight_data.4[16] + fight_data.4[18]).to_string() + 
						"-" + 
						&(fight_data.4[11] + fight_data.4[13] + fight_data.4[15] +fight_data.4[17] + fight_data.4[19]).to_string();

		let j3_total = (fight_data.4[20] + fight_data.4[22] + fight_data.4[24] + fight_data.4[26] + fight_data.4[28]).to_string() + 
						"-" + 
						&(fight_data.4[21] + fight_data.4[23] + fight_data.4[25] +fight_data.4[27] + fight_data.4[29]).to_string();


		j1_rounds = "R".to_owned()+ &fight_data.3[0].to_string() +": " + &fight_data.4[0].to_string() +"-" + &fight_data.4[1].to_string()+ "
					R" + &fight_data.3[1].to_string() + ": " + &fight_data.4[2].to_string() +"-" + &fight_data.4[3].to_string()+" 
					R" + &fight_data.3[2].to_string() + ": " + &fight_data.4[4].to_string() +"-" + &fight_data.4[5].to_string()+"
					R" + &fight_data.3[3].to_string() + ": " + &fight_data.4[6].to_string() +"-" + &fight_data.4[7].to_string()+" 
					R" + &fight_data.3[4].to_string() + ": " + &fight_data.4[8].to_string() +"-" + &fight_data.4[9].to_string()+"
					T: " + &j1_total;

		j2_rounds = "R".to_owned()+ &fight_data.3[0].to_string() +": " + &fight_data.4[10].to_string() +"-" + &fight_data.4[11].to_string()+ "
					R" + &fight_data.3[1].to_string() + ": " + &fight_data.4[12].to_string() +"-" + &fight_data.4[13].to_string()+" 
					R" + &fight_data.3[2].to_string() + ": " + &fight_data.4[14].to_string() +"-" + &fight_data.4[15].to_string()+"
					R" + &fight_data.3[3].to_string() + ": " + &fight_data.4[16].to_string() +"-" + &fight_data.4[17].to_string()+" 
					R" + &fight_data.3[4].to_string() + ": " + &fight_data.4[18].to_string() +"-" + &fight_data.4[19].to_string()+"
					T: " + &j2_total;

		j3_rounds = "R".to_owned()+ &fight_data.3[0].to_string() + ": " + &fight_data.4[20].to_string() +"-" + &fight_data.4[21].to_string()+ "
					R" + &fight_data.3[1].to_string() + ": " + &fight_data.4[22].to_string() +"-" + &fight_data.4[23].to_string()+" 
					R" + &fight_data.3[2].to_string() + ": " + &fight_data.4[24].to_string() +"-" + &fight_data.4[25].to_string()+"
					R" + &fight_data.3[3].to_string() + ": " + &fight_data.4[26].to_string() +"-" + &fight_data.4[27].to_string()+" 
					R" + &fight_data.3[4].to_string() + ": " + &fight_data.4[28].to_string() +"-" + &fight_data.4[29].to_string()+"
					T: " + &j3_total;
	}

		let _ = msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(fight.to_uppercase());
                    e.thumbnail("http://www.mmadecisions.com/resources/img/mmadlogo.png");
                    e.description("From MMADecisions.com");
                    e.fields(vec![
                    	(&scraped_fight, &offic_dec, false),
                        (&judge_1, &j1_rounds, true),
                        (&judge_2, &j2_rounds, true),
                        (&judge_3, &j3_rounds, true)
                    ]);

                    e
                })
		});	

}