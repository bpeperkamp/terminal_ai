use crossterm::{
    cursor::{Hide, MoveTo},
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use dialoguer::{theme::ColorfulTheme, BasicHistory, Input};
use dotenv::dotenv;
use reqwest::Error;
use serde::Deserialize;
use std::{
    collections::HashMap,
    io::{stdout, Write},
    process,
};

fn main() {
    clear_screen();

    Prompt::new();
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Choices {
    index: u32,
    finish_reason: String,
    message: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Answer {
    id: String,
    model: String,
    choices: Vec<Choices>,
}

#[derive(serde::Serialize)]
struct Question {
    model: String,
    messages: Vec<Messages>,
}

#[derive(serde::Serialize)]
struct Messages {
    role: String,
    content: String,
}

struct Prompt();

impl Prompt {
    fn new() {
        Prompt::show_prompt();
    }

    fn show_prompt() {
        let mut history = BasicHistory::new().no_duplicates(false);

        loop {
            if let Ok(question) = Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Ask anything")
                .history_with(&mut history)
                .interact_text()
            {
                if question == "exit" {
                    process::exit(0);
                }
                let _ = Prompt::ask(question);
            }
        }
    }

    fn ask(ask_question: String) -> Result<(), Error> {
        dotenv().ok();

        let bearer_token = std::env::var("API_KEY").expect("API_KEY must be set.");
        let base_url = std::env::var("API_URL").expect("API_URL must be set.");

        let question = Question {
            model: "llama3-8b-8192".to_string(),
            messages: vec![Messages {
                role: "user".to_owned(),
                content: ask_question.to_owned(),
            }],
        };
        let json = serde_json::to_string(&question);

        let client: reqwest::blocking::Client = reqwest::blocking::Client::new();
        let response = client
            .post(base_url)
            .body(json.unwrap())
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer ") + &bearer_token)
            .send()?;

        let maybe_err = response.error_for_status_ref().err();

        let answer = serde_json::from_str::<Answer>(&response.text()?).unwrap();

        for answer in answer.choices {
            if answer.message.contains_key("content") {
                let text = answer.message.get("content").unwrap();
                println!("{}", "");
                println!("{}", text);
                println!("{}", "");
            }
        }

        match maybe_err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

pub fn clear_screen() {
    let mut out = stdout();
    out.queue(Hide).unwrap();
    out.queue(Clear(ClearType::All)).unwrap();
    out.queue(MoveTo(0, 0)).unwrap();
    out.flush().unwrap();
}
