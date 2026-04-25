#[derive(Debug, Clone, Copy)]
pub enum Language {
    Chinese,
    English,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub theme_dark: bool,
    pub lang: Language,
    pub max_guess: usize,
}

impl Config {
    pub fn new() -> Self {
        Self {
            theme_dark: false,
            lang: detect_lang(),
            max_guess: 10,
        }
    }
}

fn detect_lang() -> Language {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return Language::English,
    };

    match window.navigator().language() {
        Some(lang) if lang.to_lowercase().starts_with("zh") => Language::Chinese,
        _ => Language::English,
    }

}

