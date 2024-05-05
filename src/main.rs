pub mod avtodor;
use std::{fs, path::Path};

fn read_params_from_file(file_path: &Path) -> avtodor::SearchParams {
    let data = fs::read_to_string(file_path).expect("Failed to read file");
    serde_json::from_str(&data).expect("Failed to load params")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let params_file = Path::new("./search_params.json");
    let my_params = read_params_from_file(params_file);

    let avtodor = avtodor::AvtodorClient::new()?;
    let my_debt = avtodor.get_debt(my_params).await?;
    
    if my_debt.total <= 0 {
        println!("No debts to pay. Good job!");
        return Ok(());
    }

    println!("Found debt: {}. Trying to generate link...", my_debt.total);
    let payment_link = avtodor.get_payment_link(my_debt).await?;
    println!("You can pay debt by link: {}", payment_link);
    Ok(())
}
