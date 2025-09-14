use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::{
    error::WorkflowResult,
    state::{StateValue, Channel},
    config::{RetryPolicy, CachePolicy},
    traits::{StateNode, BranchPath},
    node_wrappers::FunctionNodeWrapper,
};

/// 工作流常量节点名称
pub const START_NODE: &str = "__start__";
pub const END_NODE: &str = "__end__";

/// 节点规格
#[derive(Clone)]
pub struct StateNodeSpec {
    /// 节点执行器
    pub runnable: Arc<dyn StateNode>,
    /// 元数据
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// 重试策略
    pub retry_policy: Option<RetryPolicy>,
    /// 缓存策略
    pub cache_policy: Option<CachePolicy>,
    /// 是否延迟执行
    pub defer: bool,
}

impl std::fmt::Debug for StateNodeSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateNodeSpec")
            .field("runnable", &"<dyn StateNode>")
            .field("metadata", &self.metadata)
            .field("retry_policy", &self.retry_policy)
            .field("cache_policy", &self.cache_policy)
            .field("defer", &self.defer)
            .finish()
    }
}

impl StateNodeSpec {
    pub fn new(runnable: Arc<dyn StateNode>) -> Self {
        Self {
            runnable,
            metadata: None,
            retry_policy: None,
            cache_policy: None,
            defer: false,
        }
    }
    
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = Some(policy);
        self
    }
    
    pub fn with_cache_policy(mut self, policy: CachePolicy) -> Self {
        self.cache_policy = Some(policy);
        self
    }
    
    pub fn with_defer(mut self, defer: bool) -> Self {
        self.defer = defer;
        self
    }
}

/// 分支规格
#[derive(Clone)]
pub struct BranchSpec {
    /// 路径函数
    pub path: Arc<dyn BranchPath>,
    /// 结束点映射
    pub ends: Option<HashMap<String, String>>,
    /// 输入模式
    pub input_schema: Option<String>,
}

impl std::fmt::Debug for BranchSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BranchSpec")
            .field("path", &"<dyn BranchPath>")
            .field("ends", &self.ends)
            .field("input_schema", &self.input_schema)
            .finish()
    }
}

impl BranchSpec {
    pub fn new(path: Arc<dyn BranchPath>) -> Self {
        Self {
            path,
            ends: None,
            input_schema: None,
        }
    }
    
    pub fn with_ends(mut self, ends: HashMap<String, String>) -> Self {
        self.ends = Some(ends);
        self
    }
    
    pub fn with_input_schema(mut self, schema: String) -> Self {
        self.input_schema = Some(schema);
        self
    }
}

/// 状态图
#[derive(Debug)]
#[derive(Clone)]
pub struct StateGraph {
    /// 节点映射
    pub nodes: HashMap<String, StateNodeSpec>,
    /// 边集合
    pub edges: HashSet<(String, String)>,
    /// 分支映射
    pub branches: HashMap<String, HashMap<String, BranchSpec>>,
    /// 通道映射
    pub channels: HashMap<String, Channel>,
    /// 等待边
    pub waiting_edges: HashSet<(Vec<String>, String)>,
    /// 是否已编译
    pub compiled: bool,
    /// 状态模式
    pub state_schema: String,
    /// 上下文模式
    pub context_schema: Option<String>,
    /// 输入模式
    pub input_schema: String,
    /// 输出模式
    pub output_schema: String,
}

impl StateGraph {
    pub fn new(state_schema: String) -> Self {
        let mut graph = Self {
            nodes: HashMap::new(),
            edges: HashSet::new(),
            branches: HashMap::new(),
            channels: HashMap::new(),
            waiting_edges: HashSet::new(),
            compiled: false,
            state_schema,
            context_schema: None,
            input_schema: "State".to_string(),
            output_schema: "State".to_string(),
        };
        
        // 自动添加开始和结束节点
        graph.add_start_end_nodes();
        graph
    }
    
    /// 添加开始和结束节点
    fn add_start_end_nodes(&mut self) {
        // 添加开始节点 - 只是传递输入数据
        let start_node = StateNodeSpec::new(
            Arc::new(FunctionNodeWrapper::new(
                START_NODE.to_string(),
                |input| Ok(input) // 直接传递输入
            ))
        );
        
        // 添加结束节点 - 收集最终结果
        let end_node = StateNodeSpec::new(
            Arc::new(FunctionNodeWrapper::new(
                END_NODE.to_string(),
                |input| Ok(input) // 直接传递输入作为最终结果
            ))
        );
        
        self.nodes.insert(START_NODE.to_string(), start_node);
        self.nodes.insert(END_NODE.to_string(), end_node);
    }
    
    /// 添加节点
    pub fn add_node(
        &mut self,
        name: String,
        node_spec: StateNodeSpec,
    ) -> &mut Self {
        self.nodes.insert(name, node_spec);
        self
    }
    
    /// 添加边
    pub fn add_edge(
        &mut self,
        start: String,
        end: String,
    ) -> &mut Self {
        self.edges.insert((start, end));
        self
    }
    
    /// 添加条件边
    pub fn add_conditional_edges(
        &mut self,
        start: String,
        branch_spec: BranchSpec,
    ) -> &mut Self {
        let branch_map = self.branches.entry(start).or_insert_with(HashMap::new);
        branch_map.insert("default".to_string(), branch_spec);
        self
    }
    
    /// 设置入口点
    pub fn set_entry_point(&mut self, node: String) -> &mut Self {
        self.add_edge(START_NODE.to_string(), node);
        self
    }
    
    /// 设置结束点
    pub fn set_finish_point(&mut self, node: String) -> &mut Self {
        self.add_edge(node, END_NODE.to_string());
        self
    }
    
    /// 编译图
    /// 自动创建节点输出通道
    pub fn auto_create_output_channels(&mut self) {
        for node_id in self.nodes.keys() {
            let output_channel = format!("{}_output", node_id);
            if !self.channels.contains_key(&output_channel) {
                self.channels.insert(output_channel, Channel::new_last_value(StateValue::Null));
            }
        }
    }
    
    pub fn compile(mut self) -> WorkflowResult<CompiledStateGraph> {
        // 自动创建所有节点的输出通道
        self.auto_create_output_channels();
        self.compiled = true;
        CompiledStateGraph::new(self)
    }
}

/// 编译后的状态图
pub struct CompiledStateGraph {
    graph: StateGraph,
    executor: crate::workflow::executor::WorkflowExecutor,
}

impl CompiledStateGraph {
    pub fn new(graph: StateGraph) -> WorkflowResult<Self> {
        let executor = crate::workflow::executor::WorkflowExecutor::new(graph.clone(), 100)?;
        
        Ok(Self { graph, executor })
    }
    
    pub async fn invoke(&mut self, input: HashMap<String, StateValue>) -> WorkflowResult<HashMap<String, StateValue>> {
        self.executor.execute(input).await
    }
}
