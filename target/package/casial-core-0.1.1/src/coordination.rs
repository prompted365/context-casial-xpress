//! # Coordination Module
//!
//! Handles the coordination of different AI perspectives and context injection strategies.
//! Implements the consciousness-computation substrate for managing multiple viewpoints.

use crate::{CasialError, PerceptionId};
use ahash::AHashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Coordination metrics for performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationMetrics {
    pub perception_lock_latency_ms: f64,
    pub paradox_resolution_time_ms: f64,
    pub context_composition_time_ms: f64,
    pub total_coordination_time_ms: f64,
    pub memory_usage_bytes: usize,
}

/// Coordination strategy for different scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationStrategy {
    /// Fast coordination with minimal paradox handling
    Rapid,
    /// Balanced approach with moderate paradox resolution
    Balanced,
    /// Comprehensive coordination with full paradox synthesis
    Comprehensive,
    /// Custom strategy with specific parameters
    Custom {
        paradox_timeout_ms: u64,
        perception_lock_attempts: u32,
        synthesis_depth: u8,
    },
}

impl Default for CoordinationStrategy {
    fn default() -> Self {
        Self::Balanced
    }
}

/// Configuration for coordination behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationConfig {
    pub strategy: CoordinationStrategy,
    pub enable_perception_locking: bool,
    pub enable_paradox_detection: bool,
    pub enable_synthesis: bool,
    pub max_coordination_time_ms: u64,
    pub memory_limit_bytes: Option<usize>,
}

impl Default for CoordinationConfig {
    fn default() -> Self {
        Self {
            strategy: CoordinationStrategy::default(),
            enable_perception_locking: true,
            enable_paradox_detection: true,
            enable_synthesis: true,
            max_coordination_time_ms: 5000, // 5 second timeout
            memory_limit_bytes: Some(100 * 1024 * 1024), // 100MB limit
        }
    }
}

/// State of a coordination session
#[derive(Debug, Clone)]
pub struct CoordinationSession {
    pub id: uuid::Uuid,
    pub active_perceptions: Vec<PerceptionId>,
    pub locked_perceptions: Vec<PerceptionId>,
    pub detected_paradoxes: Vec<uuid::Uuid>,
    pub start_time: std::time::Instant,
    pub config: CoordinationConfig,
    pub metrics: CoordinationMetrics,
}

impl CoordinationSession {
    /// Create a new coordination session
    pub fn new(config: CoordinationConfig) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            active_perceptions: Vec::new(),
            locked_perceptions: Vec::new(),
            detected_paradoxes: Vec::new(),
            start_time: std::time::Instant::now(),
            config,
            metrics: CoordinationMetrics {
                perception_lock_latency_ms: 0.0,
                paradox_resolution_time_ms: 0.0,
                context_composition_time_ms: 0.0,
                total_coordination_time_ms: 0.0,
                memory_usage_bytes: 0,
            },
        }
    }

    /// Add a perception to this coordination session
    pub fn add_perception(&mut self, perception_id: PerceptionId) -> Result<()> {
        if !self.active_perceptions.contains(&perception_id) {
            self.active_perceptions.push(perception_id);
        }
        Ok(())
    }

    /// Attempt to lock a perception for exclusive coordination
    pub fn lock_perception(&mut self, perception_id: PerceptionId) -> Result<bool> {
        if !self.config.enable_perception_locking {
            return Ok(false);
        }

        let lock_start = std::time::Instant::now();

        // Simulate perception lock attempt
        // In a real implementation, this would coordinate with other sessions
        let lock_acquired = !self.locked_perceptions.contains(&perception_id);

        if lock_acquired {
            self.locked_perceptions.push(perception_id);
        }

        self.metrics.perception_lock_latency_ms = lock_start.elapsed().as_secs_f64() * 1000.0;

        Ok(lock_acquired)
    }

    /// Release a perception lock
    pub fn unlock_perception(&mut self, perception_id: PerceptionId) -> Result<()> {
        self.locked_perceptions.retain(|&id| id != perception_id);
        Ok(())
    }

    /// Check if session has timed out
    pub fn is_timed_out(&self) -> bool {
        self.start_time.elapsed().as_millis() > self.config.max_coordination_time_ms as u128
    }

    /// Finalize the session and calculate final metrics
    pub fn finalize(&mut self) {
        self.metrics.total_coordination_time_ms = self.start_time.elapsed().as_secs_f64() * 1000.0;

        // Estimate memory usage (simplified)
        self.metrics.memory_usage_bytes = self.active_perceptions.len()
            * std::mem::size_of::<PerceptionId>()
            + self.locked_perceptions.len() * std::mem::size_of::<PerceptionId>()
            + self.detected_paradoxes.len() * std::mem::size_of::<uuid::Uuid>();
    }
}

/// Coordination pool for managing multiple concurrent sessions
pub struct CoordinationPool {
    active_sessions: AHashMap<uuid::Uuid, CoordinationSession>,
    global_perception_locks: AHashMap<PerceptionId, uuid::Uuid>,
    max_concurrent_sessions: usize,
}

impl CoordinationPool {
    /// Create a new coordination pool
    pub fn new(max_concurrent_sessions: usize) -> Self {
        Self {
            active_sessions: AHashMap::new(),
            global_perception_locks: AHashMap::new(),
            max_concurrent_sessions,
        }
    }

    /// Start a new coordination session
    pub fn start_session(&mut self, config: CoordinationConfig) -> Result<uuid::Uuid> {
        if self.active_sessions.len() >= self.max_concurrent_sessions {
            return Err(CasialError::CoordinationFailure(
                "Maximum concurrent sessions reached".to_string(),
            )
            .into());
        }

        let session = CoordinationSession::new(config);
        let session_id = session.id;

        self.active_sessions.insert(session_id, session);

        Ok(session_id)
    }

    /// End a coordination session
    pub fn end_session(&mut self, session_id: uuid::Uuid) -> Result<CoordinationMetrics> {
        if let Some(mut session) = self.active_sessions.remove(&session_id) {
            // Release all perception locks held by this session
            for perception_id in &session.locked_perceptions {
                self.global_perception_locks.remove(perception_id);
            }

            session.finalize();
            Ok(session.metrics)
        } else {
            Err(
                CasialError::CoordinationFailure(format!("Session {} not found", session_id))
                    .into(),
            )
        }
    }

    /// Get session statistics
    pub fn get_statistics(&self) -> CoordinationPoolStats {
        let active_session_count = self.active_sessions.len();
        let total_locked_perceptions = self.global_perception_locks.len();

        let avg_session_time = if active_session_count > 0 {
            self.active_sessions
                .values()
                .map(|s| s.start_time.elapsed().as_secs_f64() * 1000.0)
                .sum::<f64>()
                / active_session_count as f64
        } else {
            0.0
        };

        CoordinationPoolStats {
            active_sessions: active_session_count,
            max_sessions: self.max_concurrent_sessions,
            locked_perceptions: total_locked_perceptions,
            average_session_duration_ms: avg_session_time,
        }
    }

    /// Cleanup timed-out sessions
    pub fn cleanup_timed_out_sessions(&mut self) -> usize {
        let timed_out_sessions: Vec<uuid::Uuid> = self
            .active_sessions
            .iter()
            .filter(|(_, session)| session.is_timed_out())
            .map(|(id, _)| *id)
            .collect();

        let count = timed_out_sessions.len();

        for session_id in timed_out_sessions {
            let _ = self.end_session(session_id);
        }

        count
    }
}

/// Statistics for coordination pool monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationPoolStats {
    pub active_sessions: usize,
    pub max_sessions: usize,
    pub locked_perceptions: usize,
    pub average_session_duration_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordination_session_creation() {
        let config = CoordinationConfig::default();
        let session = CoordinationSession::new(config);
        assert_eq!(session.active_perceptions.len(), 0);
        assert_eq!(session.locked_perceptions.len(), 0);
    }

    #[test]
    fn test_coordination_pool() {
        let mut pool = CoordinationPool::new(10);
        let config = CoordinationConfig::default();

        let session_id = pool.start_session(config).unwrap();
        assert_eq!(pool.active_sessions.len(), 1);

        let metrics = pool.end_session(session_id).unwrap();
        assert_eq!(pool.active_sessions.len(), 0);
        assert!(metrics.total_coordination_time_ms >= 0.0);
    }
}
