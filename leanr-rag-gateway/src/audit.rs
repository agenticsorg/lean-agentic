//! Audit logging for compliance and security

use crate::{GatewayError, RagQuery};
use std::sync::{Arc, Mutex};

/// Audit event types
#[derive(Debug, Clone)]
pub enum AuditEvent {
    /// Request was blocked by policy
    RequestBlocked {
        user_id: String,
        question: String,
        violation: String,
        timestamp: i64,
    },

    /// Request succeeded
    RequestSuccess {
        user_id: String,
        question: String,
        latency_ms: u64,
        cost_usd: f64,
        lane_used: String,
        timestamp: i64,
    },

    /// PII was masked in response
    PIIMasked {
        user_id: String,
        count: usize,
        timestamp: i64,
    },
}

/// Audit log for compliance tracking
pub struct AuditLog {
    events: Arc<Mutex<Vec<AuditEvent>>>,
}

impl AuditLog {
    /// Create a new audit log
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Log a blocked request
    pub fn log_blocked(&self, query: &RagQuery, violation: String) {
        let event = AuditEvent::RequestBlocked {
            user_id: query.user_id.clone(),
            question: query.question.clone(),
            violation,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    /// Log a successful request
    pub fn log_success(&self, query: &RagQuery, latency_ms: u64, cost_usd: f64, lane: &str) {
        let event = AuditEvent::RequestSuccess {
            user_id: query.user_id.clone(),
            question: query.question.clone(),
            latency_ms,
            cost_usd,
            lane_used: lane.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    /// Log PII masking
    pub fn log_pii_masked(&self, user_id: &str, count: usize) {
        let event = AuditEvent::PIIMasked {
            user_id: user_id.to_string(),
            count,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    /// Export compliance report as text
    pub fn export_compliance_report(&self) -> Result<String, GatewayError> {
        let events = self.events.lock().unwrap();
        let mut report = String::from("Audit Log Report\n================\n\n");

        for event in events.iter() {
            match event {
                AuditEvent::RequestBlocked { user_id, question, violation, timestamp } => {
                    report.push_str(&format!(
                        "[{}] BLOCKED - User: {}, Question: {}, Violation: {}\n",
                        timestamp, user_id, question, violation
                    ));
                }
                AuditEvent::RequestSuccess { user_id, question, latency_ms, cost_usd, lane_used, timestamp } => {
                    report.push_str(&format!(
                        "[{}] SUCCESS - User: {}, Lane: {}, Latency: {}ms, Cost: ${:.4}\n",
                        timestamp, user_id, lane_used, latency_ms, cost_usd
                    ));
                }
                AuditEvent::PIIMasked { user_id, count, timestamp } => {
                    report.push_str(&format!(
                        "[{}] PII_MASKED - User: {}, Count: {}\n",
                        timestamp, user_id, count
                    ));
                }
            }
        }

        Ok(report)
    }

    /// Get all audit events
    pub fn events(&self) -> Vec<AuditEvent> {
        let events = self.events.lock().unwrap();
        events.clone()
    }

    /// Get count of blocked requests
    pub fn blocked_count(&self) -> usize {
        let events = self.events.lock().unwrap();
        events.iter().filter(|e| matches!(e, AuditEvent::RequestBlocked { .. })).count()
    }

    /// Get count of successful requests
    pub fn success_count(&self) -> usize {
        let events = self.events.lock().unwrap();
        events.iter().filter(|e| matches!(e, AuditEvent::RequestSuccess { .. })).count()
    }

    /// Clear audit log (for testing)
    #[cfg(test)]
    pub fn clear(&self) {
        let mut events = self.events.lock().unwrap();
        events.clear();
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log() {
        let log = AuditLog::new();

        let query = RagQuery {
            question: "Test question".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        log.log_blocked(&query, "Policy violation".to_string());
        assert_eq!(log.blocked_count(), 1);

        log.log_success(&query, 100, 0.001, "local");
        assert_eq!(log.success_count(), 1);

        let report = log.export_compliance_report().unwrap();
        assert!(report.contains("user123"));
    }
}
