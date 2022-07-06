use serde_json::value::Value;

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub code: i64,
    pub stdout: Vec<StreamOutput>,
    pub stderr: Vec<StreamOutput>,
}

#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub code: i64,
    pub stdout: Vec<StreamOutput>,
    pub stderr: Vec<StreamOutput>,
    pub asm: Vec<AsmOutput>,
    pub execution: Option<ExecutionResult>,
}

#[derive(Debug, Clone)]
pub struct StreamOutput {
    pub text: String,
    pub tag: Option<(i64, String)>,
}

#[derive(Debug, Clone)]
pub struct AsmOutput {
    pub text: String,
    pub source: Option<(Option<String>, i64)>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid stream received as from compilation result")]
    InvalidStream,
    #[error("No stream found")]
    NoStreamFound,
    #[error("No assembly found in output")]
    NoAsmFound,
    #[error("Error parsing assembly response")]
    InvalidAsm,
    #[error("Error parsing error code")]
    InvalidErrorCode,
    #[error("Invalid HTTP response: {0}")]
    InvalidHttpResponse(String),
    #[error("HTTP Error: {0}")]
    HttpError(#[from] reqwest::Error),
}

fn parse_asm(json_array: &Value) -> Result<Vec<AsmOutput>, Error> {
    match json_array {
        Value::Array(array) => {
            let mut result = vec![];
            for node in array {
                let text = node["text"].as_str().ok_or(Error::InvalidAsm)?.to_owned();
                let source = match &node["source"] {
                    Value::Object(source_map) => {
                        let file = source_map["file"].as_str().map(|x| x.to_owned());
                        let line = source_map["line"].as_i64().ok_or(Error::InvalidAsm)?;

                        Some((file, line))
                    }
                    _ => None,
                };

                result.push(AsmOutput { text, source })
            }
            Ok(result)
        }
        Value::Null => Err(Error::NoAsmFound),
        _ => Err(Error::InvalidAsm),
    }
}

fn parse_tag(json_map: &Value) -> Option<(i64, String)> {
    match json_map {
        Value::Object(map) => {
            let line = map["line"].as_i64();
            let text = map["text"].as_str();

            if let Some(line) = line {
                if let Some(text) = text {
                    return Some((line, text.to_owned()));
                }
            }
            None
        }
        _ => None,
    }
}

fn parse_stream(json_array: &Value) -> Result<Vec<StreamOutput>, Error> {
    match json_array {
        Value::Array(array) => {
            let mut result = vec![];
            for json_stream_output in array {
                result.push(StreamOutput {
                    text: json_stream_output["text"]
                        .as_str()
                        .ok_or(Error::InvalidStream)?
                        .to_string(),
                    tag: parse_tag(&json_stream_output["tag"]),
                })
            }
            Ok(result)
        }
        Value::Null => Err(Error::NoStreamFound),
        _ => Err(Error::InvalidStream),
    }
}

pub async fn execute(
    ce_instance: &str,
    compiler: &str,
    src: &str,
    arguments: &[String],
) -> Result<ExecutionResult, Error> {
    let args = arguments.join(" ");

    let request_body = serde_json::json!({
        "source": src,
        "options": {
            "userArguments": args,
            "compilerOptions": {
                "skipAsm": true,
                "executorRequest": true
            }
        },
        "allowStoreCodeDebug": true
    });

    let client = reqwest::Client::new();
    let request_url = format!("{}/api/compiler/{}/compile", ce_instance, compiler);
    ::log::debug!("Post: {}", request_url);

    let response = client
        .post(request_url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if response.status() != reqwest::StatusCode::OK {
        log::error!("HTTP status code {}", response.status());
        return Err(Error::InvalidHttpResponse(response.text().await?));
    }

    let json: serde_json::Map<_, _> = response.json().await?;
    log::info!("HTTP response: {:?}", json);
    let result = ExecutionResult {
        code: json["code"].as_i64().ok_or(Error::InvalidErrorCode)?,
        stdout: json
            .get("stdout")
            .and_then(|data| parse_stream(data).ok())
            .unwrap_or(vec![]),
        stderr: json
            .get("stderr")
            .and_then(|data| parse_stream(data).ok())
            .unwrap_or(vec![]),
    };
    Ok(result)
}

pub async fn compile(
    ce_instance: &str,
    compiler: &str,
    src: &str,
    arguments: &[String],
    run_program: bool,
) -> Result<CompilationResult, Error> {
    let args = arguments.join(" ");

    let request_body = serde_json::json!({
        "source": src,
        "options": {
            "userArguments": args,
        },
        "allowStoreCodeDebug": true
    });

    let client = reqwest::Client::new();
    let request_url = format!("{}/api/compiler/{}/compile", ce_instance, compiler);
    ::log::debug!("Post: {}", request_url);

    let response = client
        .post(request_url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if response.status() != reqwest::StatusCode::OK {
        log::error!("HTTP status code {}", response.status());
        return Err(Error::InvalidHttpResponse(response.text().await?));
    }

    let json: serde_json::Map<_, _> = response.json().await?;
    log::info!("HTTP response: {:?}", json);

    let execution_result = if run_program {
        Some(execute(ce_instance, compiler, src, arguments).await?)
    } else {
        None
    };

    let result = CompilationResult {
        code: json["code"].as_i64().ok_or(Error::InvalidErrorCode)?,
        stdout: json
            .get("stdout")
            .and_then(|data| parse_stream(data).ok())
            .unwrap_or(vec![]),
        stderr: json
            .get("stderr")
            .and_then(|data| parse_stream(data).ok())
            .unwrap_or(vec![]),
        asm: json
            .get("asm")
            .and_then(|data| parse_asm(data).ok())
            .unwrap_or(vec![]),
        execution: execution_result,
    };

    return Ok(result);
}
