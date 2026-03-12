use crate::config::Config;
use crate::sound::SoundPreset;

pub fn choose_preset(
    config: &Config,
    command: &str,
    exit_code: i32,
    duration_ms: u64,
) -> Option<SoundPreset> {
    if duration_ms < config.min_duration_ms {
        return None;
    }

    let command = command.trim();

    if config
        .deploy_command_prefixes
        .iter()
        .any(|prefix| command.starts_with(prefix))
    {
        return Some(SoundPreset::Deploy);
    }

    if exit_code == 0 {
        Some(SoundPreset::Success)
    } else {
        Some(SoundPreset::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::choose_preset;
    use crate::config::Config;
    use crate::sound::SoundPreset;

    fn test_config() -> Config {
        Config::default()
    }

    #[test]
    fn ignores_short_commands() {
        let preset = choose_preset(&test_config(), "cargo fmt", 0, 500);
        assert_eq!(preset, None);
    }

    #[test]
    fn uses_deploy_sound_for_matching_prefix() {
        let preset = choose_preset(&test_config(), "git push origin main", 0, 3_000);
        assert_eq!(preset, Some(SoundPreset::Deploy));
    }

    #[test]
    fn uses_success_sound_for_non_deploy_success() {
        let preset = choose_preset(&test_config(), "cargo test", 0, 3_000);
        assert_eq!(preset, Some(SoundPreset::Success));
    }

    #[test]
    fn uses_error_sound_for_failed_commands() {
        let preset = choose_preset(&test_config(), "cargo test", 1, 3_000);
        assert_eq!(preset, Some(SoundPreset::Error));
    }

    #[test]
    fn supports_custom_deploy_prefixes() {
        let config = Config {
            min_duration_ms: 100,
            deploy_command_prefixes: vec!["my-release".to_string()],
        };
        let preset = choose_preset(&config, "my-release staging", 0, 2_000);
        assert_eq!(preset, Some(SoundPreset::Deploy));
    }
}
