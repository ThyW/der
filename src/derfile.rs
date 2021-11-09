use std::collections::HashMap;

pub const TEMPLATE_LEFT: &str = "[";
pub const TEMPLATE_RIGHT: &str = "]";
pub const CODE_SEP: &str = "`";
pub const VAR_PREF: &str = "$";
pub const CODE_KEYWORDS: [&str; 1] = ["env"];

#[derive(Debug, Clone, Default)]
pub struct Template {
    pub name: String,
    pub final_name: String,
    pub hostnames: Vec<String>,
    pub apply_path: String,
}

#[derive(Debug, Clone, Default)]
pub struct Variable {
    pub _name: String,
    pub value: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Derfile {
    pub templates: HashMap<String, Template>,
    pub vars: HashMap<String, Variable>,
}

impl Template {
    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    pub fn set_final_name(&mut self, final_name: String) {
        self.final_name = final_name
    }

    pub fn add_hostname(&mut self, hostname: String) {
        self.hostnames.push(hostname)
    }

    pub fn set_apply_path(&mut self, apply_path: String) {
        self.apply_path = apply_path
    }
}

impl Variable {
    fn new(_name: String, value: Vec<String>) -> Self {
        Self { _name, value }
    }
}

impl Derfile {
    pub fn add_template(&mut self, name: String) {
        self.templates.insert(name, Default::default());
    }

    pub fn get_template(&mut self, name: &String) -> Option<&mut Template> {
        self.templates.get_mut(name)
    }

    pub fn add_var(&mut self, name: String, value: Vec<String>) {
        self.vars.insert(name.clone(), Variable::new(name, value));
    }

    pub fn get_var(&mut self, name: &String) -> Option<&mut Variable> {
        self.vars.get_mut(name)
    }

    pub fn parse(&mut self) -> Self {
        let mut derfile: Derfile = Default::default();
        let mut self_clone = self.clone();
        for (template_name, template) in self.templates.iter() {
            derfile.add_template(template_name.clone());
            let mut temp = derfile.get_template(&template_name).unwrap();
            temp.name = template_name.to_string();
            if template.final_name.starts_with(VAR_PREF) {
                let variable_name = template
                    .final_name
                    .strip_prefix(VAR_PREF)
                    .unwrap()
                    .to_string();

                if let Some(variable) = self_clone.get_var(&variable_name) {
                    let mut temp = derfile.get_template(&template.name).unwrap();
                    temp.final_name = variable.value[0].clone(); // only take the fist value, sicne we only accept only one final file name
                }
            } else {
                let mut temp = derfile.get_template(&template_name).unwrap();
                temp.final_name = template.final_name.clone();
            }
            if template.apply_path.starts_with(VAR_PREF) {
                let variable_name = template
                    .apply_path
                    .strip_prefix(VAR_PREF)
                    .unwrap()
                    .to_string();

                if let Some(variable) = self_clone.get_var(&variable_name) {
                    let mut template = derfile.get_template(&template.name).unwrap();
                    template.apply_path = variable.value[0].clone(); // only take the first value, since we only accpet one apply path now
                }
            } else {
                let mut temp = derfile.get_template(&template_name).unwrap();
                temp.apply_path = template.apply_path.clone()
            }
            let mut hostname_clone: Vec<String> = Vec::new();
            for hostname_entry in template.hostnames.iter() {
                if hostname_entry.starts_with(VAR_PREF) {
                    if let Some(variable) = self_clone
                        .get_var(&hostname_entry.strip_prefix(VAR_PREF).unwrap().to_string())
                    {
                        let mut variable_value = variable.value.clone();
                        hostname_clone.append(&mut variable_value);
                    }
                } else {
                    hostname_clone.push(hostname_entry.to_string())
                }
            }

            let template = derfile.get_template(&template.name).unwrap();
            template.hostnames = hostname_clone
        }
        derfile.vars = self.vars.clone();

        derfile
    }
}
