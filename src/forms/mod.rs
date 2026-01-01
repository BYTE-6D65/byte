/// High-level form system for user input collection
/// Supports command templates, batch operations, config editing, and more

use std::collections::HashMap;

/// A complete form with multiple fields
#[derive(Clone, Debug)]
pub struct Form {
    pub title: String,
    pub description: Option<String>,
    pub fields: Vec<FormField>,
    pub current_field: usize,
    pub submitted: bool,
    pub cancelled: bool,
}

impl Form {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            fields: vec![],
            current_field: 0,
            submitted: false,
            cancelled: false,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn field(mut self, field: FormField) -> Self {
        self.fields.push(field);
        self
    }

    pub fn text_input(
        self,
        id: impl Into<String>,
        label: impl Into<String>,
        placeholder: impl Into<String>,
    ) -> Self {
        self.field(FormField::TextInput {
            id: id.into(),
            label: label.into(),
            placeholder: placeholder.into(),
            value: String::new(),
            validator: None,
        })
    }

    pub fn text_area(
        self,
        id: impl Into<String>,
        label: impl Into<String>,
        placeholder: impl Into<String>,
        height: usize,
    ) -> Self {
        self.field(FormField::TextArea {
            id: id.into(),
            label: label.into(),
            placeholder: placeholder.into(),
            value: String::new(),
            height,
        })
    }

    pub fn email(
        self,
        id: impl Into<String>,
        label: impl Into<String>,
        placeholder: impl Into<String>,
    ) -> Self {
        self.field(FormField::Email {
            id: id.into(),
            label: label.into(),
            placeholder: placeholder.into(),
            value: String::new(),
        })
    }

    pub fn number(
        self,
        id: impl Into<String>,
        label: impl Into<String>,
        default: i64,
        min: Option<i64>,
        max: Option<i64>,
    ) -> Self {
        self.field(FormField::Number {
            id: id.into(),
            label: label.into(),
            value: default,
            min,
            max,
        })
    }

    pub fn select(
        self,
        id: impl Into<String>,
        label: impl Into<String>,
        options: Vec<String>,
    ) -> Self {
        self.field(FormField::Select {
            id: id.into(),
            label: label.into(),
            options,
            selected: 0,
        })
    }

    pub fn multi_select(
        self,
        id: impl Into<String>,
        label: impl Into<String>,
        options: Vec<String>,
    ) -> Self {
        self.field(FormField::MultiSelect {
            id: id.into(),
            label: label.into(),
            options,
            selected: vec![],
        })
    }

    pub fn checkbox(self, id: impl Into<String>, label: impl Into<String>) -> Self {
        self.field(FormField::Checkbox {
            id: id.into(),
            label: label.into(),
            checked: false,
        })
    }

    pub fn path(
        self,
        id: impl Into<String>,
        label: impl Into<String>,
        kind: PathKind,
    ) -> Self {
        self.field(FormField::Path {
            id: id.into(),
            label: label.into(),
            value: String::new(),
            kind,
        })
    }

    /// Navigate to next field
    pub fn next_field(&mut self) {
        if self.current_field < self.fields.len().saturating_sub(1) {
            self.current_field += 1;
        }
    }

    /// Navigate to previous field
    pub fn prev_field(&mut self) {
        if self.current_field > 0 {
            self.current_field -= 1;
        }
    }

    /// Get current field (mutable)
    pub fn current_field_mut(&mut self) -> Option<&mut FormField> {
        self.fields.get_mut(self.current_field)
    }

    /// Get current field (immutable)
    pub fn current_field_ref(&self) -> Option<&FormField> {
        self.fields.get(self.current_field)
    }

    /// Validate all fields
    pub fn validate(&self) -> Result<(), String> {
        for field in &self.fields {
            field.validate()?;
        }
        Ok(())
    }

    /// Extract all field values as a HashMap
    pub fn values(&self) -> HashMap<String, FormValue> {
        let mut values = HashMap::new();
        for field in &self.fields {
            let (id, value) = field.get_value();
            values.insert(id, value);
        }
        values
    }

    /// Submit the form (after validation)
    pub fn submit(&mut self) -> Result<HashMap<String, FormValue>, String> {
        self.validate()?;
        self.submitted = true;
        Ok(self.values())
    }

    /// Cancel the form
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    /// Check if form is done (submitted or cancelled)
    pub fn is_done(&self) -> bool {
        self.submitted || self.cancelled
    }
}

/// Individual form field types
#[derive(Clone, Debug)]
pub enum FormField {
    TextInput {
        id: String,
        label: String,
        placeholder: String,
        value: String,
        validator: Option<ValidatorFn>,
    },
    TextArea {
        id: String,
        label: String,
        placeholder: String,
        value: String,
        height: usize,
    },
    Email {
        id: String,
        label: String,
        placeholder: String,
        value: String,
    },
    Number {
        id: String,
        label: String,
        value: i64,
        min: Option<i64>,
        max: Option<i64>,
    },
    Select {
        id: String,
        label: String,
        options: Vec<String>,
        selected: usize,
    },
    MultiSelect {
        id: String,
        label: String,
        options: Vec<String>,
        selected: Vec<usize>, // Indices of selected items
    },
    Checkbox {
        id: String,
        label: String,
        checked: bool,
    },
    Path {
        id: String,
        label: String,
        value: String,
        kind: PathKind,
    },
}

impl FormField {
    /// Get field ID and current value
    pub fn get_value(&self) -> (String, FormValue) {
        match self {
            FormField::TextInput { id, value, .. } => (id.clone(), FormValue::Text(value.clone())),
            FormField::TextArea { id, value, .. } => (id.clone(), FormValue::Text(value.clone())),
            FormField::Email { id, value, .. } => (id.clone(), FormValue::Email(value.clone())),
            FormField::Number { id, value, .. } => (id.clone(), FormValue::Number(*value)),
            FormField::Select {
                id,
                options,
                selected,
                ..
            } => (
                id.clone(),
                FormValue::Text(options.get(*selected).cloned().unwrap_or_default()),
            ),
            FormField::MultiSelect {
                id,
                options,
                selected,
                ..
            } => {
                let values: Vec<String> = selected
                    .iter()
                    .filter_map(|&i| options.get(i).cloned())
                    .collect();
                (id.clone(), FormValue::List(values))
            }
            FormField::Checkbox { id, checked, .. } => (id.clone(), FormValue::Bool(*checked)),
            FormField::Path { id, value, .. } => (id.clone(), FormValue::Path(value.clone())),
        }
    }

    /// Get field label
    pub fn label(&self) -> &str {
        match self {
            FormField::TextInput { label, .. } => label,
            FormField::TextArea { label, .. } => label,
            FormField::Email { label, .. } => label,
            FormField::Number { label, .. } => label,
            FormField::Select { label, .. } => label,
            FormField::MultiSelect { label, .. } => label,
            FormField::Checkbox { label, .. } => label,
            FormField::Path { label, .. } => label,
        }
    }

    /// Validate field value
    pub fn validate(&self) -> Result<(), String> {
        match self {
            FormField::TextInput { validator, value, .. } => {
                if let Some(validator) = validator {
                    validator(value)?;
                }
            }
            FormField::Email { value, label, .. } => {
                if !value.is_empty() && !value.contains('@') {
                    return Err(format!("{}: Invalid email format", label));
                }
            }
            FormField::Number {
                value, min, max, label, ..
            } => {
                if let Some(min_val) = min {
                    if value < min_val {
                        return Err(format!("{}: Value must be at least {}", label, min_val));
                    }
                }
                if let Some(max_val) = max {
                    if value > max_val {
                        return Err(format!("{}: Value must be at most {}", label, max_val));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle text input (for text fields)
    pub fn handle_char(&mut self, c: char) {
        match self {
            FormField::TextInput { value, .. } => value.push(c),
            FormField::TextArea { value, .. } => value.push(c),
            FormField::Email { value, .. } => value.push(c),
            _ => {}
        }
    }

    /// Handle backspace (for text fields)
    pub fn handle_backspace(&mut self) {
        match self {
            FormField::TextInput { value, .. } => {
                value.pop();
            }
            FormField::TextArea { value, .. } => {
                value.pop();
            }
            FormField::Email { value, .. } => {
                value.pop();
            }
            _ => {}
        }
    }

    /// Handle up/down navigation (for select fields)
    pub fn handle_up(&mut self) {
        match self {
            FormField::Select { selected, .. } => {
                if *selected > 0 {
                    *selected -= 1;
                }
            }
            FormField::MultiSelect { selected, options, .. } => {
                // For multi-select, this could navigate cursor (not implemented yet)
            }
            FormField::Number { value, .. } => {
                *value += 1;
            }
            _ => {}
        }
    }

    pub fn handle_down(&mut self) {
        match self {
            FormField::Select {
                selected, options, ..
            } => {
                if *selected < options.len().saturating_sub(1) {
                    *selected += 1;
                }
            }
            FormField::MultiSelect { selected, options, .. } => {
                // For multi-select, this could navigate cursor (not implemented yet)
            }
            FormField::Number { value, .. } => {
                *value -= 1;
            }
            _ => {}
        }
    }

    /// Handle space (for checkboxes and multi-select)
    pub fn handle_space(&mut self) {
        match self {
            FormField::Checkbox { checked, .. } => {
                *checked = !*checked;
            }
            FormField::MultiSelect {
                selected, options, ..
            } => {
                // Toggle selection of current item (simplified - would need cursor position)
                if selected.is_empty() {
                    selected.push(0);
                } else {
                    selected.clear();
                }
            }
            _ => {}
        }
    }
}

/// Extracted form values
#[derive(Clone, Debug)]
pub enum FormValue {
    Text(String),
    Email(String),
    Number(i64),
    Bool(bool),
    Path(String),
    List(Vec<String>),
}

impl FormValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            FormValue::Text(s) | FormValue::Email(s) | FormValue::Path(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<i64> {
        match self {
            FormValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FormValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[String]> {
        match self {
            FormValue::List(list) => Some(list),
            _ => None,
        }
    }
}

/// Path field kind
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PathKind {
    File,
    Directory,
    Any,
}

/// Custom validator function type
/// Note: We can't use function pointers in Clone, so validators are optional for now
pub type ValidatorFn = fn(&str) -> Result<(), String>;
