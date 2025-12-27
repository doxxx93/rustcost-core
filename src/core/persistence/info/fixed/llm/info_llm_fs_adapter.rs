use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

use crate::core::persistence::info::fixed::info_fixed_fs_adapter_trait::InfoFixedFsAdapterTrait;
use crate::core::persistence::storage_path::info_llm_path;

use super::info_llm_entity::InfoLlmEntity;
use super::llm_provider::LlmProvider;

/// FS adapter for persisted LLM configuration.
///
/// Uses a simple key-value `llm.rci` file with atomic writes.
pub struct InfoLlmFsAdapter;

impl InfoFixedFsAdapterTrait<InfoLlmEntity> for InfoLlmFsAdapter {
    fn new() -> Self where Self: Sized {
        Self {}
    }

    fn read(&self) -> Result<InfoLlmEntity> {
        let path = info_llm_path();
        if !path.exists() {
            return Ok(InfoLlmEntity::default());
        }

        let file = File::open(&path).context("Failed to open llm file")?;
        let reader = BufReader::new(file);
        let mut s = InfoLlmEntity::default();

        for line in reader.lines() {
            let line = line?;
            if let Some((key, val)) = line.split_once(':') {
                let key = key.trim().to_uppercase();
                let val = val.trim();

                match key.as_str() {
                    "PROVIDER" => {
                        if let Some(p) = LlmProvider::from_code(val) {
                            s.provider = p;
                        }
                    }
                    "BASE_URL" => s.base_url = if val.is_empty() { None } else { Some(val.to_string()) },
                    "TOKEN" => s.token = if val.is_empty() { None } else { Some(val.to_string()) },
                    "MODEL" => s.model = if val.is_empty() { None } else { Some(val.to_string()) },
                    "MAX_OUTPUT_TOKENS" => {
                        s.max_output_tokens = val.parse().ok();
                    }
                    "TEMPERATURE" => s.temperature = val.parse().ok(),
                    "TOP_P" => s.top_p = val.parse().ok(),
                    "TOP_K" => s.top_k = val.parse().ok(),
                    "PRESENCE_PENALTY" => s.presence_penalty = val.parse().ok(),
                    "FREQUENCY_PENALTY" => s.frequency_penalty = val.parse().ok(),
                    "TIMEOUT_MS" => s.timeout_ms = val.parse().ok(),
                    "STREAM" => s.stream = val.eq_ignore_ascii_case("true"),
                    "STOP_SEQUENCES" => {
                        let seq: Vec<String> = val
                            .split(',')
                            .map(|v| v.trim().to_string())
                            .filter(|v| !v.is_empty())
                            .collect();
                        s.stop_sequences = if seq.is_empty() { None } else { Some(seq) };
                    }
                    "ORGANIZATION" => s.organization = if val.is_empty() { None } else { Some(val.to_string()) },
                    "USER" => s.user = if val.is_empty() { None } else { Some(val.to_string()) },
                    "CREATED_AT" => {
                        if let Ok(dt) = val.parse::<DateTime<Utc>>() {
                            s.created_at = dt;
                        }
                    }
                    "UPDATED_AT" => {
                        if let Ok(dt) = val.parse::<DateTime<Utc>>() {
                            s.updated_at = dt;
                        }
                    }
                    "VERSION" => s.version = val.to_string(),
                    _ => {}
                }
            }
        }

        Ok(s)
    }

    fn insert(&self, data: &InfoLlmEntity) -> Result<()> {
        self.write(data)
    }

    fn update(&self, data: &InfoLlmEntity) -> Result<()> {
        self.write(data)
    }

    fn delete(&self) -> Result<()> {
        let path = info_llm_path();
        if path.exists() {
            fs::remove_file(&path).context("Failed to delete llm file")?;
        }
        Ok(())
    }
}

impl InfoLlmFsAdapter {
    fn write(&self, data: &InfoLlmEntity) -> Result<()> {
        use std::io::Write;

        let path = info_llm_path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).context("Failed to create llm directory")?;
        }

        let tmp_path = path.with_extension("rci.tmp");
        let mut f = File::create(&tmp_path).context("Failed to create temp llm file")?;

        writeln!(f, "PROVIDER:{}", data.provider.as_code())?;
        writeln!(f, "BASE_URL:{}", data.base_url.clone().unwrap_or_default())?;
        writeln!(f, "TOKEN:{}", data.token.clone().unwrap_or_default())?;
        writeln!(f, "MODEL:{}", data.model.clone().unwrap_or_default())?;
        if let Some(v) = data.max_output_tokens {
            writeln!(f, "MAX_OUTPUT_TOKENS:{}", v)?;
        }
        if let Some(v) = data.temperature {
            writeln!(f, "TEMPERATURE:{}", v)?;
        }
        if let Some(v) = data.top_p {
            writeln!(f, "TOP_P:{}", v)?;
        }
        if let Some(v) = data.top_k {
            writeln!(f, "TOP_K:{}", v)?;
        }
        if let Some(v) = data.presence_penalty {
            writeln!(f, "PRESENCE_PENALTY:{}", v)?;
        }
        if let Some(v) = data.frequency_penalty {
            writeln!(f, "FREQUENCY_PENALTY:{}", v)?;
        }
        if let Some(v) = data.timeout_ms {
            writeln!(f, "TIMEOUT_MS:{}", v)?;
        }
        writeln!(f, "STREAM:{}", data.stream)?;
        let stops = data
            .stop_sequences
            .as_ref()
            .map(|v| v.join(","))
            .unwrap_or_default();
        writeln!(f, "STOP_SEQUENCES:{}", stops)?;
        writeln!(f, "ORGANIZATION:{}", data.organization.clone().unwrap_or_default())?;
        writeln!(f, "USER:{}", data.user.clone().unwrap_or_default())?;
        writeln!(f, "CREATED_AT:{}", data.created_at.to_rfc3339())?;
        writeln!(f, "UPDATED_AT:{}", data.updated_at.to_rfc3339())?;
        writeln!(f, "VERSION:{}", data.version)?;

        f.flush()?;
        f.sync_all().context("Failed to sync temp llm file")?;
        fs::rename(&tmp_path, &path).context("Failed to finalize llm file")?;

        #[cfg(unix)]
        if let Some(dir) = path.parent() {
            use std::os::unix::fs::FileExt as _;
            let dir_file = File::open(dir).context("Failed to open llm directory")?;
            dir_file.sync_all().context("Failed to sync llm directory")?;
        }

        Ok(())
    }
}
