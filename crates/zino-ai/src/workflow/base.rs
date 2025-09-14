//! 工作流基础模块
//! 
//! 这个模块提供了工作流系统的核心功能，包括：
//! - 错误处理和结果类型
//! - 状态值和通道管理
//! - 配置和策略定义
//! - 各种 trait 定义
//! - 节点包装器实现
//! - 状态图和执行器
//! - 便利宏



// 重新导出公共类型，保持向后兼容性
pub use super::error::{WorkflowError, WorkflowResult};
pub use super::state::{StateValue, Channel, WorkflowState, ExecutionTask};
pub use super::config::{NodeConfig, RetryPolicy, CachePolicy, NodeParamTypes};
pub use super::traits::{
    ChannelWriter, NodeStore, Runtime, StateNode, BranchPath, BranchResult, NodeContext
};
pub use super::node_wrappers::{
    FunctionNodeWrapper, AsyncFunctionNodeWrapper, ConfigFunctionNodeWrapper,
    ConfigWriterAsyncFunctionNodeWrapper, RuntimeFunctionNodeWrapper, BranchFunctionWrapper
};
pub use super::graph::{StateNodeSpec, BranchSpec, StateGraph, CompiledStateGraph};
pub use super::executor::WorkflowExecutor;


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use crate::node;
    use crate::branch;
    
    #[tokio::test]
    async fn test_simple_workflow() {
        // 创建状态图
        let mut graph = StateGraph::new("MyState".to_string());
        
        // 创建节点
        let node1 = StateNodeSpec::new(
            node!("process", |state| {
                println!("处理数据: {:?}", state);
                Ok(StateValue::String("处理完成".to_string()))
            })
        );
        
        // 添加节点和边
        graph
            .add_node("process".to_string(), node1)
            .set_entry_point("process".to_string())
            .set_finish_point("process".to_string());
        
        // 创建通道
        let mut channels = HashMap::new();
        channels.insert("input".to_string(), Channel::new_last_value(StateValue::Null));
        channels.insert("output".to_string(), Channel::new_last_value(StateValue::Null));
        graph.channels = channels;
        
        // 编译并执行
        let mut compiled = graph.compile().unwrap();
        let mut input = HashMap::new();
        input.insert("input".to_string(), StateValue::String("测试数据".to_string()));
        
        let result = compiled.invoke(input).await.unwrap();
        println!("结果: {:?}", result);
        
        // 验证结果
        assert!(result.contains_key("output"));
    }

    #[tokio::test]
    async fn test_complex_workflow() {
        println!("=== 开始复杂工作流测试 ===");
        
        // 创建状态图
        let mut graph = StateGraph::new("ComplexWorkflow".to_string());
        
        // 1. 数据预处理节点
        let preprocess_node = StateNodeSpec::new(
            node!("preprocess", |state| {
                println!("🔧 预处理节点: 开始处理输入数据");
                let state: &StateValue = &state;
                if let Some(input) = state.as_object() {
                    if let Some(StateValue::String(data)) = input.get("raw_data") {
                        let processed = format!("预处理: {}", data.to_uppercase());
                        println!("   输入: {}", data);
                        println!("   输出: {}", processed);
                        Ok(StateValue::String(processed))
                    } else {
                        Ok(StateValue::String("无数据".to_string()))
                    }
                } else {
                    Ok(StateValue::String("无效状态".to_string()))
                }
            })
        ).with_retry_policy(RetryPolicy::FixedDelay { 
            delay_ms: 100, 
            max_retries: 3 
        });

        // 2. 数据验证节点
        let validate_node = StateNodeSpec::new(
            node!("validate", |state| {
                println!("✅ 验证节点: 检查数据有效性");
                let state: &StateValue = &state;
                if let Some(data) = state.as_string() {
                    if data.len() > 5 {
                        println!("   数据有效: {}", data);
                        Ok(StateValue::Boolean(true))
                    } else {
                        println!("   数据无效: 长度不足");
                        Ok(StateValue::Boolean(false))
                    }
                } else {
                    println!("   数据无效: 类型错误");
                    Ok(StateValue::Boolean(false))
                }
            })
        );

        // 3. 分支节点 - 根据验证结果决定路径
        let mut branch_node = BranchSpec::new(
            branch!("route", |state| {
                println!("🔀 分支节点: 根据验证结果路由");
                let state: &StateValue = &state;
                if let Some(is_valid) = state.as_boolean() {
                    if is_valid {
                        println!("   路由到: 成功路径");
                        Ok(BranchResult::Single("success".to_string()))
                    } else {
                        println!("   路由到: 错误处理路径");
                        Ok(BranchResult::Single("error".to_string()))
                    }
                } else {
                    println!("   路由到: 默认路径");
                    Ok(BranchResult::Single("error".to_string()))
                }
            })
        );
        
        // 设置分支结束点映射
        let mut ends = HashMap::new();
        ends.insert("success".to_string(), "success".to_string());
        ends.insert("error".to_string(), "error".to_string());
        branch_node = branch_node.with_ends(ends);

        // 4. 成功处理节点
        let success_node = StateNodeSpec::new(
            node!("success", |state| {
                println!("🎉 成功节点: 处理有效数据");
                let state: &StateValue = &state;
                if let Some(data) = state.as_string() {
                    let result = format!("成功处理: {}", data);
                    println!("   结果: {}", result);
                    Ok(StateValue::String(result))
                } else {
                    Ok(StateValue::String("处理失败".to_string()))
                }
            })
        );

        // 5. 错误处理节点
        let error_node = StateNodeSpec::new(
            node!("error", |state| {
                println!("❌ 错误节点: 处理无效数据");
                let error_msg = "数据验证失败，已记录错误";
                println!("   错误信息: {}", error_msg);
                Ok(StateValue::String(error_msg.to_string()))
            })
        );

        // 6. 最终汇总节点
        let summary_node = StateNodeSpec::new(
            node!("summary", |state| {
                println!("📊 汇总节点: 生成最终报告");
                let mut summary = HashMap::new();
                summary.insert("status".to_string(), StateValue::String("completed".to_string()));
                summary.insert("timestamp".to_string(), StateValue::Number(chrono::Utc::now().timestamp() as f64));
                
                let state: &StateValue = &state;
                if let Some(data) = state.as_string() {
                    summary.insert("result".to_string(), StateValue::String(data.clone()));
                    if data.contains("成功") {
                        summary.insert("success".to_string(), StateValue::Boolean(true));
                    } else {
                        summary.insert("success".to_string(), StateValue::Boolean(false));
                    }
                }
                
                println!("   汇总完成: {:?}", summary);
                Ok(StateValue::Object(summary))
            })
        );

        // 添加所有节点到图中
        graph
            .add_node("preprocess".to_string(), preprocess_node)
            .add_node("validate".to_string(), validate_node)
            .add_node("success".to_string(), success_node)
            .add_node("error".to_string(), error_node)
            .add_node("summary".to_string(), summary_node);

        // 设置边连接
        graph
            .add_edge("preprocess".to_string(), "validate".to_string())
            .add_conditional_edges("validate".to_string(), branch_node)
            .add_edge("success".to_string(), "summary".to_string())
            .add_edge("error".to_string(), "summary".to_string());

        // 设置入口和出口
        graph
            .set_entry_point("preprocess".to_string())
            .set_finish_point("summary".to_string());

        // 创建通道
        let mut channels = HashMap::new();
        channels.insert("raw_data".to_string(), Channel::new_last_value(StateValue::Null));
        channels.insert("processed_data".to_string(), Channel::new_last_value(StateValue::Null));
        channels.insert("validation_result".to_string(), Channel::new_last_value(StateValue::Null));
        channels.insert("final_result".to_string(), Channel::new_last_value(StateValue::Null));
        graph.channels = channels;

        // 编译工作流
        println!("🔨 编译工作流...");
        let mut compiled = graph.compile().unwrap();
        println!("✅ 编译成功");

        // 测试场景1: 有效数据
        println!("\n--- 测试场景1: 有效数据 ---");
        let mut input1 = HashMap::new();
        input1.insert("raw_data".to_string(), StateValue::String("Hello World".to_string()));
        
        let result1 = compiled.invoke(input1).await.unwrap();
        println!("场景1结果: {:?}", result1);
        
        // 验证场景1结果
        if let Some(StateValue::Object(summary)) = result1.get("final_result") {
            if let Some(StateValue::Boolean(success)) = summary.get("success") {
                assert!(*success, "场景1应该成功");
            }
        }

        // 测试场景2: 无效数据
        println!("\n--- 测试场景2: 无效数据 ---");
        let mut input2 = HashMap::new();
        input2.insert("raw_data".to_string(), StateValue::String("Hi".to_string()));
        
        let result2 = compiled.invoke(input2).await.unwrap();
        println!("场景2结果: {:?}", result2);
        
        // 验证场景2结果
        if let Some(StateValue::Object(summary)) = result2.get("final_result") {
            if let Some(StateValue::Boolean(success)) = summary.get("success") {
                assert!(!*success, "场景2应该失败");
            }
        }

        // 测试场景3: 空数据
        println!("\n--- 测试场景3: 空数据 ---");
        let mut input3 = HashMap::new();
        input3.insert("raw_data".to_string(), StateValue::String("".to_string()));
        
        let result3 = compiled.invoke(input3).await.unwrap();
        println!("场景3结果: {:?}", result3);

        println!("\n=== 复杂工作流测试完成 ===");
    }

    #[tokio::test]
    async fn test_async_workflow() {
        println!("=== 开始异步工作流测试 ===");
        
        let mut graph = StateGraph::new("AsyncWorkflow".to_string());
        
        // 异步处理节点
        let async_node = StateNodeSpec::new(
            node!("async_process", async |state| {
                println!("⏳ 异步节点: 开始异步处理");
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
                let state: &StateValue = &state;
                if let Some(data) = state.as_string() {
                    let result = format!("异步处理完成: {}", data);
                    println!("   异步结果: {}", result);
                    Ok(StateValue::String(result))
                } else {
                    Ok(StateValue::String("异步处理失败".to_string()))
                }
            })
        );

        graph
            .add_node("async_process".to_string(), async_node)
            .set_entry_point("async_process".to_string())
            .set_finish_point("async_process".to_string());

        let mut channels = HashMap::new();
        channels.insert("input".to_string(), Channel::new_last_value(StateValue::Null));
        channels.insert("output".to_string(), Channel::new_last_value(StateValue::Null));
        graph.channels = channels;

        let mut compiled = graph.compile().unwrap();
        let mut input = HashMap::new();
        input.insert("input".to_string(), StateValue::String("异步测试数据".to_string()));
        
        let result = compiled.invoke(input).await.unwrap();
        println!("异步工作流结果: {:?}", result);
        
        assert!(result.contains_key("output"));
        println!("=== 异步工作流测试完成 ===");
    }
}
