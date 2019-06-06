use std::fs;
use std::path::Path;

use crate::emoji;
use crate::settings::global_user::GlobalUser;

pub fn global_config(email: &str, api_key: &str) -> Result<(), failure::Error> {
    let s = GlobalUser {
        email: email.to_string(),
        api_key: api_key.to_string(),
    };

    let toml = toml::to_string(&s)?;

    let config_dir = Path::new(&GlobalUser::config_directory().unwrap_or_else(|| {
        panic!(
            "{0} could not determine config directory. {0}",
            emoji::CONSTRUCTION
        )
    }))
    .to_owned();
    fs::create_dir_all(&config_dir)?;

    let config_file = config_dir.join("default.toml");
    fs::write(&config_file, &toml)?;

    println!(
        "{1} Successfully configured. You can find your configuration file at: {0}. {1}",
        &config_file.to_string_lossy(),
        emoji::SPARKLES,
    );
    Ok(())
}
