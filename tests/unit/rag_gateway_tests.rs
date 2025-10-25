//! Comprehensive unit tests for RAG Gateway
//!
//! Tests cover:
//! - Policy enforcement
//! - PII masking
//! - Cost-aware routing
//! - Proof generation
//! - Audit logging

#[cfg(test)]
mod rag_gateway_tests {
    use leanr_rag_gateway::{
        RagGateway, RagQuery, Policy, PolicyViolation, GatewayError,
    };

    // ============================================================================
    // Policy Enforcement Tests
    // ============================================================================

    #[test]
    fn test_allow_user_policy() {
        let policies = vec![Policy::allow_user("user123")];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "What is the refund policy?".to_string(),
            sources: vec!["docs/policies.txt".to_string()],
            user_id: "user123".to_string(),
            latency_sla: Some(150),
            cost_budget: Some(0.01),
        };

        let result = gateway.process(query);
        assert!(result.is_ok(), "Allowed user should succeed");

        let response = result.unwrap();
        assert!(response.metrics.latency_ms < 200, "Latency should be reasonable");
        assert!(!response.proof.claims.is_empty(), "Proof should have claims");
    }

    #[test]
    fn test_deny_user_policy() {
        let policies = vec![Policy::deny_user("blocked_user")];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "What is the refund policy?".to_string(),
            sources: vec!["docs/policies.txt".to_string()],
            user_id: "blocked_user".to_string(),
            latency_sla: Some(150),
            cost_budget: Some(0.01),
        };

        let result = gateway.process(query);
        assert!(result.is_err(), "Blocked user should be denied");

        match result.unwrap_err() {
            GatewayError::PolicyViolation(PolicyViolation::UserDenied { user_id }) => {
                assert_eq!(user_id, "blocked_user");
            }
            _ => panic!("Expected PolicyViolation::UserDenied"),
        }
    }

    #[test]
    fn test_multiple_policies() {
        let policies = vec![
            Policy::allow_user("alice"),
            Policy::allow_user("bob"),
            Policy::deny_user("eve"),
            Policy::mask_pii(),
        ];
        let mut gateway = RagGateway::new(policies);

        // Alice should succeed
        let query_alice = RagQuery {
            question: "Test question".to_string(),
            sources: vec![],
            user_id: "alice".to_string(),
            latency_sla: None,
            cost_budget: None,
        };
        assert!(gateway.process(query_alice).is_ok());

        // Eve should be blocked
        let query_eve = RagQuery {
            question: "Test question".to_string(),
            sources: vec![],
            user_id: "eve".to_string(),
            latency_sla: None,
            cost_budget: None,
        };
        assert!(gateway.process(query_eve).is_err());
    }

    // ============================================================================
    // PII Masking Tests
    // ============================================================================

    #[test]
    fn test_ssn_masking() {
        let policies = vec![Policy::mask_pii()];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "My SSN is 123-45-6789".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        let response = gateway.process(query).unwrap();
        assert!(
            !response.answer.contains("123-45-6789"),
            "SSN should be masked"
        );
    }

    #[test]
    fn test_email_masking() {
        let policies = vec![Policy::mask_pii()];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "Contact me at john.doe@example.com".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        let response = gateway.process(query).unwrap();
        assert!(
            !response.answer.contains("john.doe@example.com"),
            "Email should be masked"
        );
    }

    #[test]
    fn test_credit_card_masking() {
        let policies = vec![Policy::mask_pii()];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "My card is 4532111111111111".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        let response = gateway.process(query).unwrap();
        assert!(
            !response.answer.contains("4532111111111111"),
            "Credit card should be masked"
        );
    }

    #[test]
    fn test_multiple_pii_in_single_text() {
        let policies = vec![Policy::mask_pii()];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "SSN: 123-45-6789, Email: user@test.com, Card: 4532111111111111".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        let response = gateway.process(query).unwrap();
        assert!(!response.answer.contains("123-45-6789"));
        assert!(!response.answer.contains("user@test.com"));
        assert!(!response.answer.contains("4532111111111111"));
    }

    // ============================================================================
    // Cost-Aware Routing Tests
    // ============================================================================

    #[test]
    fn test_low_latency_routing() {
        let policies = vec![Policy::allow_user("user123")];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "Quick question".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: Some(50), // Very tight SLA
            cost_budget: Some(0.1),
        };

        let response = gateway.process(query).unwrap();
        assert_eq!(response.metrics.lane_used, "local", "Should use local lane for low latency");
        assert!(response.metrics.latency_ms < 100);
    }

    #[test]
    fn test_cost_budget_routing() {
        let policies = vec![Policy::allow_user("user123")];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "Budget-conscious question".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: Some(300),
            cost_budget: Some(0.001), // Very tight budget
        };

        let response = gateway.process(query).unwrap();
        assert!(
            response.metrics.cost_usd <= 0.001,
            "Should respect cost budget"
        );
    }

    #[test]
    fn test_impossible_sla_fails() {
        let policies = vec![Policy::allow_user("user123")];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "Impossible requirements".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: Some(1), // Impossible latency
            cost_budget: Some(0.0), // Zero cost
        };

        let result = gateway.process(query);
        assert!(result.is_err(), "Impossible SLA should fail");

        match result.unwrap_err() {
            GatewayError::RoutingError(_) => {},
            _ => panic!("Expected RoutingError"),
        }
    }

    // ============================================================================
    // Proof Certificate Tests
    // ============================================================================

    #[test]
    fn test_proof_generation() {
        let policies = vec![
            Policy::allow_user("user123"),
            Policy::mask_pii(),
        ];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "Test question".to_string(),
            sources: vec!["source1.txt".to_string()],
            user_id: "user123".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        let response = gateway.process(query).unwrap();

        assert!(response.proof.verify(), "Proof should be valid");
        assert!(!response.proof.claims.is_empty(), "Proof should have claims");
        assert!(!response.proof.proof_hash.is_empty(), "Proof should have hash");
    }

    #[test]
    fn test_proof_audit_bundle() {
        let policies = vec![Policy::allow_user("user123")];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "Test question".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        let response = gateway.process(query).unwrap();
        let bundle = response.proof.export_audit_bundle();

        assert!(bundle.contains("Proof Certificate"));
        assert!(bundle.contains("access_granted"));
    }

    // ============================================================================
    // Audit Logging Tests
    // ============================================================================

    #[test]
    fn test_successful_request_logged() {
        let policies = vec![Policy::allow_user("user123")];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "Test question".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        let _ = gateway.process(query);

        let audit_log = gateway.audit_log();
        let events = audit_log.get_events();

        assert!(!events.is_empty(), "Audit log should have events");
        assert!(events.iter().any(|e| e.is_success()), "Should have success event");
    }

    #[test]
    fn test_blocked_request_logged() {
        let policies = vec![Policy::deny_user("blocked")];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "Test question".to_string(),
            sources: vec![],
            user_id: "blocked".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        let _ = gateway.process(query);

        let audit_log = gateway.audit_log();
        let events = audit_log.get_events();

        assert!(events.iter().any(|e| e.is_blocked()), "Should have blocked event");
    }

    // ============================================================================
    // Performance Tests
    // ============================================================================

    #[test]
    fn test_latency_target() {
        let policies = vec![Policy::allow_user("user123")];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "Performance test".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: Some(150),
            cost_budget: None,
        };

        let response = gateway.process(query).unwrap();
        assert!(
            response.metrics.latency_ms < 150,
            "Should meet p99 latency target of <150ms"
        );
    }

    #[test]
    fn test_batch_processing() {
        let policies = vec![Policy::allow_user("user123")];
        let mut gateway = RagGateway::new(policies);

        let queries: Vec<_> = (0..10)
            .map(|i| RagQuery {
                question: format!("Question {}", i),
                sources: vec![],
                user_id: "user123".to_string(),
                latency_sla: None,
                cost_budget: None,
            })
            .collect();

        let start = std::time::Instant::now();
        for query in queries {
            let _ = gateway.process(query);
        }
        let elapsed = start.elapsed();

        assert!(elapsed.as_millis() < 1500, "Batch processing should be efficient");
    }

    // ============================================================================
    // Edge Cases and Error Handling
    // ============================================================================

    #[test]
    fn test_empty_question() {
        let policies = vec![Policy::allow_user("user123")];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        let result = gateway.process(query);
        assert!(result.is_ok(), "Empty question should be handled gracefully");
    }

    #[test]
    fn test_empty_sources() {
        let policies = vec![Policy::allow_user("user123")];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "Question with no sources".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        let result = gateway.process(query);
        assert!(result.is_ok(), "Empty sources should be handled");
    }

    #[test]
    fn test_very_long_question() {
        let policies = vec![Policy::allow_user("user123")];
        let mut gateway = RagGateway::new(policies);

        let long_question = "a".repeat(10000);
        let query = RagQuery {
            question: long_question,
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        let result = gateway.process(query);
        assert!(result.is_ok(), "Long questions should be handled");
    }

    #[test]
    fn test_special_characters_in_question() {
        let policies = vec![Policy::allow_user("user123")];
        let mut gateway = RagGateway::new(policies);

        let query = RagQuery {
            question: "Test with special chars: <>&\"'`.;[]{}()".to_string(),
            sources: vec![],
            user_id: "user123".to_string(),
            latency_sla: None,
            cost_budget: None,
        };

        let result = gateway.process(query);
        assert!(result.is_ok(), "Special characters should be handled");
    }

    // ============================================================================
    // Concurrent Access Tests
    // ============================================================================

    #[test]
    fn test_concurrent_queries() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let policies = vec![Policy::allow_user("user123")];
        let gateway = Arc::new(Mutex::new(RagGateway::new(policies)));

        let mut handles = vec![];
        for i in 0..5 {
            let gateway_clone = Arc::clone(&gateway);
            let handle = thread::spawn(move || {
                let query = RagQuery {
                    question: format!("Concurrent question {}", i),
                    sources: vec![],
                    user_id: "user123".to_string(),
                    latency_sla: None,
                    cost_budget: None,
                };

                let mut gw = gateway_clone.lock().unwrap();
                gw.process(query)
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result.is_ok(), "Concurrent access should work");
        }
    }
}
