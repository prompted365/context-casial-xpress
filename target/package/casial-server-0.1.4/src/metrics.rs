//! # Metrics Collection
//!
//! Performance and coordination metrics for the Casial server.

use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use tracing::info;

/// Metrics collector for server performance and coordination statistics
pub struct MetricsCollector {
    coordination_events: u64,
    active_sessions: u64,
    paradoxes_resolved: u64,
    perception_locks: u64,
    substrate_operations: u64,
    last_updated: DateTime<Utc>,
    history: VecDeque<MetricsSnapshot>,
}

/// A snapshot of metrics at a point in time
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub timestamp: DateTime<Utc>,
    pub coordination_events: u64,
    pub active_sessions: u64,
    pub paradoxes_resolved: u64,
    pub perception_locks: u64,
    pub substrate_operations: u64,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            coordination_events: 0,
            active_sessions: 0,
            paradoxes_resolved: 0,
            perception_locks: 0,
            substrate_operations: 0,
            last_updated: Utc::now(),
            history: VecDeque::with_capacity(1000), // Keep last 1000 snapshots
        }
    }

    pub fn record_coordination_events(&mut self, count: usize) {
        self.coordination_events = count as u64;
        self.last_updated = Utc::now();
    }

    pub fn record_active_sessions(&mut self, count: usize) {
        self.active_sessions = count as u64;
        self.last_updated = Utc::now();
    }

    pub fn increment_paradoxes_resolved(&mut self) {
        self.paradoxes_resolved += 1;
        self.last_updated = Utc::now();
    }

    pub fn increment_perception_locks(&mut self) {
        self.perception_locks += 1;
        self.last_updated = Utc::now();
    }

    pub fn increment_substrate_operations(&mut self) {
        self.substrate_operations += 1;
        self.last_updated = Utc::now();
    }

    pub fn take_snapshot(&mut self) {
        let snapshot = MetricsSnapshot {
            timestamp: Utc::now(),
            coordination_events: self.coordination_events,
            active_sessions: self.active_sessions,
            paradoxes_resolved: self.paradoxes_resolved,
            perception_locks: self.perception_locks,
            substrate_operations: self.substrate_operations,
        };

        self.history.push_back(snapshot);

        // Keep only the last 1000 snapshots
        if self.history.len() > 1000 {
            self.history.pop_front();
        }
    }

    pub fn export_prometheus(&self) -> String {
        format!(
            r#"# HELP casial_coordination_events_total Total number of coordination events
# TYPE casial_coordination_events_total counter
casial_coordination_events_total {}

# HELP casial_active_sessions Current number of active WebSocket sessions
# TYPE casial_active_sessions gauge
casial_active_sessions {}

# HELP casial_paradoxes_resolved_total Total number of paradoxes resolved
# TYPE casial_paradoxes_resolved_total counter
casial_paradoxes_resolved_total {}

# HELP casial_perception_locks_total Total number of perception locks acquired
# TYPE casial_perception_locks_total counter
casial_perception_locks_total {}

# HELP casial_substrate_operations_total Total number of substrate operations
# TYPE casial_substrate_operations_total counter
casial_substrate_operations_total {}

# HELP casial_last_updated_timestamp Last metrics update timestamp
# TYPE casial_last_updated_timestamp gauge
casial_last_updated_timestamp {}
"#,
            self.coordination_events,
            self.active_sessions,
            self.paradoxes_resolved,
            self.perception_locks,
            self.substrate_operations,
            self.last_updated.timestamp()
        )
    }

    pub fn log_summary(&self) {
        info!("📊 Casial Metrics Summary:");
        info!("    Coordination Events: {}", self.coordination_events);
        info!("    Active Sessions: {}", self.active_sessions);
        info!("    Paradoxes Resolved: {}", self.paradoxes_resolved);
        info!("    Perception Locks: {}", self.perception_locks);
        info!("    Substrate Operations: {}", self.substrate_operations);
        info!("    Last Updated: {}", self.last_updated);
    }

    pub fn get_history(&self) -> &VecDeque<MetricsSnapshot> {
        &self.history
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();

        collector.record_coordination_events(5);
        collector.record_active_sessions(3);
        collector.increment_paradoxes_resolved();

        assert_eq!(collector.coordination_events, 5);
        assert_eq!(collector.active_sessions, 3);
        assert_eq!(collector.paradoxes_resolved, 1);
    }

    #[test]
    fn test_metrics_snapshot() {
        let mut collector = MetricsCollector::new();
        collector.record_coordination_events(10);
        collector.take_snapshot();

        assert_eq!(collector.history.len(), 1);
        assert_eq!(collector.history[0].coordination_events, 10);
    }

    #[test]
    fn test_prometheus_export() {
        let collector = MetricsCollector::new();
        let prometheus_output = collector.export_prometheus();

        assert!(prometheus_output.contains("casial_coordination_events_total"));
        assert!(prometheus_output.contains("casial_active_sessions"));
        assert!(prometheus_output.contains("# TYPE"));
        assert!(prometheus_output.contains("# HELP"));
    }
}
