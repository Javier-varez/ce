struct CeStatus {}

pub async fn compile(
    src: &str,
    arguments: &[String],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = arguments.join(" ");

    let request_body = json::object! {
        source: src,
        options: {
            userArguments: args
        },
        allowStoreCodeDebug: true
    };

    let client = reqwest::Client::new();
    let response = client
        .post("http://godbolt.org/api/compiler/clang_trunk/compile")
        .json(&request_body)
        .send()
        .await?;

    if response.status() != reqwest::StatusCode::OK {
        println!(
            "Error running compilation request: {}",
            response.text().await?
        );
    } else {
        let response = json::parse(&response.text().await?)?;
        println!("response: {:?}", response);
    }

    Ok(())
}
