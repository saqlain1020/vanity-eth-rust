use clap::Parser;
use ethereum_types::H160;
use indicatif::{ProgressBar, ProgressStyle};
use rand::rngs::OsRng;
use rayon::prelude::*;
use secp256k1::{Secp256k1, SecretKey};
use sha3::{Digest, Keccak256};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Desired address prefix (without 0x)
    #[arg(short, long)]
    prefix: Option<String>,

    /// Desired address suffix
    #[arg(short, long)]
    suffix: Option<String>,

    /// Number of threads to use (default: number of CPU cores)
    #[arg(short, long)]
    threads: Option<usize>,
    
    /// Number of addresses to generate (default: 1)
    #[arg(short, long, default_value_t = 1)]
    quantity: usize,
}

#[derive(Clone)]
struct KeyPair {
    private_key: SecretKey,
    address: String,
}

fn generate_key_pair(secp: &Secp256k1<secp256k1::All>) -> KeyPair {
    let secret_key = SecretKey::new(&mut OsRng);
    let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
    
    let public_key_bytes = public_key.serialize_uncompressed();
    let public_key_hash = Keccak256::digest(&public_key_bytes[1..]);
    let address = H160::from_slice(&public_key_hash[12..32]);
    
    KeyPair {
        private_key: secret_key,
        address: format!("0x{:x}", address),
    }
}

fn matches_criteria(address: &str, prefix: &Option<String>, suffix: &Option<String>) -> bool {
    let addr_without_prefix = &address[2..]; // Remove "0x" prefix
    
    if let Some(prefix) = prefix {
        if !addr_without_prefix.to_lowercase().starts_with(&prefix.to_lowercase()) {
            return false;
        }
    }
    
    if let Some(suffix) = suffix {
        if !addr_without_prefix.to_lowercase().ends_with(&suffix.to_lowercase()) {
            return false;
        }
    }
    
    true
}

fn main() {
    let args = Args::parse();
    let num_threads = args.threads.unwrap_or_else(num_cpus::get);
    rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global().unwrap();
    
    println!("Ethereum Vanity Address Generator");
    println!("--------------------------------");
    println!("Using {} threads", num_threads);
    println!("Generating {} address(es)", args.quantity);
    if let Some(prefix) = &args.prefix {
        println!("Looking for prefix: {}", prefix);
    }
    if let Some(suffix) = &args.suffix {
        println!("Looking for suffix: {}", suffix);
    }
    println!();
    
    let found_keypairs = Arc::new(Mutex::new(Vec::with_capacity(args.quantity)));
    let attempts = Arc::new(AtomicU64::new(0));
    let completed = Arc::new(AtomicBool::new(false));
    let start_time = Instant::now();
    let pb = ProgressBar::new_spinner();
    
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap(),
    );
    
    // Update progress and stats every 100ms
    let attempts_clone = attempts.clone();
    let pb_clone = pb.clone();
    let found_keypairs_clone = found_keypairs.clone();
    let completed_clone = completed.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(100));
            
            if completed_clone.load(Ordering::Relaxed) {
                break;
            }
            
            let current_attempts = attempts_clone.load(Ordering::Relaxed);
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = current_attempts as f64 / elapsed;
            let found_count = found_keypairs_clone.lock().unwrap().len();
            pb_clone.set_message(format!("{:.2} keys/s | Found: {}/{}", speed, found_count, args.quantity));
        }
    });
    
    // Generate addresses in parallel
    (0..num_threads).into_par_iter().for_each(|_| {
        let secp = Secp256k1::new();
        let attempts = attempts.clone();
        let found_keypairs = found_keypairs.clone();
        let completed = completed.clone();
        
        loop {
            // Check if we're done
            if completed.load(Ordering::Relaxed) {
                break;
            }
            
            let keypair = generate_key_pair(&secp);
            attempts.fetch_add(1, Ordering::Relaxed);
            
            if matches_criteria(&keypair.address, &args.prefix, &args.suffix) {
                let mut found = found_keypairs.lock().unwrap();
                
                // Only add if we haven't reached the quantity
                if found.len() < args.quantity {
                    found.push(keypair);
                    
                    // If we've found all the addresses, mark as completed
                    if found.len() >= args.quantity {
                        completed.store(true, Ordering::Relaxed);
                    }
                }
                
                drop(found);
            }
        }
    });
    
    // Mark as completed to stop the progress thread
    completed.store(true, Ordering::Relaxed);
    
    pb.finish_and_clear();
    
    // Print results
    let total_attempts = attempts.load(Ordering::Relaxed);
    let elapsed = start_time.elapsed().as_secs_f64();
    let speed = if elapsed > 0.0 { total_attempts as f64 / elapsed } else { 0.0 };
    let found_keypairs = found_keypairs.lock().unwrap();
    
    if !found_keypairs.is_empty() {
        println!("\nFound {} matching address(es)!", found_keypairs.len());
        
        for (i, keypair) in found_keypairs.iter().enumerate() {
            println!("\nAddress #{}", i + 1);
            println!("Private Key: {}", hex::encode(keypair.private_key.secret_bytes()));
            println!("Address: {}", keypair.address);
        }
        
        println!("\nStats:");
        println!("Time taken: {:.2} seconds", elapsed);
        println!("Total attempts: {}", total_attempts);
        println!("Average speed: {:.2} keys/s", speed);
    }
    
    println!("\nIMPORTANT: Store your private key securely and never share it with anyone!");
} 