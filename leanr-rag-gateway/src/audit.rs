//! Audit logging for RAG Gateway

use crate::{RagQuery, GatewayError};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum AuditEvent {
    RequestSuccess {
        user_id: String,
        question: String,
        latency_ms: u64,
        cost_usd: f64,
        lane_used: String,
        timestamp: i64,
    },
    RequestBlocked {
        user_id: String,
        question: String,
        violation: String,
        timestamp: i64,
    },
    PIIMasked {
        user_id: String,
        count: usize,
        timestamp: i64,
    },
}

impl AuditEvent {
    pub fn is_success(&self) -> bool {
        matches!(self, AuditEvent::RequestSuccess { .. })
    }

    pub fn is_blocked(&self) -> bool {
        matches!(self, AuditEvent::RequestBlocked { .. })
    }
}

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
    fn test_audit_logging() {
        let log = AuditLog::new();

        let query = RagQuery {
            question: "Test".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        log.log_success(&query, 100, 0.01, "local");
        log.log_blocked(&query, "Policy violation".to_string());

        let events = log.events();
        assert_eq!(events.len(), 2);
        assert!(events[0].is_success());
        assert!(events[1].is_blocked());
    }

    #[test]
    fn test_export_report() {
        let log = AuditLog::new();

        let query = RagQuery {
            question: "Test".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        log.log_success(&query, 100, 0.01, "local");
        log.log_blocked(&query, "Test block".to_string());

        let report = log.export_compliance_report().unwrap();
        assert!(report.contains("Audit Log Report"));
        assert!(report.contains("SUCCESS"));
        assert!(report.contains("BLOCKED"));
    }
}
