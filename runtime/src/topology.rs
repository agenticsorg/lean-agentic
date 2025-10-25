//! Network topologies for agent coordination
//!
//! Supports mesh, ring, star, and hierarchical topologies.

use crate::orchestration::AgentRef;
use std::collections::HashMap;

/// Topology type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopologyType {
    /// Mesh: All agents connected to all others
    Mesh,
    /// Ring: Agents connected in a circular chain
    Ring,
    /// Star: Central hub with spokes
    Star,
    /// Hierarchical: Tree structure
    Hierarchical,
}

/// Topology graph
#[derive(Debug)]
pub struct Topology<T: Send> {
    topology_type: TopologyType,
    agents: Vec<AgentRef<T>>,
    connections: HashMap<u64, Vec<u64>>,
}

impl<T: Send> Topology<T> {
    /// Create new topology
    pub fn new(topology_type: TopologyType) -> Self {
        Self {
            topology_type,
            agents: Vec::new(),
            connections: HashMap::new(),
        }
    }

    /// Add agent to topology
    pub fn add_agent(&mut self, agent: AgentRef<T>) {
        let agent_id = agent.id;
        self.agents.push(agent);

        // Update connections based on topology type
        match self.topology_type {
            TopologyType::Mesh => self.connect_mesh(agent_id),
            TopologyType::Ring => self.connect_ring(agent_id),
            TopologyType::Star => self.connect_star(agent_id),
            TopologyType::Hierarchical => self.connect_hierarchical(agent_id),
        }
    }

    /// Get neighbors for agent
    pub fn neighbors(&self, agent_id: u64) -> Vec<&AgentRef<T>> {
        self.connections
            .get(&agent_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.agents.iter().find(|a| a.id == *id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all agents
    pub fn agents(&self) -> &[AgentRef<T>] {
        &self.agents
    }

    fn connect_mesh(&mut self, agent_id: u64) {
        // Connect to all other agents
        let others: Vec<u64> = self
            .agents
            .iter()
            .map(|a| a.id)
            .filter(|id| *id != agent_id)
            .collect();

        self.connections.insert(agent_id, others.clone());

        // Update existing agents to include new agent
        for other_id in others {
            self.connections
                .entry(other_id)
                .or_insert_with(Vec::new)
                .push(agent_id);
        }
    }

    fn connect_ring(&mut self, agent_id: u64) {
        let n = self.agents.len();
        if n == 1 {
            self.connections.insert(agent_id, vec![]);
            return;
        }

        // Connect to previous and next in ring
        let idx = n - 1;
        let prev_idx = if idx == 0 { n - 1 } else { idx - 1 };
        let next_idx = (idx + 1) % n;

        let prev_id = self.agents[prev_idx].id;
        let next_id = if n > 1 {
            self.agents[next_idx].id
        } else {
            agent_id
        };

        if n > 1 {
            self.connections.insert(agent_id, vec![next_id]);

            // Update previous agent's next
            if let Some(prev_conn) = self.connections.get_mut(&prev_id) {
                prev_conn.clear();
                prev_conn.push(agent_id);
            }
        }
    }

    fn connect_star(&mut self, agent_id: u64) {
        let n = self.agents.len();
        if n == 1 {
            // First agent is the hub
            self.connections.insert(agent_id, vec![]);
            return;
        }

        let hub_id = self.agents[0].id;

        if agent_id == hub_id {
            // Hub connects to all
            let spokes: Vec<u64> = self.agents.iter().map(|a| a.id).skip(1).collect();
            self.connections.insert(hub_id, spokes);
        } else {
            // Spoke connects only to hub
            self.connections.insert(agent_id, vec![hub_id]);

            // Add spoke to hub's connections
            self.connections
                .entry(hub_id)
                .or_insert_with(Vec::new)
                .push(agent_id);
        }
    }

    fn connect_hierarchical(&mut self, agent_id: u64) {
        let n = self.agents.len();
        if n == 1 {
            self.connections.insert(agent_id, vec![]);
            return;
        }

        // Binary tree structure
        let idx = n - 1;
        let parent_idx = (idx - 1) / 2;

        if idx > 0 {
            let parent_id = self.agents[parent_idx].id;

            // Connect to parent
            self.connections.insert(agent_id, vec![parent_id]);

            // Add to parent's children
            self.connections
                .entry(parent_id)
                .or_insert_with(Vec::new)
                .push(agent_id);
        } else {
            self.connections.insert(agent_id, vec![]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mailbox::Mailbox;
    use crate::orchestration::spawn;

    #[tokio::test]
    async fn test_mesh_topology() {
        let mut topology = Topology::new(TopologyType::Mesh);

        let agent1 = spawn(|_: Mailbox<i32>| async {}).await;
        let agent2 = spawn(|_: Mailbox<i32>| async {}).await;
        let agent3 = spawn(|_: Mailbox<i32>| async {}).await;

        topology.add_agent(agent1.clone());
        topology.add_agent(agent2.clone());
        topology.add_agent(agent3.clone());

        // In mesh, each node connects to all others
        assert_eq!(topology.neighbors(agent1.id).len(), 2);
        assert_eq!(topology.neighbors(agent2.id).len(), 2);
        assert_eq!(topology.neighbors(agent3.id).len(), 2);
    }

    #[tokio::test]
    async fn test_star_topology() {
        let mut topology = Topology::new(TopologyType::Star);

        let hub = spawn(|_: Mailbox<i32>| async {}).await;
        let spoke1 = spawn(|_: Mailbox<i32>| async {}).await;
        let spoke2 = spawn(|_: Mailbox<i32>| async {}).await;

        topology.add_agent(hub.clone());
        topology.add_agent(spoke1.clone());
        topology.add_agent(spoke2.clone());

        // Hub connects to all spokes
        assert_eq!(topology.neighbors(hub.id).len(), 2);

        // Spokes connect only to hub
        assert_eq!(topology.neighbors(spoke1.id).len(), 1);
        assert_eq!(topology.neighbors(spoke2.id).len(), 1);
    }
}
