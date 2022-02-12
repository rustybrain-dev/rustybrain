use serde::Deserialize;

#[derive(Deserialize)]
pub struct Color {
    primary: String,
    primary_light: String,
    primary_dark: String,
    primary_text: String,

    secondary: String,
    secondary_light: String,
    secondary_dark: String,
    secondary_text: String,

    foreground: String,
    background: String,

    yellow: String,
    brown: String,
    orange: String,
    red: String,
    pink: String,
    purple: String,
    blue: String,
    indigo: String,
    cyan: String,
    teal: String,
    green: String,
}

impl Color {
    pub fn from_json(content: &str) -> Option<Self> {
        serde_json::from_str(content).ok()
    }
}

// light colors from https://github.com/waymondo/apropospriate-theme/blob/master/apropospriate.el
pub const APROPOSPRIATE_COLOR_CONTENT: &'static str = r##"
{
  "primary": "#546E7A",
  "primary_text": "#FAFAFA",
  "primary_dark": "#29434e",
  "primary_light": "#819ca9",

  "secondary": "#B2EBF2",
  "secondary_light": "#e5ffff",
  "secondary_dark": "#81b9bf",
  "secondary_text": "#000000",

  "foreground": "#546E7A",
  "background": "#FAFAFA",

  "base00": "#FAFAFA",
  "base01": "#90A4AE",
  "base02": "#78909C",
  "base03": "#546E7A",
  "yellow": "#F57F17",
  "yellow1": "#F9A725",
  "brown": "#4E342E",
  "brown1": "#6D4C41",
  "orange": "#D84315",
  "orange1": "#FF5722",
  "red": "#D50000",
  "red1": "#FF1744",
  "pink": "#F8BBD0",
  "pink1": "#EC407A",
  "purple": "#7E57C2",
  "purple1": "#B388FF",
  "blue": "#42A5F5",
  "blue1": "#1E88E5",
  "indigo": "#5C6BC0",
  "indigo1": "#9FA8DA",
  "cyan": "#0097A7",
  "cyan1": "#00B8D4",
  "teal": "#26A69A",
  "teal1": "#00897B",
  "green": "#66BB6A",
  "green1": "#558B2F"
}
"##;
