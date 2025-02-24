use anyhow::Result;
use jwt_crackng::*;

fn main() -> Result<()> {
    let args = cli::parse_args()?;
    let result = bruteforce::crack(&args)?;
    
    match result {
        Some(secret) => {
            println!("\nSecret found: {}", secret);
            if args.verbose {
                println!("Time taken: {:?}", std::time::SystemTime::now()
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)?);
            }
        },
        None => println!("\nSecret not found after exhausting all possibilities")
    }
    Ok(())
}