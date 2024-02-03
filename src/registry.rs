#![allow(dead_code)]

use anyhow::Result;
use bytes::Bytes;
use reqwest::{blocking::Client, StatusCode, Url};

#[derive(Debug, serde::Deserialize)]
struct AuthResponse {
    token: String,
    access_token: String,
    expires_in: Option<i32>,
    issued_at: Option<String>,
    refresh_token: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    schema_version: u32,
    media_type: String,
    config: Config,
    layers: Vec<Layer>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    media_type: String,
    size: u32,
    digest: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Layer {
    media_type: String,
    digest: String,
    size: u32,
    urls: Option<Vec<String>>,
}

pub fn pull_image(image: &str) -> Result<Vec<Bytes>> {
    let client = Client::new();
    let image: Vec<&str> = image.split(':').collect();
    let image_name = image[0];
    let image_tag = image.get(1).unwrap_or(&"latest");
    let registry_url = Url::parse(&format!(
        "https://registry.hub.docker.com/v2/library/{image_name}/manifests/{image_tag}"
    ))?;

    let res = client.get(registry_url.clone()).send()?;

    match res.status() {
        StatusCode::UNAUTHORIZED => {
            let token = get_auth_token(&client, image_name)?;
            let res = client
                .get(registry_url)
                .header("Authorization", format!("Bearer {}", token))
                .header(
                    "Accept",
                    "application/vnd.docker.distribution.manifest.v2+json",
                )
                .send()?;

            match res.status() {
                StatusCode::OK => {
                    let manifest: Manifest = res.json()?;
                    let mut layers = Vec::with_capacity(manifest.layers.len());
                    for layer in manifest.layers {
                        let url = Url::parse(&format!(
                            "https://registry.hub.docker.com/v2/library/{}/blobs/{}",
                            image_name, layer.digest
                        ))?;

                        let res = client
                            .get(url)
                            .header("Authorization", format!("Bearer {}", token))
                            .send()?;
                        match res.status() {
                            StatusCode::OK => {
                                layers.push(res.bytes()?);
                            }
                            _ => {
                                anyhow::bail!("Unexpected status code: {}", res.status());
                            }
                        }
                    }
                    Ok(layers)
                }
                _ => {
                    anyhow::bail!("Unexpected status code: {}", res.status());
                }
            }
        }
        //StatusCode::OK => {}
        _ => {
            anyhow::bail!("Unexpected status code: {}", res.status());
        }
    }
}

fn get_auth_token(client: &Client, image_name: &str) -> Result<String> {
    let auth_url = Url::parse(&format!(
    "https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/{image_name}:pull"
))?;
    let res = client.get(auth_url).send()?;
    match res.status() {
        StatusCode::OK => {
            let auth_response: AuthResponse = res.json()?;
            Ok(auth_response.token)
        }
        _ => {
            anyhow::bail!("Unexpected status code: {}", res.status());
        }
    }
}
