use std::string::String;
use term::terminfo;

#[derive(Debug)]
pub struct TermInfo {
    pub info: terminfo::TermInfo,
}

#[allow(dead_code)]
impl TermInfo {
    pub fn new() -> Self {
        let empty = terminfo::TermInfo { names: Default::default(),
                                         bools: Default::default(),
                                         numbers: Default::default(),
                                         strings: Default::default() };
        TermInfo { info: terminfo::TermInfo::from_env().unwrap_or(empty) }
    }

    pub fn get_string(&self, command: &str) -> String {
        String::from_utf8(self.info.strings[command].clone()).unwrap()
    }

    pub fn format(s: &str, args: &[usize]) -> String {
        let vecarg: Vec<usize> = match s.find("%i") {
            Some(_) => args.iter().map(|x| x + 1).collect(),
            None => args.to_owned(),
        };
        let mut ret: String = s.replace("%i", "");
        for (i, a) in vecarg.iter().enumerate() {
            let from = format!("%p{}%d", i + 1);
            let to = format!("{a}");
            ret = ret.replace(from.as_str(), to.as_str());
        }
        ret
    }
}

impl Default for TermInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for TermInfo {
    fn clone(&self) -> Self {
        TermInfo { info: terminfo::TermInfo { names: self.info.names.clone(),
                                              bools: self.info.bools.clone(),
                                              numbers: self.info.numbers.clone(),
                                              strings: self.info.strings.clone() } }
    }
}
