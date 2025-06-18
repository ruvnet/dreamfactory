use crate::DatabaseError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Computed field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedField {
    pub name: String,
    pub label: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub field_type: String,
    pub expression: String,
    pub depends_on: Vec<String>,
}

/// Computed field expression types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionType {
    Concat,
    Math,
    Conditional,
    Format,
    Aggregate,
    Custom,
}

/// Computed field evaluator
pub struct ComputedFieldEvaluator {
    fields: HashMap<String, Vec<ComputedField>>,
}

impl ComputedFieldEvaluator {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Add computed field for a table
    pub fn add_computed_field(&mut self, table: &str, field: ComputedField) {
        self.fields
            .entry(table.to_string())
            .or_insert_with(Vec::new)
            .push(field);
    }

    /// Get computed fields for a table
    pub fn get_computed_fields(&self, table: &str) -> Vec<&ComputedField> {
        self.fields
            .get(table)
            .map(|fields| fields.iter().collect())
            .unwrap_or_default()
    }

    /// Evaluate all computed fields for records
    pub async fn evaluate_computed_fields(
        &self,
        table: &str,
        records: &mut Vec<Value>,
    ) -> Result<(), DatabaseError> {
        let computed_fields = self.get_computed_fields(table);
        
        if computed_fields.is_empty() {
            return Ok(());
        }

        for record in records.iter_mut() {
            for computed_field in &computed_fields {
                let computed_value = self.evaluate_expression(record, computed_field)?;
                
                if let Some(record_obj) = record.as_object_mut() {
                    record_obj.insert(computed_field.name.clone(), computed_value);
                }
            }
        }

        Ok(())
    }

    /// Evaluate a single computed field expression
    fn evaluate_expression(
        &self,
        record: &Value,
        computed_field: &ComputedField,
    ) -> Result<Value, DatabaseError> {
        let expression_type = self.determine_expression_type(&computed_field.expression);
        
        match expression_type {
            ExpressionType::Concat => self.evaluate_concat(record, computed_field),
            ExpressionType::Math => self.evaluate_math(record, computed_field),
            ExpressionType::Conditional => self.evaluate_conditional(record, computed_field),
            ExpressionType::Format => self.evaluate_format(record, computed_field),
            ExpressionType::Aggregate => self.evaluate_aggregate(record, computed_field),
            ExpressionType::Custom => self.evaluate_custom(record, computed_field),
        }
    }

    /// Determine expression type from expression string
    fn determine_expression_type(&self, expression: &str) -> ExpressionType {
        let expr_lower = expression.to_lowercase();
        
        if expr_lower.contains("concat") || expr_lower.contains("||") {
            ExpressionType::Concat
        } else if expr_lower.contains("+") || expr_lower.contains("-") || 
                 expr_lower.contains("*") || expr_lower.contains("/") {
            ExpressionType::Math
        } else if expr_lower.contains("if") || expr_lower.contains("case") ||
                 expr_lower.contains("when") {
            ExpressionType::Conditional
        } else if expr_lower.contains("format") || expr_lower.contains("sprintf") {
            ExpressionType::Format
        } else if expr_lower.contains("sum") || expr_lower.contains("count") ||
                 expr_lower.contains("avg") || expr_lower.contains("max") ||
                 expr_lower.contains("min") {
            ExpressionType::Aggregate
        } else {
            ExpressionType::Custom
        }
    }

    /// Evaluate concatenation expression
    fn evaluate_concat(
        &self,
        record: &Value,
        computed_field: &ComputedField,
    ) -> Result<Value, DatabaseError> {
        let expression = &computed_field.expression;
        
        // Simple concatenation: "field1 || ' ' || field2"
        if expression.contains("||") {
            let parts: Vec<&str> = expression.split("||").collect();
            let mut result = String::new();
            
            for part in parts {
                let part = part.trim();
                if part.starts_with('\'') && part.ends_with('\'') {
                    // String literal
                    result.push_str(&part[1..part.len()-1]);
                } else if part.starts_with('"') && part.ends_with('"') {
                    // String literal
                    result.push_str(&part[1..part.len()-1]);
                } else {
                    // Field reference
                    if let Some(field_value) = record.get(part) {
                        result.push_str(&field_value.to_string().trim_matches('"'));
                    }
                }
            }
            
            return Ok(Value::String(result));
        }
        
        // CONCAT function: "CONCAT(field1, ' ', field2)"
        if expression.starts_with("CONCAT(") && expression.ends_with(")") {
            let inner = &expression[7..expression.len()-1];
            let parts: Vec<&str> = inner.split(',').collect();
            let mut result = String::new();
            
            for part in parts {
                let part = part.trim();
                if part.starts_with('\'') && part.ends_with('\'') {
                    result.push_str(&part[1..part.len()-1]);
                } else if part.starts_with('"') && part.ends_with('"') {
                    result.push_str(&part[1..part.len()-1]);
                } else {
                    if let Some(field_value) = record.get(part) {
                        result.push_str(&field_value.to_string().trim_matches('"'));
                    }
                }
            }
            
            return Ok(Value::String(result));
        }
        
        // Default: return expression as-is
        Ok(Value::String(expression.clone()))
    }

    /// Evaluate mathematical expression
    fn evaluate_math(
        &self,
        record: &Value,
        computed_field: &ComputedField,
    ) -> Result<Value, DatabaseError> {
        let expression = &computed_field.expression;
        
        // Simple math operations: "field1 + field2", "field1 * 2", etc.
        if expression.contains('+') {
            let parts: Vec<&str> = expression.split('+').collect();
            if parts.len() == 2 {
                let left = self.get_numeric_value(record, parts[0].trim())?;
                let right = self.get_numeric_value(record, parts[1].trim())?;
                return Ok(Value::Number(serde_json::Number::from_f64(left + right).unwrap()));
            }
        }
        
        if expression.contains('-') {
            let parts: Vec<&str> = expression.split('-').collect();
            if parts.len() == 2 {
                let left = self.get_numeric_value(record, parts[0].trim())?;
                let right = self.get_numeric_value(record, parts[1].trim())?;
                return Ok(Value::Number(serde_json::Number::from_f64(left - right).unwrap()));
            }
        }
        
        if expression.contains('*') {
            let parts: Vec<&str> = expression.split('*').collect();
            if parts.len() == 2 {
                let left = self.get_numeric_value(record, parts[0].trim())?;
                let right = self.get_numeric_value(record, parts[1].trim())?;
                return Ok(Value::Number(serde_json::Number::from_f64(left * right).unwrap()));
            }
        }
        
        if expression.contains('/') {
            let parts: Vec<&str> = expression.split('/').collect();
            if parts.len() == 2 {
                let left = self.get_numeric_value(record, parts[0].trim())?;
                let right = self.get_numeric_value(record, parts[1].trim())?;
                if right != 0.0 {
                    return Ok(Value::Number(serde_json::Number::from_f64(left / right).unwrap()));
                } else {
                    return Err(DatabaseError::ComputedField {
                        message: "Division by zero".to_string(),
                    });
                }
            }
        }
        
        // Default: return 0
        Ok(Value::Number(serde_json::Number::from(0)))
    }

    /// Evaluate conditional expression
    fn evaluate_conditional(
        &self,
        record: &Value,
        computed_field: &ComputedField,
    ) -> Result<Value, DatabaseError> {
        let expression = &computed_field.expression;
        
        // Simple IF: "IF(field1 > 0, 'positive', 'negative')"
        if expression.starts_with("IF(") && expression.ends_with(")") {
            let inner = &expression[3..expression.len()-1];
            let parts: Vec<&str> = inner.splitn(3, ',').collect();
            
            if parts.len() == 3 {
                let condition = parts[0].trim();
                let true_value = parts[1].trim();
                let false_value = parts[2].trim();
                
                let condition_result = self.evaluate_condition(record, condition)?;
                
                if condition_result {
                    return self.parse_value(record, true_value);
                } else {
                    return self.parse_value(record, false_value);
                }
            }
        }
        
        // Default: return null
        Ok(Value::Null)
    }

    /// Evaluate format expression
    fn evaluate_format(
        &self,
        record: &Value,
        computed_field: &ComputedField,
    ) -> Result<Value, DatabaseError> {
        let expression = &computed_field.expression;
        
        // Simple format: "FORMAT('Hello %s', field1)"
        if expression.starts_with("FORMAT(") && expression.ends_with(")") {
            let inner = &expression[7..expression.len()-1];
            let parts: Vec<&str> = inner.splitn(2, ',').collect();
            
            if parts.len() == 2 {
                let format_str = parts[0].trim().trim_matches('\'').trim_matches('"');
                let field_name = parts[1].trim();
                
                if let Some(field_value) = record.get(field_name) {
                    let formatted = format_str.replace("%s", &field_value.to_string().trim_matches('"'));
                    return Ok(Value::String(formatted));
                }
            }
        }
        
        // Default: return expression as-is
        Ok(Value::String(expression.clone()))
    }

    /// Evaluate aggregate expression (for single record, this is limited)
    fn evaluate_aggregate(
        &self,
        record: &Value,
        computed_field: &ComputedField,
    ) -> Result<Value, DatabaseError> {
        // For single record aggregates, we can only do basic operations
        // Full aggregates would require database queries
        
        let expression = &computed_field.expression;
        
        // COUNT: "COUNT(field1)" - returns 1 if field exists and is not null, 0 otherwise
        if expression.starts_with("COUNT(") && expression.ends_with(")") {
            let field_name = &expression[6..expression.len()-1];
            if let Some(field_value) = record.get(field_name) {
                if field_value.is_null() {
                    return Ok(Value::Number(serde_json::Number::from(0)));
                } else {
                    return Ok(Value::Number(serde_json::Number::from(1)));
                }
            }
        }
        
        // Default: return null
        Ok(Value::Null)
    }

    /// Evaluate custom expression
    fn evaluate_custom(
        &self,
        record: &Value,
        computed_field: &ComputedField,
    ) -> Result<Value, DatabaseError> {
        let expression = &computed_field.expression;
        
        // Check if it's a simple field reference
        if let Some(field_value) = record.get(expression) {
            return Ok(field_value.clone());
        }
        
        // Check if it's a string literal
        if expression.starts_with('\'') && expression.ends_with('\'') {
            return Ok(Value::String(expression[1..expression.len()-1].to_string()));
        }
        
        if expression.starts_with('"') && expression.ends_with('"') {
            return Ok(Value::String(expression[1..expression.len()-1].to_string()));
        }
        
        // Check if it's a numeric literal
        if let Ok(num) = expression.parse::<i64>() {
            return Ok(Value::Number(serde_json::Number::from(num)));
        }
        
        if let Ok(num) = expression.parse::<f64>() {
            return Ok(Value::Number(serde_json::Number::from_f64(num).unwrap()));
        }
        
        // Default: return expression as string
        Ok(Value::String(expression.clone()))
    }

    /// Get numeric value from field or literal
    fn get_numeric_value(&self, record: &Value, input: &str) -> Result<f64, DatabaseError> {
        // Try parsing as number first
        if let Ok(num) = input.parse::<f64>() {
            return Ok(num);
        }
        
        // Try getting from record
        if let Some(field_value) = record.get(input) {
            match field_value {
                Value::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        return Ok(f);
                    } else if let Some(i) = n.as_i64() {
                        return Ok(i as f64);
                    }
                }
                Value::String(s) => {
                    if let Ok(num) = s.parse::<f64>() {
                        return Ok(num);
                    }
                }
                _ => {}
            }
        }
        
        Err(DatabaseError::ComputedField {
            message: format!("Cannot convert '{}' to numeric value", input),
        })
    }

    /// Evaluate condition for conditional expressions
    fn evaluate_condition(&self, record: &Value, condition: &str) -> Result<bool, DatabaseError> {
        // Simple conditions: "field1 > 0", "field1 = 'value'", etc.
        if condition.contains('>') {
            let parts: Vec<&str> = condition.split('>').collect();
            if parts.len() == 2 {
                let left = self.get_numeric_value(record, parts[0].trim())?;
                let right = self.get_numeric_value(record, parts[1].trim())?;
                return Ok(left > right);
            }
        }
        
        if condition.contains('<') {
            let parts: Vec<&str> = condition.split('<').collect();
            if parts.len() == 2 {
                let left = self.get_numeric_value(record, parts[0].trim())?;
                let right = self.get_numeric_value(record, parts[1].trim())?;
                return Ok(left < right);
            }
        }
        
        if condition.contains('=') {
            let parts: Vec<&str> = condition.split('=').collect();
            if parts.len() == 2 {
                let left_str = parts[0].trim();
                let right_str = parts[1].trim();
                
                let left_val = self.parse_value(record, left_str)?;
                let right_val = self.parse_value(record, right_str)?;
                
                return Ok(left_val == right_val);
            }
        }
        
        // Default: false
        Ok(false)
    }

    /// Parse value from string (field reference or literal)
    fn parse_value(&self, record: &Value, input: &str) -> Result<Value, DatabaseError> {
        let input = input.trim();
        
        // String literal
        if input.starts_with('\'') && input.ends_with('\'') {
            return Ok(Value::String(input[1..input.len()-1].to_string()));
        }
        
        if input.starts_with('"') && input.ends_with('"') {
            return Ok(Value::String(input[1..input.len()-1].to_string()));
        }
        
        // Numeric literal
        if let Ok(num) = input.parse::<i64>() {
            return Ok(Value::Number(serde_json::Number::from(num)));
        }
        
        if let Ok(num) = input.parse::<f64>() {
            return Ok(Value::Number(serde_json::Number::from_f64(num).unwrap()));
        }
        
        // Boolean literal
        if input.eq_ignore_ascii_case("true") {
            return Ok(Value::Bool(true));
        }
        
        if input.eq_ignore_ascii_case("false") {
            return Ok(Value::Bool(false));
        }
        
        // Null literal
        if input.eq_ignore_ascii_case("null") {
            return Ok(Value::Null);
        }
        
        // Field reference
        if let Some(field_value) = record.get(input) {
            return Ok(field_value.clone());
        }
        
        // Default: return as string
        Ok(Value::String(input.to_string()))
    }
}