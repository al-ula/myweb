use crate::post::Join;

pub trait TitleCase {
    fn title_case(&self) -> String;
}

impl TitleCase for &str {
    fn title_case(&self) -> String {
        self.split(' ')
            .map(|s| {
                s.chars().next().unwrap().to_uppercase().collect::<String>() + s.get(1..).unwrap()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl TitleCase for String {
    fn title_case(&self) -> String {
        self.as_str().title_case()
    }
}
pub trait SnakeToTitleCase {
    fn snake_to_title_case(&self) -> String;
}

impl SnakeToTitleCase for &str {
    fn snake_to_title_case(&self) -> String {
        fn capitalize_first_letter(s: &str) -> String {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }

        self.split('_')
            .map(capitalize_first_letter)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl SnakeToTitleCase for String {
    fn snake_to_title_case(&self) -> String {
        self.as_str().snake_to_title_case()
    }
}

pub trait StringCutter {
    fn cut_to_length(&self, max_length: usize) -> String;
}

impl StringCutter for String {
    fn cut_to_length(&self, max_length: usize) -> String {
        if self.len() <= max_length {
            self.clone()
        } else {
            self.chars().take(max_length).collect::<String>()
        }
    }
}

impl StringCutter for str {
    fn cut_to_length(&self, max_length: usize) -> String {
        if self.len() <= max_length {
            self.to_string()
        } else {
            self.chars().take(max_length).collect::<String>()
        }
    }
}

impl Join<String> for String {
    fn join(&self, other: &std::string::String) -> String {
        format!("{}{}", self, &other)
    }
}
