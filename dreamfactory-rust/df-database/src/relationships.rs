use crate::{DatabaseError, DatabaseService};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Database relationship types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    #[serde(rename = "belongs_to")]
    BelongsTo,
    #[serde(rename = "has_one")]
    HasOne,
    #[serde(rename = "has_many")]
    HasMany,
    #[serde(rename = "many_to_many")]
    ManyToMany,
}

/// Database relationship definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub name: String,
    pub label: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub relationship_type: RelationshipType,
    pub field: String,
    pub ref_service: String,
    pub ref_table: String,
    pub ref_field: String,
    pub ref_on_update: Option<String>,
    pub ref_on_delete: Option<String>,
    pub junction_table: Option<String>,
    pub junction_field: Option<String>,
    pub junction_ref_field: Option<String>,
    pub always_fetch: Option<bool>,
    pub flatten: Option<bool>,
}

/// Virtual relationship that doesn't exist in database schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualRelationship {
    pub name: String,
    pub label: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub relationship_type: RelationshipType,
    pub field: String,
    pub ref_service: String,
    pub ref_table: String,
    pub ref_field: String,
    pub junction_table: Option<String>,
    pub junction_field: Option<String>,
    pub junction_ref_field: Option<String>,
    pub always_fetch: Option<bool>,
    pub flatten: Option<bool>,
}

/// Relationship resolver for executing relationship queries
pub struct RelationshipResolver {
    database_service: Box<dyn DatabaseService>,
}

impl RelationshipResolver {
    pub fn new(database_service: Box<dyn DatabaseService>) -> Self {
        Self { database_service }
    }

    /// Execute all relationships for a set of records
    pub async fn execute_relationships(
        &self,
        table: &str,
        records: &mut Vec<Value>,
        relationships: &[String],
    ) -> Result<(), DatabaseError> {
        if relationships.is_empty() || records.is_empty() {
            return Ok(());
        }

        // Get all available relationships for the table
        let available_relationships = self.database_service.get_relationships(table).await?;
        
        for relationship_name in relationships {
            if let Some(relationship) = available_relationships
                .iter()
                .find(|r| r.name == *relationship_name)
            {
                self.execute_single_relationship(table, records, relationship)
                    .await?;
            }
        }

        Ok(())
    }

    /// Execute a single relationship
    async fn execute_single_relationship(
        &self,
        _table: &str,
        records: &mut Vec<Value>,
        relationship: &Relationship,
    ) -> Result<(), DatabaseError> {
        match relationship.relationship_type {
            RelationshipType::BelongsTo => {
                self.execute_belongs_to(records, relationship).await?;
            }
            RelationshipType::HasOne => {
                self.execute_has_one(records, relationship).await?;
            }
            RelationshipType::HasMany => {
                self.execute_has_many(records, relationship).await?;
            }
            RelationshipType::ManyToMany => {
                self.execute_many_to_many(records, relationship).await?;
            }
        }

        Ok(())
    }

    /// Execute belongs_to relationship
    async fn execute_belongs_to(
        &self,
        records: &mut Vec<Value>,
        relationship: &Relationship,
    ) -> Result<(), DatabaseError> {
        // Collect all foreign key values
        let mut foreign_keys = Vec::new();
        for record in records.iter() {
            if let Some(fk_value) = record.get(&relationship.field) {
                if !fk_value.is_null() {
                    foreign_keys.push(fk_value.clone());
                }
            }
        }

        if foreign_keys.is_empty() {
            return Ok(());
        }

        // Build query to fetch related records
        let filter = format!(
            "{} IN ({})",
            relationship.ref_field,
            foreign_keys
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(",")
        );

        let params = crate::QueryParams {
            filter: Some(filter),
            ..Default::default()
        };

        let related_result = self
            .database_service
            .get_records(&relationship.ref_table, &params)
            .await?;

        // Create lookup map
        let mut lookup = HashMap::new();
        for related_record in related_result.resource {
            if let Some(key_value) = related_record.get(&relationship.ref_field) {
                lookup.insert(key_value.clone(), related_record);
            }
        }

        // Attach related records to main records
        for record in records.iter_mut() {
            if let Some(fk_value) = record.get(&relationship.field) {
                if let Some(related_record) = lookup.get(fk_value) {
                    if let Some(record_obj) = record.as_object_mut() {
                        if relationship.flatten.unwrap_or(false) {
                            // Flatten related fields into main record
                            if let Some(related_obj) = related_record.as_object() {
                                for (key, value) in related_obj {
                                    let prefixed_key = format!("{}_{}", relationship.name, key);
                                    record_obj.insert(prefixed_key, value.clone());
                                }
                            }
                        } else {
                            record_obj.insert(relationship.name.clone(), related_record.clone());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute has_one relationship
    async fn execute_has_one(
        &self,
        records: &mut Vec<Value>,
        relationship: &Relationship,
    ) -> Result<(), DatabaseError> {
        // Collect all primary key values
        let mut primary_keys = Vec::new();
        for record in records.iter() {
            if let Some(pk_value) = record.get(&relationship.field) {
                if !pk_value.is_null() {
                    primary_keys.push(pk_value.clone());
                }
            }
        }

        if primary_keys.is_empty() {
            return Ok(());
        }

        // Build query to fetch related records
        let filter = format!(
            "{} IN ({})",
            relationship.ref_field,
            primary_keys
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(",")
        );

        let params = crate::QueryParams {
            filter: Some(filter),
            ..Default::default()
        };

        let related_result = self
            .database_service
            .get_records(&relationship.ref_table, &params)
            .await?;

        // Create lookup map
        let mut lookup = HashMap::new();
        for related_record in related_result.resource {
            if let Some(key_value) = related_record.get(&relationship.ref_field) {
                lookup.insert(key_value.clone(), related_record);
            }
        }

        // Attach related records to main records
        for record in records.iter_mut() {
            if let Some(pk_value) = record.get(&relationship.field) {
                if let Some(related_record) = lookup.get(pk_value) {
                    if let Some(record_obj) = record.as_object_mut() {
                        record_obj.insert(relationship.name.clone(), related_record.clone());
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute has_many relationship
    async fn execute_has_many(
        &self,
        records: &mut Vec<Value>,
        relationship: &Relationship,
    ) -> Result<(), DatabaseError> {
        // Collect all primary key values
        let mut primary_keys = Vec::new();
        for record in records.iter() {
            if let Some(pk_value) = record.get(&relationship.field) {
                if !pk_value.is_null() {
                    primary_keys.push(pk_value.clone());
                }
            }
        }

        if primary_keys.is_empty() {
            return Ok(());
        }

        // Build query to fetch related records
        let filter = format!(
            "{} IN ({})",
            relationship.ref_field,
            primary_keys
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(",")
        );

        let params = crate::QueryParams {
            filter: Some(filter),
            ..Default::default()
        };

        let related_result = self
            .database_service
            .get_records(&relationship.ref_table, &params)
            .await?;

        // Group related records by foreign key
        let mut grouped = HashMap::new();
        for related_record in related_result.resource {
            if let Some(key_value) = related_record.get(&relationship.ref_field) {
                grouped
                    .entry(key_value.clone())
                    .or_insert_with(Vec::new)
                    .push(related_record);
            }
        }

        // Attach related record arrays to main records
        for record in records.iter_mut() {
            if let Some(pk_value) = record.get(&relationship.field) {
                let related_records = grouped.get(pk_value).cloned().unwrap_or_default();
                if let Some(record_obj) = record.as_object_mut() {
                    record_obj.insert(
                        relationship.name.clone(),
                        Value::Array(related_records.into_iter().collect()),
                    );
                }
            }
        }

        Ok(())
    }

    /// Execute many_to_many relationship
    async fn execute_many_to_many(
        &self,
        records: &mut Vec<Value>,
        relationship: &Relationship,
    ) -> Result<(), DatabaseError> {
        let junction_table = relationship
            .junction_table
            .as_ref()
            .ok_or_else(|| DatabaseError::Relationship {
                message: "Junction table required for many-to-many relationship".to_string(),
            })?;

        let junction_field = relationship
            .junction_field
            .as_ref()
            .ok_or_else(|| DatabaseError::Relationship {
                message: "Junction field required for many-to-many relationship".to_string(),
            })?;

        let junction_ref_field = relationship
            .junction_ref_field
            .as_ref()
            .ok_or_else(|| DatabaseError::Relationship {
                message: "Junction ref field required for many-to-many relationship".to_string(),
            })?;

        // Collect all primary key values
        let mut primary_keys = Vec::new();
        for record in records.iter() {
            if let Some(pk_value) = record.get(&relationship.field) {
                if !pk_value.is_null() {
                    primary_keys.push(pk_value.clone());
                }
            }
        }

        if primary_keys.is_empty() {
            return Ok(());
        }

        // First, get junction records
        let junction_filter = format!(
            "{} IN ({})",
            junction_field,
            primary_keys
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(",")
        );

        let junction_params = crate::QueryParams {
            filter: Some(junction_filter),
            ..Default::default()
        };

        let junction_result = self
            .database_service
            .get_records(junction_table, &junction_params)
            .await?;

        // Extract related IDs from junction records
        let mut related_ids = Vec::new();
        let mut junction_map = HashMap::new();

        for junction_record in junction_result.resource {
            if let (Some(local_id), Some(related_id)) = (
                junction_record.get(junction_field),
                junction_record.get(junction_ref_field),
            ) {
                related_ids.push(related_id.clone());
                junction_map
                    .entry(local_id.clone())
                    .or_insert_with(Vec::new)
                    .push(related_id.clone());
            }
        }

        if related_ids.is_empty() {
            return Ok(());
        }

        // Get related records
        let related_filter = format!(
            "{} IN ({})",
            relationship.ref_field,
            related_ids
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(",")
        );

        let related_params = crate::QueryParams {
            filter: Some(related_filter),
            ..Default::default()
        };

        let related_result = self
            .database_service
            .get_records(&relationship.ref_table, &related_params)
            .await?;

        // Create lookup map for related records
        let mut related_lookup = HashMap::new();
        for related_record in related_result.resource {
            if let Some(key_value) = related_record.get(&relationship.ref_field) {
                related_lookup.insert(key_value.clone(), related_record);
            }
        }

        // Attach related records to main records
        for record in records.iter_mut() {
            if let Some(pk_value) = record.get(&relationship.field) {
                let mut related_records = Vec::new();
                
                if let Some(related_ids) = junction_map.get(pk_value) {
                    for related_id in related_ids {
                        if let Some(related_record) = related_lookup.get(related_id) {
                            related_records.push(related_record.clone());
                        }
                    }
                }

                if let Some(record_obj) = record.as_object_mut() {
                    record_obj.insert(
                        relationship.name.clone(),
                        Value::Array(related_records),
                    );
                }
            }
        }

        Ok(())
    }
}

/// Virtual relationship manager
pub struct VirtualRelationshipManager {
    relationships: HashMap<String, Vec<VirtualRelationship>>,
}

impl VirtualRelationshipManager {
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
        }
    }

    /// Add virtual relationship for a table
    pub fn add_virtual_relationship(&mut self, table: &str, relationship: VirtualRelationship) {
        self.relationships
            .entry(table.to_string())
            .or_insert_with(Vec::new)
            .push(relationship);
    }

    /// Get virtual relationships for a table
    pub fn get_virtual_relationships(&self, table: &str) -> Vec<&VirtualRelationship> {
        self.relationships
            .get(table)
            .map(|rels| rels.iter().collect())
            .unwrap_or_default()
    }

    /// Convert virtual relationship to regular relationship
    pub fn to_relationship(&self, virtual_rel: &VirtualRelationship) -> Relationship {
        Relationship {
            name: virtual_rel.name.clone(),
            label: virtual_rel.label.clone(),
            description: virtual_rel.description.clone(),
            relationship_type: virtual_rel.relationship_type.clone(),
            field: virtual_rel.field.clone(),
            ref_service: virtual_rel.ref_service.clone(),
            ref_table: virtual_rel.ref_table.clone(),
            ref_field: virtual_rel.ref_field.clone(),
            ref_on_update: None,
            ref_on_delete: None,
            junction_table: virtual_rel.junction_table.clone(),
            junction_field: virtual_rel.junction_field.clone(),
            junction_ref_field: virtual_rel.junction_ref_field.clone(),
            always_fetch: virtual_rel.always_fetch,
            flatten: virtual_rel.flatten,
        }
    }
}