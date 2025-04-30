#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Shell {
    CMD,
    Bash,
    Fish,
    SH,
    NU,
}

impl Shell {
    pub fn from_name(name: &str) -> Self {
        match name {
            "cmd" => Shell::CMD,
            "bash" => Shell::Bash,
            "fish" => Shell::Fish,
            "sh" => Shell::SH,
            "nu" => Shell::NU,
            _ => Shell::SH,
        }
    }

    pub fn program(&self) -> &'static str {
        match self {
            Shell::CMD => "cmd",
            Shell::Bash => "bash",
            Shell::Fish => "fish",
            Shell::SH => "sh",
            Shell::NU => "nu",
        }
    }

    pub fn argument(&self) -> &'static str {
        match self {
            Shell::CMD => "/c",
            _ => "-c",
        }
    }

    pub fn version(&self) -> Option<String> {
        if *self == Shell::CMD {
            if let Ok(output) = std::process::Command::new(self.program())
                .arg("/v")
                .output()
            {
                if output.status.success() {
                    return Some(encoding_rs::GBK.decode(&output.stdout).0.to_string());
                }
            }
        } else if let Ok(output) = std::process::Command::new(self.program())
            .arg("--version")
            .output()
        {
            if output.status.success() {
                return Some(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
        return None;
    }
}

pub fn available_shells() -> Vec<Shell> {
    vec![Shell::CMD, Shell::Bash, Shell::SH, Shell::NU]
        .into_iter()
        .filter(|shell| shell.version().is_some())
        .collect()
}
