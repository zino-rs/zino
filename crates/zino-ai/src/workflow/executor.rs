use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::{
    error::{WorkflowResult, WorkflowError},
    state::{StateValue, WorkflowState, ExecutionTask},
    graph::StateGraph,
    traits::NodeContext,
};

/// 工作流执行器
pub struct WorkflowExecutor {
    graph: StateGraph,
    state: WorkflowState,
    max_steps: usize,
}

impl WorkflowExecutor {
    pub fn new(
        graph: StateGraph,
        max_steps: usize,
    ) -> WorkflowResult<Self> {
        Ok(Self {
            state: WorkflowState::new(graph.channels.clone()),
            graph,
            max_steps,
        })
    }
    
    /// 执行工作流 - 实现类似 Pregel 的三阶段算法
    pub async fn execute(&mut self, input: HashMap<String, StateValue>) -> WorkflowResult<HashMap<String, StateValue>> {
        // 重置工作流状态
        self.state = WorkflowState::new(self.graph.channels.clone());
        
        // 初始化输入
        for (channel_name, value) in input {
            self.state.write_to_channel(&channel_name, value)?;
        }
        
        // 主执行循环
        while self.state.step < self.max_steps {
            // 阶段1: Plan - 确定要执行的节点
            let tasks_to_execute = self.plan_next_tasks().await?;
            
            if tasks_to_execute.is_empty() {
                break; // 没有更多任务，执行完成
            }
            
            // 阶段2: Execution - 并行执行所有任务
            let results = self.execute_tasks_parallel(tasks_to_execute).await?;
            
            // 阶段3: Update - 更新通道值
            self.update_channels(results).await?;
            
            self.state.step += 1;
        }
        
        // 收集输出
        self.collect_outputs()
    }
    
    /// 阶段1: 计划下一个要执行的任务
    async fn plan_next_tasks(&mut self) -> WorkflowResult<Vec<ExecutionTask>> {
        let mut tasks = Vec::new();
        
        for (node_id, _node_spec) in &self.graph.nodes {
            // 跳过已完成的节点
            if self.state.completed_nodes.contains(node_id) {
                continue;
            }
            
            // 检查触发器是否满足
            if self.should_trigger_node(node_id).await? {
                // 读取输入数据
                let input = self.read_node_input(node_id)?;
                
                let task = ExecutionTask {
                    id: Uuid::new_v4(),
                    node_id: node_id.clone(),
                    input,
                    step: self.state.step,
                };
                
                tasks.push(task);
            }
        }
        
        Ok(tasks)
    }
    
    /// 检查节点是否应该被触发
    async fn should_trigger_node(&self, node_id: &str) -> WorkflowResult<bool> {
        // 检查节点是否已经完成
        if self.state.completed_nodes.contains(&node_id.to_string()) {
            return Ok(false);
        }
        
        // 获取所有前置节点
        let predecessors: Vec<&String> = self.graph.edges.iter()
            .filter_map(|(from, to)| if to == node_id { Some(from) } else { None })
            .collect();
        
        // 如果没有前置节点，说明是入口节点，总是触发
        if predecessors.is_empty() {
            return Ok(true);
        }
        
        // 检查分支逻辑：如果当前节点是分支节点的直接后继节点
        for predecessor in &predecessors {
            // 检查这个前置节点是否是分支节点
            if self.is_branch_node(predecessor) {
                // 检查分支节点的输出，确定应该执行哪个分支
                let branch_output_channel = format!("{}_output", predecessor);
                if let Some(channel) = self.state.get_channel(&branch_output_channel) {
                    if let Some(output_value) = channel.read() {
                        if let Some(target_branch) = output_value.as_string() {
                            // 如果当前节点是分支节点的直接后继节点
                            let is_direct_successor = self.graph.edges.iter()
                                .any(|(from, to)| from == *predecessor && to == node_id);
                            
                            if is_direct_successor {
                                // 只有被选择的目标分支节点才能执行
                                return Ok(node_id == target_branch);
                            }
                            // 如果当前节点不是分支节点的直接后继节点，继续正常检查
                        }
                    }
                }
                // 如果分支节点没有输出，不允许任何节点被触发
                return Ok(false);
            }
        }
        
        
        // 检查前置节点完成情况
        let completed_predecessors: Vec<&String> = predecessors.iter()
            .filter(|pred| self.state.completed_nodes.contains(**pred))
            .map(|pred| *pred)
            .collect();
        
        // 如果没有前置节点完成，不允许执行
        if completed_predecessors.is_empty() {
            return Ok(false);
        }
        
        // 检查是否存在分支依赖关系
        // 分支依赖：前置节点是分支节点的直接后继节点
        let has_branch_dependency = predecessors.iter()
            .any(|pred| {
                // 检查这个前置节点是否是某个分支节点的直接后继节点
                self.graph.edges.iter()
                    .any(|(from, to)| to == *pred && self.is_branch_node(from))
            });
        
        // 调试信息
        if node_id == "output" || node_id == "large" || node_id == "small" || node_id == "final" {
            println!("🔍 检查节点 {} 执行条件:", node_id);
            println!("   前置节点: {:?}", predecessors);
            println!("   已完成的前置节点: {:?}", completed_predecessors);
            println!("   是否有分支依赖: {}", has_branch_dependency);
            println!("   已完成节点数量: {}, 总前置节点数量: {}", completed_predecessors.len(), predecessors.len());
        }
        
        // 如果有分支依赖，只需要一个前置节点完成即可
        // 如果没有分支依赖，需要所有前置节点完成
        if !has_branch_dependency && completed_predecessors.len() != predecessors.len() {
            if node_id == "output" || node_id == "large" || node_id == "small" || node_id == "final" {
                println!("   ❌ 阻止节点 {} 执行：需要所有前置节点完成", node_id);
            }
            return Ok(false);
        }
        
        
        Ok(true)
    }
    
    /// 检查节点是否是分支节点
    fn is_branch_node(&self, node_id: &str) -> bool {
        // 检查这个节点是否有多个直接后继节点
        let successors: Vec<&String> = self.graph.edges.iter()
            .filter_map(|(from, to)| if from == node_id { Some(to) } else { None })
            .collect();
        
        // 如果有多个后继节点，则认为是分支节点
        successors.len() > 1
    }
    
    
    /// 找到控制指定分支后继节点的分支节点
    fn find_controlling_branch_for_successors(&self, successors: &[&String]) -> Option<String> {
        // 找到这些后继节点的共同父节点
        let mut parent_counts = HashMap::new();
        
        for successor in successors {
            let parents: Vec<&String> = self.graph.edges.iter()
                .filter_map(|(from, to)| if to == *successor { Some(from) } else { None })
                .collect();
            
            for parent in parents {
                *parent_counts.entry(parent).or_insert(0) += 1;
            }
        }
        
        // 找到所有后继节点都依赖的父节点
        let controlling_parents: Vec<String> = parent_counts.iter()
            .filter(|&(_, &count)| count == successors.len())
            .map(|(parent, _)| (*parent).clone())
            .collect();
        
        // 选择第一个控制父节点（通常只有一个）
        controlling_parents.first().cloned()
    }
    
    /// 检查节点是否依赖于分支节点
    fn is_branch_dependent_node(&self, node_id: &str) -> bool {
        // 检查这个节点是否有多个前置节点，且这些前置节点都是分支节点的直接后继节点
        let predecessors: Vec<&String> = self.graph.edges.iter()
            .filter_map(|(from, to)| if to == node_id { Some(from) } else { None })
            .collect();
        
        if predecessors.len() <= 1 {
            return false;
        }
        
        // 检查是否所有前置节点都是分支节点的直接后继节点
        let mut branch_successor_count = 0;
        for predecessor in &predecessors {
            // 检查这个前置节点是否是某个分支节点的直接后继节点
            let is_branch_successor = self.graph.edges.iter()
                .any(|(from, to)| to == *predecessor && self.is_branch_node(from));
            
            if is_branch_successor {
                branch_successor_count += 1;
            }
        }
        
        // 如果所有前置节点都是分支节点的直接后继节点，则认为是分支依赖
        branch_successor_count == predecessors.len()
    }
    
    /// 找到控制指定节点的分支节点
    fn find_controlling_branch_node(&self, node_ids: &[&String]) -> Option<String> {
        // 找到这些节点的共同祖先节点
        let mut common_ancestors = HashMap::new();
        
        for node_id in node_ids {
            let ancestors = self.get_all_ancestors(node_id);
            for ancestor in ancestors {
                *common_ancestors.entry(ancestor).or_insert(0) += 1;
            }
        }
        
        // 找到所有节点都依赖的祖先节点
        let controlling_ancestors: Vec<String> = common_ancestors.iter()
            .filter(|&(_, &count)| count == node_ids.len())
            .map(|(ancestor, _)| ancestor.clone())
            .collect();
        
        // 选择最近的祖先节点（距离最短的）
        if let Some(closest_ancestor) = controlling_ancestors.iter()
            .min_by_key(|ancestor| self.get_distance_to_nodes(ancestor, node_ids)) {
            Some(closest_ancestor.clone())
        } else {
            None
        }
    }
    
    /// 获取节点的所有祖先节点
    fn get_all_ancestors(&self, node_id: &str) -> Vec<String> {
        let mut ancestors = Vec::new();
        let mut to_visit = vec![node_id.to_string()];
        let mut visited = HashSet::new();
        
        while let Some(current) = to_visit.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());
            
            let direct_ancestors: Vec<String> = self.graph.edges.iter()
                .filter_map(|(from, to)| if to == &current { Some(from.clone()) } else { None })
                .collect();
            
            for ancestor in direct_ancestors {
                if !visited.contains(&ancestor) {
                    ancestors.push(ancestor.clone());
                    to_visit.push(ancestor);
                }
            }
        }
        
        ancestors
    }
    
    /// 计算从指定节点到目标节点集合的距离
    fn get_distance_to_nodes(&self, from_node: &str, to_nodes: &[&String]) -> usize {
        let mut min_distance = usize::MAX;
        
        for to_node in to_nodes {
            let distance = self.get_shortest_path_length(from_node, to_node);
            min_distance = min_distance.min(distance);
        }
        
        min_distance
    }
    
    /// 计算两个节点之间的最短路径长度
    fn get_shortest_path_length(&self, from: &str, to: &str) -> usize {
        if from == to {
            return 0;
        }
        
        let mut queue = std::collections::VecDeque::new();
        let mut visited = HashSet::new();
        
        queue.push_back((from.to_string(), 0));
        visited.insert(from.to_string());
        
        while let Some((current, distance)) = queue.pop_front() {
            let neighbors: Vec<String> = self.graph.edges.iter()
                .filter_map(|(from_edge, to_edge)| {
                    if from_edge == &current { Some(to_edge.clone()) } else { None }
                })
                .collect();
            
            for neighbor in neighbors {
                if neighbor == to {
                    return distance + 1;
                }
                
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor.clone());
                    queue.push_back((neighbor, distance + 1));
                }
            }
        }
        
        usize::MAX // 没有路径
    }
    
    /// 读取节点输入数据
    fn read_node_input(&self, node_id: &str) -> WorkflowResult<StateValue> {
        // 获取所有前置节点
        let predecessors: Vec<&String> = self.graph.edges.iter()
            .filter_map(|(from, to)| if to == node_id { Some(from) } else { None })
            .collect();
        
        if !predecessors.is_empty() {
            for predecessor in &predecessors {
                if self.is_branch_node(predecessor) {
                    let branch_input_predecessors: Vec<&String> = self.graph.edges.iter()
                        .filter_map(|(from, to)| if to == *predecessor { Some(from) } else { None })
                        .collect();
                    
                    
                    // 优先读取已完成的前置节点的数据
                    for branch_input_predecessor in &branch_input_predecessors {
                        if self.state.completed_nodes.contains(*branch_input_predecessor) {
                            let output_channel = format!("{}_output", branch_input_predecessor);
                            if let Some(channel) = self.state.get_channel(&output_channel) {
                                if let Some(value) = channel.read() {
                                    // 如果当前节点是结果节点（通常以 success/error/result 结尾），
                                    // 且分支输入是布尔类型（通常是验证结果），则尝试查找更早的数据源
                                    if (node_id.contains("success") || node_id.contains("error") || node_id.contains("result")) 
                                        && matches!(value, StateValue::Boolean(_)) {
                                        // 尝试查找更早的数据源
                                        let earlier_predecessors: Vec<&String> = self.graph.edges.iter()
                                            .filter_map(|(from, to)| if to == *branch_input_predecessor { Some(from) } else { None })
                                            .collect();
                                        
                                        for earlier_predecessor in &earlier_predecessors {
                                            if self.state.completed_nodes.contains(*earlier_predecessor) {
                                                let earlier_output_channel = format!("{}_output", earlier_predecessor);
                                                if let Some(earlier_channel) = self.state.get_channel(&earlier_output_channel) {
                                                    if let Some(earlier_value) = earlier_channel.read() {
                                                        if !matches!(earlier_value, StateValue::Boolean(_)) {
                                                            return Ok(earlier_value.clone());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    return Ok(value.clone());
                                }
                            }
                        }
                    }
                    
                    // 如果没有已完成的前置节点，尝试读取第一个有数据的前置节点
                    for branch_input_predecessor in &branch_input_predecessors {
                        let output_channel = format!("{}_output", branch_input_predecessor);
                        if let Some(channel) = self.state.get_channel(&output_channel) {
                            if let Some(value) = channel.read() {
                                if !matches!(value, StateValue::Null) {
                                    return Ok(value.clone());
                                }
                            }
                        }
                    }
                }
            }
            
            // 优先读取已完成的前置节点的输出
            for predecessor in &predecessors {
                if self.state.completed_nodes.contains(*predecessor) {
                    let output_channel = format!("{}_output", predecessor);
                    if let Some(channel) = self.state.get_channel(&output_channel) {
                        if let Some(value) = channel.read() {
                            return Ok(value.clone());
                        }
                    }
                }
            }
            
            // 如果没有已完成的前置节点，尝试读取第一个前置节点的输出
            if let Some(first_predecessor) = predecessors.first() {
                let output_channel = format!("{}_output", first_predecessor);
                if let Some(channel) = self.state.get_channel(&output_channel) {
                    if let Some(value) = channel.read() {
                        return Ok(value.clone());
                    }
                }
            }
        }
        
        // 如果没有前置节点，尝试从所有可用的输入通道读取
        for (channel_name, channel) in &self.state.channels {
            // 跳过输出通道
            if channel_name.ends_with("_output") {
                continue;
            }
            
            if let Some(value) = channel.read() {
                if !matches!(value, StateValue::Null) {
                    return Ok(value.clone());
                }
            }
        }
        
        Ok(StateValue::Null)
    }
    
    /// 阶段2: 并行执行任务
    async fn execute_tasks_parallel(&self, tasks: Vec<ExecutionTask>) -> WorkflowResult<Vec<(String, StateValue)>> {
        let mut handles = Vec::new();
        
        for task in tasks {
            let node_spec = self.graph.nodes.get(&task.node_id)
                .ok_or_else(|| WorkflowError::InvalidState(format!("Node {} not found", task.node_id)))?;
            
            let executor = node_spec.runnable.clone();
            let input = task.input.clone();
            let node_id = task.node_id.clone();
            
            let handle = tokio::spawn(async move {
                let context = NodeContext::new();
                let result = executor.execute(input, &context).await;
                (node_id, result)
            });
            
            handles.push(handle);
        }
        
        let mut results = Vec::new();
        for handle in handles {
            let (node_id, result) = handle.await
                .map_err(|e| WorkflowError::NodeExecutionFailed(format!("Task join error: {}", e)))?;
            
            match result {
                Ok(output) => results.push((node_id, output)),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }
    
    /// 阶段3: 更新通道值
    async fn update_channels(&mut self, results: Vec<(String, StateValue)>) -> WorkflowResult<()> {
        for (node_id, output) in results {
            // 写入节点的输出通道
            let output_channel = format!("{}_output", node_id);
            self.state.write_to_channel(&output_channel, output)?;
            
            // 标记节点为已完成
            self.state.completed_nodes.push(node_id);
        }
        
        Ok(())
    }
    
    /// 收集输出结果
    fn collect_outputs(&self) -> WorkflowResult<HashMap<String, StateValue>> {
        let mut outputs = HashMap::new();
        
        // 优先从结束节点的输出通道收集最终结果
        let end_output_channel = format!("{}_output", crate::workflow::graph::END_NODE);
        if let Some(channel) = self.state.get_channel(&end_output_channel) {
            if let Some(value) = channel.read() {
                if !matches!(value, StateValue::Null) {
                    outputs.insert("final_result".to_string(), value.clone());
                }
            }
        }
        
        // 收集所有其他非空通道的值
        for (name, channel) in &self.state.channels {
            if name != &end_output_channel {
                if let Some(value) = channel.read() {
                    if !matches!(value, StateValue::Null) {
                        outputs.insert(name.clone(), value.clone());
                    }
                }
            }
        }
        
        Ok(outputs)
    }
}
