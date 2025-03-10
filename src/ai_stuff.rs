use rig::{
    agent::Agent,
    completion::Prompt,
    providers::gemini::{self, completion::CompletionModel},
};
use tokio::time::Instant;

async fn gemini_test() -> Result<(), Box<dyn std::error::Error>> {
    let model = get_translation_model();
    let prompt = "";
    let start = Instant::now();
    let response = model.prompt(prompt).await?;
    let time = start.elapsed();
    println!("Prompt:   {}", prompt);
    println!("Response: {} ({time:?})", response);

    Ok(())
}

pub fn get_translation_model() -> Agent<CompletionModel> {
    let client = gemini::Client::from_env();
    client.agent("gemini-2.0-flash-lite").preamble(
    "You are an assistant here to translate words or short sentences between Dutch and English.
    Follow the following instructions closely:
    1) Translate the user's input correctly.
    2) Only return the result of the translation back to the user.
    If the user inputs 'cat' you return 'kat'.
    If the user inputs 'aardappel' you return 'potato'.
    If the user inputs 'consider this' you return 'overweeg het volgende'.").build()
}
pub fn get_assistant_model(enable_thinking: bool) -> Agent<CompletionModel> {
    let client = gemini::Client::from_env();
    client
        .agent(if enable_thinking { "gemini-2.0-flash-thinking-exp-01-21" } else { "gemini-2.0-flash-lite" })
        .preamble(
            "You are an assistant that has control over the users keyboard.
            Your task is to give the text that the user requests for, this should not include any text formatting such as bold words or code blocks.
            The user can request many things, for example
            1) A specific word
            2) A code snippet
            3) A short sentence
            Here is an example of what you should output for a given input:
            The input 'word very sad' becomes 'depressed'.
            The input 'composer old austrian deaf' becomes 'Beethoven'.
            The input 'python quicksort oneliner' becomes 'q = lambda l: q([x for x in l[1:] if x <= l[0]]) + [l[0]] + q([x for x in l if x > l[0]]) if l else []'",
        )
        .build()
}
