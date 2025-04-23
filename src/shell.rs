#[derive(Clone)]
pub struct Shell {
    pub program: &'static str,
    pub argument: &'static str,
    pub version: &'static str,
    pub encoding: &'static encoding_rs::Encoding,
}

impl Shell {
    pub fn from_name(name: &str) -> Self {
        match name {
            "cmd" => CMD.clone(),
            "sh" => SH.clone(),
            "nu" => NU.clone(),
            _ => CMD.clone(),
        }
    }

    pub fn get_version(&self) -> Option<String> {
        if let Ok(output) = std::process::Command::new(self.program)
            .arg(self.version)
            .output()
        {
            if output.status.success() {
                return Some(self.encoding.decode(&output.stdout).0.to_string());
            }
        }
        return None;
    }
}

pub static CMD: Shell = Shell {
    program: "cmd",
    argument: "/c",
    version: "/v",
    encoding: encoding_rs::GBK,
};

pub static SH: Shell = Shell {
    program: "sh",
    argument: "-c",
    version: "--version",
    encoding: encoding_rs::UTF_8,
};

pub static NU: Shell = Shell {
    program: "nu",
    argument: "-c",
    version: "--version",
    encoding: encoding_rs::UTF_8,
};
