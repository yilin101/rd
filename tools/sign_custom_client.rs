use hbb_common::sodiumoxide::{base64, crypto::sign};
use serde_json::Value;
use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if hbb_common::sodiumoxide::init().is_err() {
        return Err("failed to initialize sodiumoxide".into());
    }
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("generate-key") => {
            let (public, secret) = sign::gen_keypair();
            println!(
                "RUSTDESK_CUSTOM_PUBLIC_KEY={}",
                base64::encode(public.as_ref(), base64::Variant::Original)
            );
            println!(
                "RUSTDESK_CUSTOM_SECRET_KEY={}",
                base64::encode(secret.as_ref(), base64::Variant::Original)
            );
            Ok(())
        }
        Some("sign") => {
            let input = args.next().unwrap_or_else(|| "custom-config.json".to_owned());
            let output = args.next().unwrap_or_else(|| "custom.txt".to_owned());
            sign_config(&input, &output)
        }
        _ => {
            eprintln!(
                "用法：sign-custom-client generate-key | sign [custom-config.json] [custom.txt]"
            );
            Err("invalid arguments".into())
        }
    }
}

fn sign_config(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let raw = fs::read(input)?;
    let _: Value = serde_json::from_slice(&raw)?;
    let secret = env::var("RUSTDESK_CUSTOM_SECRET_KEY")?;
    let secret = base64::decode(&secret, base64::Variant::Original)
        .map_err(|_| "invalid base64 Ed25519 secret key")?;
    let secret = match secret.len() {
        sign::SECRETKEYBYTES => {
            sign::SecretKey::from_slice(&secret).ok_or("invalid Ed25519 secret key")?
        }
        sign::SEEDBYTES => {
            let seed = sign::Seed::from_slice(&secret).ok_or("invalid Ed25519 seed")?;
            sign::keypair_from_seed(&seed).1
        }
        _ => return Err("Ed25519 key must be a 32-byte seed or 64-byte secret key".into()),
    };
    let signed = sign::sign(&raw, &secret);
    let encoded = base64::encode(signed, base64::Variant::Original);
    let output_path = PathBuf::from(output);
    fs::write(&output_path, encoded)?;
    println!("已生成 {}", output_path.display());
    Ok(())
}
