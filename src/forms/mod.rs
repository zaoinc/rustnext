use crate::Request;
// Removed unused imports: Response, Deserialize, Serialize
use std::collections::HashMap;

// Removed pub mod validation;
// Removed pub mod form_builder;

#[derive(Debug, Clone)]
pub struct FormField {
    pub name: String,
    pub field_type: String,
    pub value: String,
    pub required: bool,
    pub validation_rules: Vec<ValidationRule>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ValidationRule {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Email,
    Numeric,
    Custom(fn(&str) -> Result<(), String>),
}

#[derive(Debug, Clone)]
pub struct Form {
    pub fields: HashMap<String, FormField>,
    pub is_valid: bool,
    pub errors: Vec<String>,
}

impl Form {
    pub fn new() -> Self {
        Form {
            fields: HashMap::new(),
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn add_field(&mut self, name: &str, field_type: &str, required: bool) -> &mut FormField {
        let field = FormField {
            name: name.to_string(),
            field_type: field_type.to_string(),
            value: String::new(),
            required,
            validation_rules: Vec::new(),
            errors: Vec::new(),
        };
        
        self.fields.insert(name.to_string(), field);
        self.fields.get_mut(name).unwrap()
    }

    pub fn validate(&mut self) -> bool {
        self.is_valid = true;
        self.errors.clear();

        for (_, field) in &mut self.fields {
            field.errors.clear();
            
            for rule in &field.validation_rules {
                match rule {
                    ValidationRule::Required => {
                        if field.value.trim().is_empty() {
                            field.errors.push(format!("{} is required", field.name));
                            self.is_valid = false;
                        }
                    }
                    ValidationRule::MinLength(min) => {
                        if field.value.len() < *min {
                            field.errors.push(format!("{} must be at least {} characters", field.name, min));
                            self.is_valid = false;
                        }
                    }
                    ValidationRule::MaxLength(max) => {
                        if field.value.len() > *max {
                            field.errors.push(format!("{} must be no more than {} characters", field.name, max));
                            self.is_valid = false;
                        }
                    }
                    ValidationRule::Email => {
                        if !field.value.contains('@') || !field.value.contains('.') {
                            field.errors.push(format!("{} must be a valid email", field.name));
                            self.is_valid = false;
                        }
                    }
                    ValidationRule::Numeric => {
                        if field.value.parse::<f64>().is_err() {
                            field.errors.push(format!("{} must be a number", field.name));
                            self.is_valid = false;
                        }
                    }
                    ValidationRule::Custom(validator) => {
                        if let Err(error) = validator(&field.value) {
                            field.errors.push(error);
                            self.is_valid = false;
                        }
                    }
                }
            }
        }

        self.is_valid
    }

    pub fn populate_from_request(&mut self, req: &Request) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // This would parse form data from the request body
        // For now, we'll use query parameters as a simple example
        for (key, value) in &req.query {
            if let Some(field) = self.fields.get_mut(key) {
                field.value = value.clone();
            }
        }
        Ok(())
    }
}

impl FormField {
    pub fn required(mut self) -> Self {
        self.validation_rules.push(ValidationRule::Required);
        self
    }

    pub fn min_length(mut self, min: usize) -> Self {
        self.validation_rules.push(ValidationRule::MinLength(min));
        self
    }

    pub fn max_length(mut self, max: usize) -> Self {
        self.validation_rules.push(ValidationRule::MaxLength(max));
        self
    }

    pub fn email(mut self) -> Self {
        self.validation_rules.push(ValidationRule::Email);
        self
    }

    pub fn numeric(mut self) -> Self {
        self.validation_rules.push(ValidationRule::Numeric);
        self
    }
}
