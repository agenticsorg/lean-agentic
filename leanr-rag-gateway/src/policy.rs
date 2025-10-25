//! Policy engine for access control and PII masking

use crate::{RagQuery, GatewayError, AccessCheckResult};

#[derive(Debug, Clone)]
pub enum Policy {
    AllowUser { user_id: String },
    DenyUser { user_id: String },
    RequireSource { pattern: String },
    MaskPII,
    MaxRetention { days: u32 },
}

impl Policy {
    pub fn allow_user(user_id: &str) -> Self {
        Policy::AllowUser { user_id: user_id.to_string() }
    }

    pub fn deny_user(user_id: &str) -> Self {
        Policy::DenyUser { user_id: user_id.to_string() }
    }

    pub fn mask_pii() -> Self {
        Policy::MaskPII
    }
}

#[derive(Debug, Clone)]
pub enum PolicyViolation {
    UserDenied { user_id: String },
    SourceUnauthorized { source: String },
    PIIDetected { field: String },
    RetentionExceeded,
}

pub struct PolicyEngine {
    policies: Vec<Policy>,
}

impl PolicyEngine {
    pub fn new(policies: Vec<Policy>) -> Self {
        Self { policies }
    }

    pub fn check_access(&self, query: &RagQuery) -> Result<AccessCheckResult, GatewayError> {
        for policy in &self.policies {
            match policy {
                Policy::DenyUser { user_id } if user_id == &query.user_id => {
                    return Ok(AccessCheckResult {
                        allowed: false,
                        violation: PolicyViolation::UserDenied {
                            user_id: query.user_id.clone(),
                        },
                    });
                }
                _ => {}
            }
        }

        Ok(AccessCheckResult {
            allowed: true,
            violation: PolicyViolation::UserDenied { user_id: String::new() },
        })
    }

    pub fn mask_pii(&self, text: &str) -> Result<String, GatewayError> {
        let mut masked = text.to_string();

        // Simple PII detection without regex
        // Mask potential SSN patterns (XXX-XX-XXXX)
        masked = mask_ssn_pattern(&masked);

        // Mask potential credit card numbers (16 consecutive digits)
        masked = mask_credit_card_pattern(&masked);

        // Mask potential email addresses
        masked = mask_email_pattern(&masked);

        Ok(masked)
    }
}

fn mask_ssn_pattern(text: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 10 < chars.len() {
            // Check for XXX-XX-XXXX pattern
            if chars[i].is_ascii_digit() &&
               chars[i+1].is_ascii_digit() &&
               chars[i+2].is_ascii_digit() &&
               chars[i+3] == '-' &&
               chars[i+4].is_ascii_digit() &&
               chars[i+5].is_ascii_digit() &&
               chars[i+6] == '-' &&
               chars[i+7].is_ascii_digit() &&
               chars[i+8].is_ascii_digit() &&
               chars[i+9].is_ascii_digit() &&
               chars[i+10].is_ascii_digit() {
                result.push_str("[REDACTED]");
                i += 11;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

fn mask_credit_card_pattern(text: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 15 < chars.len() {
            // Check for 16 consecutive digits
            let is_card = (0..16).all(|j| chars[i+j].is_ascii_digit());
            if is_card {
                result.push_str("[REDACTED]");
                i += 16;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

fn mask_email_pattern(text: &str) -> String {
    // Simple email detection: look for word@word.word pattern
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut result = String::new();

    for (idx, word) in words.iter().enumerate() {
        if word.contains('@') && word.contains('.') {
            let parts: Vec<&str> = word.split('@').collect();
            if parts.len() == 2 && parts[1].contains('.') {
                result.push_str("[REDACTED]");
            } else {
                result.push_str(word);
            }
        } else {
            result.push_str(word);
        }

        if idx < words.len() - 1 {
            result.push(' ');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pii_masking() {
        let engine = PolicyEngine::new(vec![Policy::MaskPII]);

        let text = "My SSN is 123-45-6789 and email is user@example.com";
        let masked = engine.mask_pii(text).unwrap();

        assert!(!masked.contains("123-45-6789"));
        assert!(!masked.contains("user@example.com"));
        assert!(masked.contains("[REDACTED]"));
    }

    #[test]
    fn test_access_control() {
        let engine = PolicyEngine::new(vec![Policy::deny_user("blocked")]);

        let query = RagQuery {
            question: "test".to_string(),
            sources: vec![],
            user_id: "blocked".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        let result = engine.check_access(&query).unwrap();
        assert!(!result.allowed);
    }
}
