use std::collections::HashMap;
use uuid::Uuid;

use super::{
    error::{WorkflowResult, WorkflowError},
    state::{StateValue, Channel, WorkflowState, ExecutionTask},
    graph::StateNodeSpec,
    traits::NodeContext,
};

/// 工作流执行器
pub struct WorkflowExecutor {
    nodes: HashMap<String, StateNodeSpec>,
    state: WorkflowState,
    max_steps: usize,
}

impl WorkflowExecutor {
    pub fn new(
        nodes: HashMap<String, StateNodeSpec>,
        channels: HashMap<String, Channel>,
        max_steps: usize,
    ) -> WorkflowResult<Self> {
        Ok(Self {
            nodes,
            state: WorkflowState::new(channels),
            max_steps,
        })
    }
    
    /// 执行工作流 - 实现类似 Pregel 的三阶段算法
    pub async fn execute(&mut self, input: HashMap<String, StateValue>) -> WorkflowResult<HashMap<String, StateValue>> {
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
        
        for (node_id, _node_spec) in &self.nodes {
            // 跳过已完成的节点
            if self.state.completed_nodes.contains(node_id) {
                continue;
            }
            
            // 检查触发器是否满足
            if self.should_trigger_node(node_id)? {
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
    fn should_trigger_node(&self, _node_id: &str) -> WorkflowResult<bool> {
        // 简化版本：检查是否有输入数据
        if let Some(channel) = self.state.get_channel("input") {
            if !channel.is_empty() {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// 读取节点输入数据
    fn read_node_input(&self, _node_id: &str) -> WorkflowResult<StateValue> {
        // 简化版本：从 input 通道读取
        if let Some(channel) = self.state.get_channel("input") {
            if let Some(value) = channel.read() {
                return Ok(value.clone());
            }
        }
        Ok(StateValue::Null)
    }
    
    /// 阶段2: 并行执行任务
    async fn execute_tasks_parallel(&self, tasks: Vec<ExecutionTask>) -> WorkflowResult<Vec<(String, StateValue)>> {
        let mut handles = Vec::new();
        
        for task in tasks {
            let node_spec = self.nodes.get(&task.node_id)
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
            // 写入输出通道
            self.state.write_to_channel("output", output)?;
            
            // 标记节点为已完成
            self.state.completed_nodes.push(node_id);
        }
        
        Ok(())
    }
    
    /// 收集输出结果
    fn collect_outputs(&self) -> WorkflowResult<HashMap<String, StateValue>> {
        let mut outputs = HashMap::new();
        
        // 收集所有非空通道的值
        for (name, channel) in &self.state.channels {
            if let Some(value) = channel.read() {
                if !matches!(value, StateValue::Null) {
                    outputs.insert(name.clone(), value.clone());
                }
            }
        }
        
        Ok(outputs)
    }
}
