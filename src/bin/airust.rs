// src/bin/airust.rs - Corrected unified CLI tool
use airust::agent::{Agent, ContextualAgent, ResponseFormat, TrainableAgent, TrainingExample};
use airust::context_agent::ContextAgent;
use airust::knowledge::KnowledgeBase;
use airust::match_agent::MatchAgent;
use airust::tfidf_agent::TfidfAgent;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

enum Command {
    Query(String),
    Interactive,
    Knowledge,
    Help,
}

enum AgentType {
    Simple,
    Fuzzy,
    TFIDF,
    Context,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let command = match args[1].as_str() {
        "query" => {
            if args.len() < 4 {
                eprintln!("Error: 'query' requires agent type and question");
                print_help();
                return;
            }
            let _agent_type = match args[2].as_str() {
                "simple" => AgentType::Simple,
                "fuzzy" => AgentType::Fuzzy,
                "tfidf" => AgentType::TFIDF,
                "context" => AgentType::Context,
                _ => {
                    eprintln!("Unknown agent type: {}", args[2]);
                    print_help();
                    return;
                }
            };

            let question = args[3..].join(" ");
            Command::Query(question)
        }
        "interactive" => Command::Interactive,
        "knowledge" => Command::Knowledge,
        "help" => Command::Help,
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_help();
            return;
        }
    };

    match command {
        Command::Query(question) => handle_query(&args[2], &question),
        Command::Interactive => run_interactive_mode(),
        Command::Knowledge => run_knowledge_management(),
        Command::Help => print_help(),
    }
}

fn print_help() {
    println!("airust - Modular AI Library in Rust");
    println!();
    println!("Usage:");
    println!("  airust query <agent> <question>   - Ask a question to an agent");
    println!("  airust interactive             - Start interactive mode");
    println!("  airust knowledge               - Knowledge base management");
    println!("  airust help                    - Show this help");
    println!();
    println!("Available agents:");
    println!("  simple   - Exact match");
    println!("  fuzzy    - Approximate match (Levenshtein)");
    println!("  tfidf    - BM25 algorithm for better matching");
    println!("  context  - Considers previous conversation");
}

fn handle_query(agent_type: &str, question: &str) {
    // Loads knowledge base
    let kb = KnowledgeBase::from_embedded();
    let examples = kb.get_examples();

    // Creates matching agent
    let answer = match agent_type {
        "simple" => {
            let mut agent = MatchAgent::new_exact();
            agent.train(examples);
            agent.predict(question)
        }
        "fuzzy" => {
            let mut agent = MatchAgent::new_fuzzy();
            agent.train(examples);
            agent.predict(question)
        }
        "tfidf" => {
            let mut agent = TfidfAgent::new();
            agent.train(examples);
            agent.predict(question)
        }
        "context" => {
            // In non-interactive mode there is no context
            let mut base_agent = TfidfAgent::new();
            base_agent.train(examples);
            let agent = ContextAgent::new(base_agent, 3);
            agent.predict(question)
        }
        _ => ResponseFormat::Text(format!("Unknown agent type: {}", agent_type)),
    };

    println!("Answer: {}", String::from(answer));
}

fn run_interactive_mode() {
    println!("=== Interactive Mode ===");
    println!("Select an agent type:");
    println!("1. Exact (SimpleAgent)");
    println!("2. Fuzzy (FuzzyAgent)");
    println!("3. TFIDF (TfidfAgent)");
    println!("4. Context (ContextAgent)");
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let choice = input.trim();

    // Loading knowledge base
    let kb = KnowledgeBase::from_embedded();
    let examples = kb.get_examples();

    match choice {
        "1" => interactive_loop_simple(examples),
        "2" => interactive_loop_fuzzy(examples),
        "3" => interactive_loop_tfidf(examples),
        "4" => interactive_loop_context(examples),
        _ => println!("Invalid selection. Please restart the program."),
    }
}

fn interactive_loop_simple(examples: &[TrainingExample]) {
    println!("=== Exact Matching Agent ===");
    println!("Enter questions or 'exit' to quit.");

    let mut agent = MatchAgent::new_exact();
    agent.train(examples);

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.to_lowercase() == "exit" {
            break;
        }

        let answer = agent.predict(input);
        println!("Answer: {}", String::from(answer.clone()));
    }
}

fn interactive_loop_fuzzy(examples: &[TrainingExample]) {
    println!("=== Fuzzy Matching Agent ===");
    println!("Enter questions or 'exit' to quit.");

    let mut agent = MatchAgent::new_fuzzy();
    agent.train(examples);

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.to_lowercase() == "exit" {
            break;
        }

        let answer = agent.predict(input);
        println!("Answer: {}", String::from(answer));
    }
}

fn interactive_loop_tfidf(examples: &[TrainingExample]) {
    println!("=== TFIDF Agent (BM25) ===");
    println!("Enter questions or 'exit' to quit.");

    let mut agent = TfidfAgent::new();
    agent.train(examples);

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.to_lowercase() == "exit" {
            break;
        }

        let answer = agent.predict(input);
        println!("Answer: {}", String::from(answer));
    }
}

fn interactive_loop_context(examples: &[TrainingExample]) {
    println!("=== Context Agent ===");
    println!("Enter questions or 'exit' to quit.");
    println!("The agent uses context from previous questions.");

    let mut base_agent = TfidfAgent::new();
    base_agent.train(examples);
    let mut agent = ContextAgent::new(base_agent, 3);

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.to_lowercase() == "exit" {
            break;
        }

        let answer = agent.predict(input);
        let answer_str = String::from(answer.clone());
        println!("Answer: {}", answer_str);
        agent.add_context(input.to_string(), answer);
    }
}

fn run_knowledge_management() {
    println!("=== Knowledge Base Management ===");
    println!("1. Create new knowledge base");
    println!("2. Load knowledge base");
    println!("3. Back");
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let choice = input.trim();

    match choice {
        "1" => create_knowledge_base(),
        "2" => load_knowledge_base(),
        _ => return,
    }
}

fn create_knowledge_base() {
    let mut kb = KnowledgeBase::new();

    println!("=== Create New Knowledge Base ===");
    println!("Enter examples. Press Enter without input to finish.");

    loop {
        println!("\nNew example:");
        print!("Question: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            break;
        }

        print!("Answer: ");
        io::stdout().flush().unwrap();

        let mut output = String::new();
        io::stdin().read_line(&mut output).unwrap();
        let output = output.trim();

        print!("Weight (Default 1.0): ");
        io::stdout().flush().unwrap();

        let mut weight_str = String::new();
        io::stdin().read_line(&mut weight_str).unwrap();
        let weight_str = weight_str.trim();

        let weight = if weight_str.is_empty() {
            1.0
        } else {
            weight_str.parse::<f32>().unwrap_or(1.0)
        };

        kb.add_example(
            input.to_string(),
            ResponseFormat::Text(output.to_string()),
            weight,
        );
        println!("Example added!");
    }

    println!("\nEnter path to save:");
    print!("> ");
    io::stdout().flush().unwrap();

    let mut path_str = String::new();
    io::stdin().read_line(&mut path_str).unwrap();
    let path_str = path_str.trim();

    if path_str.is_empty() {
        println!("No path specified. Aborting.");
        return;
    }

    let path = PathBuf::from(path_str);
    match kb.save(Some(path.clone())) {
        Ok(_) => {
            println!("Knowledge base saved to {:?}", path);
            println!("Would you like to test the knowledge base? (y/n)");
            print!("> ");
            io::stdout().flush().unwrap();

            let mut test = String::new();
            io::stdin().read_line(&mut test).unwrap();
            let test = test.trim();

            if test.to_lowercase() == "y" {
                test_knowledge_base(&kb);
            }
        }
        Err(e) => println!("Error saving: {}", e),
    }
}

fn load_knowledge_base() {
    println!("Enter path to knowledge base:");
    print!("> ");
    io::stdout().flush().unwrap();

    let mut path_str = String::new();
    io::stdin().read_line(&mut path_str).unwrap();
    let path_str = path_str.trim();

    if path_str.is_empty() {
        println!("No path specified. Aborting.");
        return;
    }

    let path = PathBuf::from(path_str);
    match KnowledgeBase::load(path) {
        Ok(kb) => {
            println!(
                "Knowledge base loaded! {} examples found.",
                kb.get_examples().len()
            );
            println!("\nWhat would you like to do?");
            println!("1. Test knowledge base");
            println!("2. Add examples");
            println!("3. Back");
            print!("> ");
            io::stdout().flush().unwrap();

            let mut choice = String::new();
            io::stdin().read_line(&mut choice).unwrap();
            let choice = choice.trim();

            match choice {
                "1" => test_knowledge_base(&kb),
                "2" => add_examples_to_kb(kb),
                _ => return,
            }
        }
        Err(e) => println!("Error loading: {}", e),
    }
}

fn test_knowledge_base(kb: &KnowledgeBase) {
    println!("=== Test Knowledge Base ===");
    println!("Select an agent type for testing:");
    println!("1. Exact (SimpleAgent)");
    println!("2. Fuzzy (FuzzyAgent)");
    println!("3. TFIDF (TfidfAgent)");
    print!("> ");
    io::stdout().flush().unwrap();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    let choice = choice.trim();

    let examples = kb.get_examples();

    match choice {
        "1" => {
            let mut agent = MatchAgent::new_exact();
            agent.train(examples);
            test_loop(&agent)
        }
        "2" => {
            let mut agent = MatchAgent::new_fuzzy();
            agent.train(examples);
            test_loop(&agent)
        }
        "3" => {
            let mut agent = TfidfAgent::new();
            agent.train(examples);
            test_loop(&agent)
        }
        _ => println!("Invalid selection."),
    }
}

fn test_loop(agent: &impl Agent) {
    println!("Ask questions or enter 'exit' to quit.");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.to_lowercase() == "exit" {
            break;
        }

        let answer = agent.predict(input);
        println!("Answer: {}", String::from(answer));
    }
}

fn add_examples_to_kb(mut kb: KnowledgeBase) {
    println!("=== Add Examples ===");
    println!("Enter examples. Press Enter without input to finish.");

    loop {
        println!("\nNew example:");
        print!("Question: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            break;
        }

        print!("Answer: ");
        io::stdout().flush().unwrap();

        let mut output = String::new();
        io::stdin().read_line(&mut output).unwrap();
        let output = output.trim();

        print!("Weight (Default 1.0): ");
        io::stdout().flush().unwrap();

        let mut weight_str = String::new();
        io::stdin().read_line(&mut weight_str).unwrap();
        let weight_str = weight_str.trim();

        let weight = if weight_str.is_empty() {
            1.0
        } else {
            weight_str.parse::<f32>().unwrap_or(1.0)
        };

        kb.add_example(
            input.to_string(),
            ResponseFormat::Text(output.to_string()),
            weight,
        );
        println!("Example added!");
    }

    println!("\nWould you like to save the changes? (y/n)");
    print!("> ");
    io::stdout().flush().unwrap();

    let mut save = String::new();
    io::stdin().read_line(&mut save).unwrap();
    let save = save.trim();

    if save.to_lowercase() == "y" {
        match kb.save(None) {
            Ok(_) => println!("Knowledge base saved!"),
            Err(e) => println!("Error saving: {}", e),
        }
    }
}
