use serde_json::value::Value;

#[derive(Debug)]
pub struct CompilationResult {
    pub code: i64,
    pub stdout: Vec<StreamOutput>,
    pub stderr: Vec<StreamOutput>,
    pub asm: Vec<AsmOutput>,
}

#[derive(Debug)]
pub struct StreamOutput {
    pub text: String,
    pub tag: Option<(i64, String)>,
}

#[derive(Debug)]
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
    #[error("Error parsing error code")]
    InvalidHttpResponse(String),
    #[error("Error parsing error code")]
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

pub async fn compile(
    ce_instance: &str,
    compiler: &str,
    src: &str,
    arguments: &[String],
) -> Result<CompilationResult, Error> {
    let args = arguments.join(" ");

    let request_body = serde_json::json!({
        "source": src,
        "options": {
            "userArguments": args
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
        return Err(Error::InvalidHttpResponse(response.text().await?));
    }

    let json: serde_json::Map<_, _> = response.json().await?;
    let result = CompilationResult {
        code: json["code"].as_i64().ok_or(Error::InvalidErrorCode)?,
        stdout: parse_stream(&json["stdout"])?,
        stderr: parse_stream(&json["stderr"])?,
        asm: parse_asm(&json["asm"])?,
    };

    return Ok(result);
}
