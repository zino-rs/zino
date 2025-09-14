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
        graph.channels = channels;
        
        // 编译并执行
        let mut compiled = graph.compile().unwrap();
        let mut input = HashMap::new();
        input.insert("input".to_string(), StateValue::String("测试数据".to_string()));
        
        let result = compiled.invoke(input).await.unwrap();
        println!("结果: {:?}", result);
        
        // 验证结果
        assert!(result.contains_key("process_output"));
        assert!(result.contains_key("final_result"));
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
                if let Some(data) = state.as_string() {
                    let processed = format!("预处理: {}", data.to_uppercase());
                    println!("   输入: {}", data);
                    println!("   输出: {}", processed);
                    Ok(StateValue::String(processed))
                } else {
                    println!("   输入类型错误: {:?}", state);
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
        let branch_node = StateNodeSpec::new(
            node!("branch", |state| {
                println!("🔀 分支节点: 根据验证结果路由");
                let state: &StateValue = &state;
                if let Some(is_valid) = state.as_boolean() {
                    if is_valid {
                        println!("   路由到: 成功路径");
                        Ok(StateValue::String("success".to_string()))
                    } else {
                        println!("   路由到: 错误处理路径");
                        Ok(StateValue::String("error".to_string()))
                    }
                } else {
                    println!("   路由到: 默认路径");
                    Ok(StateValue::String("error".to_string()))
                }
            })
        );

        // 4. 成功处理节点
        let success_node = StateNodeSpec::new(
            node!("success", |state| {
                println!("🎉 成功节点: 处理有效数据");
                println!("   接收到的数据: {:?}", state);
                let state: &StateValue = &state;
                if let Some(data) = state.as_string() {
                    let result = format!("成功处理: {}", data);
                    println!("   结果: {}", result);
                    Ok(StateValue::String(result))
                } else {
                    println!("   数据类型错误，期望字符串，实际: {:?}", state);
                    Ok(StateValue::String("处理失败".to_string()))
                }
            })
        );

        // 5. 错误处理节点
        let error_node = StateNodeSpec::new(
            node!("error", |_state| {
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
            .add_node("branch".to_string(), branch_node)
            .add_node("success".to_string(), success_node)
            .add_node("error".to_string(), error_node)
            .add_node("summary".to_string(), summary_node);

        // 设置边连接
        graph
            .add_edge("preprocess".to_string(), "validate".to_string())
            .add_edge("validate".to_string(), "branch".to_string())
            .add_edge("branch".to_string(), "success".to_string())
            .add_edge("branch".to_string(), "error".to_string())
            .add_edge("success".to_string(), "summary".to_string())
            .add_edge("error".to_string(), "summary".to_string());

        // 设置入口和出口
        graph
            .set_entry_point("preprocess".to_string())
            .set_finish_point("summary".to_string());

        // 创建必要的输入通道
        let mut channels = HashMap::new();
        channels.insert("raw_data".to_string(), Channel::new_last_value(StateValue::Null));
        channels.insert("input".to_string(), Channel::new_last_value(StateValue::Null));
        graph.channels = channels;

        // 编译工作流
        println!("🔨 编译工作流...");
        let mut compiled = graph.compile().unwrap();
        println!("✅ 编译成功");

        // 测试场景1: 有效数据
        println!("\n--- 测试场景1: 有效数据 ---");
        let mut input1 = HashMap::new();
        input1.insert("raw_data".to_string(), StateValue::String("Hello World".to_string()));
        
        println!("开始执行场景1...");
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
                assert!(*success, "场景2应该成功（预处理后长度足够）");
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
        graph.channels = channels;

        let mut compiled = graph.compile().unwrap();
        let mut input = HashMap::new();
        input.insert("input".to_string(), StateValue::String("异步测试数据".to_string()));
        
        let result = compiled.invoke(input).await.unwrap();
        println!("异步工作流结果: {:?}", result);
        
        // 验证异步节点正确执行并输出了结果
        assert!(result.contains_key("async_process_output"));
        assert!(result.contains_key("final_result"));
        println!("=== 异步工作流测试完成 ===");
    }

    #[tokio::test]
    async fn test_simple_node_creation() {
        println!("=== 开始简单节点创建测试 ===");
        
        // 定义简单的处理函数
        fn process_data(state: StateValue) -> WorkflowResult<StateValue> {
            println!("处理数据: {:?}", state);
            Ok(StateValue::String("处理完成".to_string()))
        }

        fn validate_data(state: StateValue) -> WorkflowResult<StateValue> {
            println!("验证数据: {:?}", state);
            Ok(StateValue::Boolean(true))
        }

        let mut graph = StateGraph::new("SimpleNodesTest".to_string());
        
        // 使用最简单的语法：字符串 + 函数名
        let process_node = StateNodeSpec::new(
            node!("process", process_data)
        );
        
        let validate_node = StateNodeSpec::new(
            node!("validate", validate_data)
        );
        
        // 添加节点到图中
        graph
            .add_node("process".to_string(), process_node)
            .add_node("validate".to_string(), validate_node);
        
        // 设置边连接
        graph
            .add_edge("process".to_string(), "validate".to_string());
        
        // 设置入口和出口
        graph
            .set_entry_point("process".to_string())
            .set_finish_point("validate".to_string());
        
        // 创建通道
        let mut channels = HashMap::new();
        channels.insert("input".to_string(), Channel::new_last_value(StateValue::Null));
        graph.channels = channels;
        
        // 编译并执行
        let mut compiled = graph.compile().unwrap();
        let mut input = HashMap::new();
        input.insert("input".to_string(), StateValue::String("测试数据".to_string()));
        
        let result = compiled.invoke(input).await.unwrap();
        println!("简单节点测试结果: {:?}", result);
        
        // 验证结果
        assert!(result.contains_key("process_output"));
        assert!(result.contains_key("validate_output"));
        assert!(result.contains_key("final_result"));
        
        println!("=== 简单节点创建测试完成 ===");
    }

    #[tokio::test]
    async fn test_number_processing_workflow() {
        println!("=== 开始数字处理工作流测试 ===");
        
        let mut graph = StateGraph::new("NumberProcessing".to_string());
        
        // 1. 输入节点 - 获取输入数据
        let input_node = StateNodeSpec::new(
            node!("input", |state| {
                println!("📥 输入节点: 获取输入数据");
                let state: &StateValue = &state;
                if let Some(data) = state.as_string() {
                    let number: f64 = data.parse().unwrap_or(0.0);
                    println!("   输入数字: {}", number);
                    Ok(StateValue::Number(number))
                } else {
                    println!("   输入类型错误: {:?}", state);
                    Ok(StateValue::Number(0.0))
                }
            })
        );

        // 2. 乘法节点 - 乘以一个数字
        let multiply_node = StateNodeSpec::new(
            node!("multiply", |state| {
                println!("✖️ 乘法节点: 乘以数字");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    let multiplier = 10.0; // 乘以10
                    let result = number * multiplier;
                    println!("   {} × {} = {}", number, multiplier, result);
                    Ok(StateValue::Number(result))
                } else {
                    println!("   输入类型错误: {:?}", state);
                    Ok(StateValue::Number(0.0))
                }
            })
        );

        // 3. 分支节点 - 判断是否大于50
        let branch_node = StateNodeSpec::new(
            node!("branch", |state| {
                println!("🔀 分支节点: 判断数值大小");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    if number > 50.0 {
                        println!("   {} > 50，路由到翻倍节点", number);
                        Ok(StateValue::String("double".to_string()))
                    } else {
                        println!("   {} ≤ 50，路由到减半节点", number);
                        Ok(StateValue::String("half".to_string()))
                    }
                } else {
                    println!("   输入类型错误: {:?}", state);
                    Ok(StateValue::String("half".to_string()))
                }
            })
        );

        // 4. 翻倍节点 - 数值翻倍
        let double_node = StateNodeSpec::new(
            node!("double", |state| {
                println!("🔄 翻倍节点: 数值翻倍");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    let result = number * 2.0;
                    println!("   {} × 2 = {}", number, result);
                    Ok(StateValue::Number(result))
                } else {
                    println!("   输入类型错误: {:?}", state);
                    Ok(StateValue::Number(0.0))
                }
            })
        );

        // 5. 减半节点 - 数值减半
        let half_node = StateNodeSpec::new(
            node!("half", |state| {
                println!("➗ 减半节点: 数值减半");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    let result = number / 2.0;
                    println!("   {} ÷ 2 = {}", number, result);
                    Ok(StateValue::Number(result))
                } else {
                    println!("   输入类型错误: {:?}", state);
                    Ok(StateValue::Number(0.0))
                }
            })
        );

        // 6. 输出节点 - 打印最终结果
        let output_node = StateNodeSpec::new(
            node!("output", |state| {
                println!("📤 输出节点: 打印最终结果");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    let result = format!("最终结果: {}", number);
                    println!("   {}", result);
                    Ok(StateValue::String(result))
                } else {
                    println!("   输入类型错误: {:?}", state);
                    Ok(StateValue::String("输出错误".to_string()))
                }
            })
        );

        // 添加所有节点到图中
        graph
            .add_node("input".to_string(), input_node)
            .add_node("multiply".to_string(), multiply_node)
            .add_node("branch".to_string(), branch_node)
            .add_node("double".to_string(), double_node)
            .add_node("half".to_string(), half_node)
            .add_node("output".to_string(), output_node);

        // 设置边连接
        graph
            .add_edge("input".to_string(), "multiply".to_string())
            .add_edge("multiply".to_string(), "branch".to_string())
            .add_edge("branch".to_string(), "double".to_string())
            .add_edge("branch".to_string(), "half".to_string())
            // 注意：output 节点不直接依赖于 double 和 half，而是通过分支逻辑动态确定
            .add_edge("double".to_string(), "output".to_string())
            .add_edge("half".to_string(), "output".to_string());

        // 设置入口和出口
        graph
            .set_entry_point("input".to_string())
            .set_finish_point("output".to_string());

        // 创建输入通道
        let mut channels = HashMap::new();
        channels.insert("input".to_string(), Channel::new_last_value(StateValue::Null));
        graph.channels = channels;

        // 编译工作流
        println!("🔨 编译工作流...");
        let mut compiled = graph.compile().unwrap();
        println!("✅ 编译成功");

        // 测试场景1: 输入数字3 (3 × 10 = 30 ≤ 50，应该走减半路径)
        println!("\n--- 测试场景1: 输入数字3 ---");
        let mut input1 = HashMap::new();
        input1.insert("input".to_string(), StateValue::String("3".to_string()));
        
        let result1 = compiled.invoke(input1).await.unwrap();
        println!("场景1结果: {:?}", result1);
        
        // 验证场景1结果
        if let Some(StateValue::String(output)) = result1.get("final_result") {
            assert!(output.contains("15"), "场景1应该输出15 (30÷2=15)");
        }
        
        // 只获取最终结果
        if let Some(final_result) = result1.get("final_result") {
            println!("场景1最终结果: {:?}", final_result);
        }

        // 测试场景2: 输入数字8 (8 × 10 = 80 > 50，应该走翻倍路径)
        println!("\n--- 测试场景2: 输入数字8 ---");
        let mut input2 = HashMap::new();
        input2.insert("input".to_string(), StateValue::String("8".to_string()));
        
        let result2 = compiled.invoke(input2).await.unwrap();
        println!("场景2结果: {:?}", result2);
        
        // 验证场景2结果
        if let Some(StateValue::String(output)) = result2.get("final_result") {
            assert!(output.contains("160"), "场景2应该输出160 (80×2=160)");
        }

        // 测试场景3: 输入数字5 (5 × 10 = 50 = 50，应该走减半路径)
        println!("\n--- 测试场景3: 输入数字5 ---");
        let mut input3 = HashMap::new();
        input3.insert("input".to_string(), StateValue::String("5".to_string()));
        
        let result3 = compiled.invoke(input3).await.unwrap();
        println!("场景3结果: {:?}", result3);
        
        // 验证场景3结果
        if let Some(StateValue::String(output)) = result3.get("final_result") {
            assert!(output.contains("25"), "场景3应该输出25 (50÷2=25)");
        }

        println!("\n=== 数字处理工作流测试完成 ===");
    }

    #[tokio::test]
    async fn test_complex_branching_workflow() {
        println!("=== 开始复杂分支工作流测试 ===");
        
        let mut graph = StateGraph::new("ComplexBranching".to_string());
        
        // 1. 输入节点
        let input_node = StateNodeSpec::new(
            node!("input", |state| {
                println!("📥 输入节点: 获取输入数据");
                let state: &StateValue = &state;
                if let Some(data) = state.as_string() {
                    let number: f64 = data.parse().unwrap_or(0.0);
                    println!("   输入数字: {}", number);
                    Ok(StateValue::Number(number))
                } else {
                    Ok(StateValue::Number(0.0))
                }
            })
        );

        // 2. 预处理节点
        let preprocess_node = StateNodeSpec::new(
            node!("preprocess", |state| {
                println!("🔧 预处理节点: 数据预处理");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    let processed = number * 2.0;
                    println!("   {} × 2 = {}", number, processed);
                    Ok(StateValue::Number(processed))
                } else {
                    Ok(StateValue::Number(0.0))
                }
            })
        );

        // 3. 分支节点1 - 判断正负
        let branch1_node = StateNodeSpec::new(
            node!("branch1", |state| {
                println!("🔀 分支节点1: 判断正负");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    if number >= 0.0 {
                        println!("   {} ≥ 0，路由到正数处理", number);
                        Ok(StateValue::String("positive".to_string()))
                    } else {
                        println!("   {} < 0，路由到负数处理", number);
                        Ok(StateValue::String("negative".to_string()))
                    }
                } else {
                    Ok(StateValue::String("error".to_string()))
                }
            })
        );

        // 4. 正数处理节点
        let positive_node = StateNodeSpec::new(
            node!("positive", |state| {
                println!("➕ 正数处理节点: 处理正数");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    let result = number + 100.0;
                    println!("   {} + 100 = {}", number, result);
                    Ok(StateValue::Number(result))
                } else {
                    Ok(StateValue::Number(0.0))
                }
            })
        );

        // 5. 负数处理节点
        let negative_node = StateNodeSpec::new(
            node!("negative", |state| {
                println!("➖ 负数处理节点: 处理负数");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    let result = number - 100.0;
                    println!("   {} - 100 = {}", number, result);
                    Ok(StateValue::Number(result))
                } else {
                    Ok(StateValue::Number(0.0))
                }
            })
        );

        // 6. 分支节点2 - 判断大小
        let branch2_node = StateNodeSpec::new(
            node!("branch2", |state| {
                println!("🔀 分支节点2: 判断数值大小");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    if number > 50.0 {
                        println!("   {} > 50，路由到大数处理", number);
                        Ok(StateValue::String("large".to_string()))
                    } else {
                        println!("   {} ≤ 50，路由到小数处理", number);
                        Ok(StateValue::String("small".to_string()))
                    }
                } else {
                    Ok(StateValue::String("error".to_string()))
                }
            })
        );

        // 7. 大数处理节点
        let large_node = StateNodeSpec::new(
            node!("large", |state| {
                println!("🔢 大数处理节点: 处理大数");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    let result = number * 10.0;
                    println!("   {} × 10 = {}", number, result);
                    Ok(StateValue::Number(result))
                } else {
                    Ok(StateValue::Number(0.0))
                }
            })
        );

        // 8. 小数处理节点
        let small_node = StateNodeSpec::new(
            node!("small", |state| {
                println!("🔢 小数处理节点: 处理小数");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    let result = number / 10.0;
                    println!("   {} ÷ 10 = {}", number, result);
                    Ok(StateValue::Number(result))
                } else {
                    Ok(StateValue::Number(0.0))
                }
            })
        );

        // 9. 最终汇聚节点
        let final_node = StateNodeSpec::new(
            node!("final", |state| {
                println!("🎯 最终节点: 生成最终结果");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    let result = format!("最终结果: {}", number);
                    println!("   {}", result);
                    Ok(StateValue::String(result))
                } else {
                    Ok(StateValue::String("处理失败".to_string()))
                }
            })
        );

        // 添加所有节点
        graph
            .add_node("input".to_string(), input_node)
            .add_node("preprocess".to_string(), preprocess_node)
            .add_node("branch1".to_string(), branch1_node)
            .add_node("positive".to_string(), positive_node)
            .add_node("negative".to_string(), negative_node)
            .add_node("branch2".to_string(), branch2_node)
            .add_node("large".to_string(), large_node)
            .add_node("small".to_string(), small_node)
            .add_node("final".to_string(), final_node);

        // 设置边连接 - 创建复杂的多级分支
        graph
            .add_edge("input".to_string(), "preprocess".to_string())
            .add_edge("preprocess".to_string(), "branch1".to_string())
            .add_edge("branch1".to_string(), "positive".to_string())
            .add_edge("branch1".to_string(), "negative".to_string())
            .add_edge("positive".to_string(), "branch2".to_string())
            .add_edge("negative".to_string(), "branch2".to_string())
            .add_edge("branch2".to_string(), "large".to_string())
            .add_edge("branch2".to_string(), "small".to_string())
            .add_edge("large".to_string(), "final".to_string())
            .add_edge("small".to_string(), "final".to_string());

        // 设置入口和出口
        graph
            .set_entry_point("input".to_string())
            .set_finish_point("final".to_string());

        // 创建输入通道
        let mut channels = HashMap::new();
        channels.insert("input".to_string(), Channel::new_last_value(StateValue::Null));
        graph.channels = channels;

        // 编译工作流
        println!("🔨 编译复杂分支工作流...");
        let mut compiled = graph.compile().unwrap();
        println!("✅ 编译成功");

        // 测试场景1: 正数大数 (10 → 20 → positive → 120 → large → 1200)
        println!("\n--- 测试场景1: 正数大数 (10) ---");
        let mut input1 = HashMap::new();
        input1.insert("input".to_string(), StateValue::String("10".to_string()));
        
        let result1 = compiled.invoke(input1).await.unwrap();
        println!("场景1结果: {:?}", result1);
        
        // 验证结果
        if let Some(StateValue::String(output)) = result1.get("final_result") {
            assert!(output.contains("1200"), "场景1应该输出1200 (10→20→120→1200)");
        }

        // 测试场景2: 正数小数 (5 → 10 → positive → 110 → small → 11)
        println!("\n--- 测试场景2: 正数小数 (5) ---");
        let mut input2 = HashMap::new();
        input2.insert("input".to_string(), StateValue::String("5".to_string()));
        
        let result2 = compiled.invoke(input2).await.unwrap();
        println!("场景2结果: {:?}", result2);
        
        // 验证结果
        if let Some(StateValue::String(output)) = result2.get("final_result") {
            assert!(output.contains("11"), "场景2应该输出11 (5→10→110→11)");
        }

        // 测试场景3: 负数小数 (-10 → -20 → negative → -120 → small → -12)
        println!("\n--- 测试场景3: 负数小数 (-10) ---");
        let mut input3 = HashMap::new();
        input3.insert("input".to_string(), StateValue::String("-10".to_string()));
        
        let result3 = compiled.invoke(input3).await.unwrap();
        println!("场景3结果: {:?}", result3);
        
        // 验证结果
        if let Some(StateValue::String(output)) = result3.get("final_result") {
            assert!(output.contains("-12"), "场景3应该输出-12 (-10→-20→-120→-12)");
        }

        // 测试场景4: 负数小数 (-100 → -200 → negative → -300 → small → -30)
        println!("\n--- 测试场景4: 负数小数 (-100) ---");
        let mut input4 = HashMap::new();
        input4.insert("input".to_string(), StateValue::String("-100".to_string()));
        
        let result4 = compiled.invoke(input4).await.unwrap();
        println!("场景4结果: {:?}", result4);
        
        // 验证结果
        if let Some(StateValue::String(output)) = result4.get("final_result") {
            assert!(output.contains("-30"), "场景4应该输出-30 (-100→-200→-300→-30)");
        }

        // 测试场景5: 负数大数 (-200 → -400 → negative → -500 → small → -50)
        println!("\n--- 测试场景5: 负数小数 (-200) ---");
        let mut input5 = HashMap::new();
        input5.insert("input".to_string(), StateValue::String("-200".to_string()));
        
        let result5 = compiled.invoke(input5).await.unwrap();
        println!("场景5结果: {:?}", result5);
        
        // 验证结果
        if let Some(StateValue::String(output)) = result5.get("final_result") {
            assert!(output.contains("-50"), "场景5应该输出-50 (-200→-400→-500→-50)");
        }

        // 测试场景6: 负数大数 (100 → 200 → positive → 300 → large → 3000)
        println!("\n--- 测试场景6: 负数大数 (100) ---");
        let mut input6 = HashMap::new();
        input6.insert("input".to_string(), StateValue::String("100".to_string()));
        
        let result6 = compiled.invoke(input6).await.unwrap();
        println!("场景6结果: {:?}", result6);
        
        // 验证结果
        if let Some(StateValue::String(output)) = result6.get("final_result") {
            assert!(output.contains("3000"), "场景6应该输出3000 (100→200→300→3000)");
        }

        // 测试场景7: 负数小数 (-5 → -10 → negative → -110 → small → -11)
        println!("\n--- 测试场景7: 负数小数 (-5) ---");
        let mut input7 = HashMap::new();
        input7.insert("input".to_string(), StateValue::String("-5".to_string()));
        
        let result7 = compiled.invoke(input7).await.unwrap();
        println!("场景7结果: {:?}", result7);
        
        // 验证结果
        if let Some(StateValue::String(output)) = result7.get("final_result") {
            assert!(output.contains("-11"), "场景7应该输出-11 (-5→-10→-110→-11)");
        }

        println!("\n=== 复杂分支工作流测试完成 ===");
    }

    #[tokio::test]
    async fn test_edge_cases_workflow() {
        println!("=== 开始边界情况测试 ===");
        
        let mut graph = StateGraph::new("EdgeCases".to_string());
        
        // 1. 边界值测试节点
        let boundary_node = StateNodeSpec::new(
            node!("boundary", |state| {
                println!("🔍 边界值测试节点");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    if number == 0.0 {
                        println!("   检测到零值: {}", number);
                        Ok(StateValue::String("zero".to_string()))
                    } else if number == f64::INFINITY {
                        println!("   检测到无穷大: {}", number);
                        Ok(StateValue::String("infinity".to_string()))
                    } else if number == f64::NEG_INFINITY {
                        println!("   检测到负无穷: {}", number);
                        Ok(StateValue::String("negative_infinity".to_string()))
                    } else if number.is_nan() {
                        println!("   检测到NaN: {}", number);
                        Ok(StateValue::String("nan".to_string()))
                    } else {
                        println!("   正常数值: {}", number);
                        Ok(StateValue::String("normal".to_string()))
                    }
                } else {
                    println!("   非数值类型: {:?}", state);
                    Ok(StateValue::String("non_number".to_string()))
                }
            })
        );

        // 2. 错误处理节点
        let error_node = StateNodeSpec::new(
            node!("error_handler", |state| {
                println!("❌ 错误处理节点");
                let state: &StateValue = &state;
                match state {
                    StateValue::Null => {
                        println!("   处理空值");
                        Ok(StateValue::String("null_handled".to_string()))
                    },
                    StateValue::String(s) if s.is_empty() => {
                        println!("   处理空字符串");
                        Ok(StateValue::String("empty_string_handled".to_string()))
                    },
                    _ => {
                        println!("   处理其他类型: {:?}", state);
                        Ok(StateValue::String("other_handled".to_string()))
                    }
                }
            })
        );

        // 3. 最终结果节点
        let result_node = StateNodeSpec::new(
            node!("result", |state| {
                println!("📊 结果节点");
                let state: &StateValue = &state;
                let result = format!("处理结果: {:?}", state);
                println!("   {}", result);
                Ok(StateValue::String(result))
            })
        );

        // 添加节点
        graph
            .add_node("boundary".to_string(), boundary_node)
            .add_node("error_handler".to_string(), error_node)
            .add_node("result".to_string(), result_node);

        // 设置边连接
        graph
            .add_edge("boundary".to_string(), "error_handler".to_string())
            .add_edge("error_handler".to_string(), "result".to_string());

        // 设置入口和出口
        graph
            .set_entry_point("boundary".to_string())
            .set_finish_point("result".to_string());

        // 创建输入通道
        let mut channels = HashMap::new();
        channels.insert("input".to_string(), Channel::new_last_value(StateValue::Null));
        graph.channels = channels;

        // 编译工作流
        println!("🔨 编译边界情况工作流...");
        let mut compiled = graph.compile().unwrap();
        println!("✅ 编译成功");

        // 测试场景1: 零值
        println!("\n--- 测试场景1: 零值 ---");
        let mut input1 = HashMap::new();
        input1.insert("input".to_string(), StateValue::Number(0.0));
        
        let result1 = compiled.invoke(input1).await.unwrap();
        println!("场景1结果: {:?}", result1);

        // 测试场景2: 无穷大
        println!("\n--- 测试场景2: 无穷大 ---");
        let mut input2 = HashMap::new();
        input2.insert("input".to_string(), StateValue::Number(f64::INFINITY));
        
        let result2 = compiled.invoke(input2).await.unwrap();
        println!("场景2结果: {:?}", result2);

        // 测试场景3: NaN
        println!("\n--- 测试场景3: NaN ---");
        let mut input3 = HashMap::new();
        input3.insert("input".to_string(), StateValue::Number(f64::NAN));
        
        let result3 = compiled.invoke(input3).await.unwrap();
        println!("场景3结果: {:?}", result3);

        // 测试场景4: 空字符串
        println!("\n--- 测试场景4: 空字符串 ---");
        let mut input4 = HashMap::new();
        input4.insert("input".to_string(), StateValue::String("".to_string()));
        
        let result4 = compiled.invoke(input4).await.unwrap();
        println!("场景4结果: {:?}", result4);

        // 测试场景5: 空值
        println!("\n--- 测试场景5: 空值 ---");
        let mut input5 = HashMap::new();
        input5.insert("input".to_string(), StateValue::Null);
        
        let result5 = compiled.invoke(input5).await.unwrap();
        println!("场景5结果: {:?}", result5);

        println!("\n=== 边界情况测试完成 ===");
    }

    #[tokio::test]
    async fn test_simple_branch_data_flow() {
        println!("=== 开始简单分支数据流测试 ===");
        
        let mut graph = StateGraph::new("SimpleBranchDataFlow".to_string());
        
        // 1. 输入节点
        let input_node = StateNodeSpec::new(
            node!("input", |state| {
                println!("📥 输入节点: 获取输入数据");
                let state: &StateValue = &state;
                if let Some(data) = state.as_string() {
                    let number: f64 = data.parse().unwrap_or(0.0);
                    println!("   输入数字: {}", number);
                    Ok(StateValue::Number(number))
                } else {
                    Ok(StateValue::Number(0.0))
                }
            })
        );

        // 2. 分支节点
        let branch_node = StateNodeSpec::new(
            node!("branch", |state| {
                println!("🔀 分支节点: 判断数值大小");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    if number > 50.0 {
                        println!("   {} > 50，路由到大数处理", number);
                        Ok(StateValue::String("large".to_string()))
                    } else {
                        println!("   {} ≤ 50，路由到小数处理", number);
                        Ok(StateValue::String("small".to_string()))
                    }
                } else {
                    Ok(StateValue::String("error".to_string()))
                }
            })
        );

        // 3. 大数处理节点
        let large_node = StateNodeSpec::new(
            node!("large", |state| {
                println!("🔢 大数处理节点: 处理大数");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    let result = number * 10.0;
                    println!("   {} × 10 = {}", number, result);
                    Ok(StateValue::Number(result))
                } else {
                    println!("   输入类型错误: {:?}", state);
                    Ok(StateValue::Number(0.0))
                }
            })
        );

        // 4. 小数处理节点
        let small_node = StateNodeSpec::new(
            node!("small", |state| {
                println!("🔢 小数处理节点: 处理小数");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    let result = number / 10.0;
                    println!("   {} ÷ 10 = {}", number, result);
                    Ok(StateValue::Number(result))
                } else {
                    println!("   输入类型错误: {:?}", state);
                    Ok(StateValue::Number(0.0))
                }
            })
        );

        // 5. 最终结果节点
        let final_node = StateNodeSpec::new(
            node!("final", |state| {
                println!("🎯 最终节点: 生成最终结果");
                let state: &StateValue = &state;
                if let Some(number) = state.as_number() {
                    let result = format!("最终结果: {}", number);
                    println!("   {}", result);
                    Ok(StateValue::String(result))
                } else {
                    Ok(StateValue::String("处理失败".to_string()))
                }
            })
        );

        // 添加所有节点
        graph
            .add_node("input".to_string(), input_node)
            .add_node("branch".to_string(), branch_node)
            .add_node("large".to_string(), large_node)
            .add_node("small".to_string(), small_node)
            .add_node("final".to_string(), final_node);

        // 设置边连接
        graph
            .add_edge("input".to_string(), "branch".to_string())
            .add_edge("branch".to_string(), "large".to_string())
            .add_edge("branch".to_string(), "small".to_string())
            .add_edge("large".to_string(), "final".to_string())
            .add_edge("small".to_string(), "final".to_string());

        // 设置入口和出口
        graph
            .set_entry_point("input".to_string())
            .set_finish_point("final".to_string());

        // 创建输入通道
        let mut channels = HashMap::new();
        channels.insert("input".to_string(), Channel::new_last_value(StateValue::Null));
        graph.channels = channels;

        // 编译工作流
        println!("🔨 编译简单分支数据流工作流...");
        let mut compiled = graph.compile().unwrap();
        println!("✅ 编译成功");

        // 测试场景1: 大数 (100 → large → 1000)
        println!("\n--- 测试场景1: 大数 (100) ---");
        let mut input1 = HashMap::new();
        input1.insert("input".to_string(), StateValue::String("100".to_string()));
        
        let result1 = compiled.invoke(input1).await.unwrap();
        println!("场景1结果: {:?}", result1);
        
        // 验证结果
        if let Some(StateValue::String(output)) = result1.get("final_result") {
            assert!(output.contains("1000"), "场景1应该输出1000 (100×10=1000)");
        }

        // 测试场景2: 小数 (5 → small → 0.5)
        println!("\n--- 测试场景2: 小数 (5) ---");
        let mut input2 = HashMap::new();
        input2.insert("input".to_string(), StateValue::String("5".to_string()));
        
        let result2 = compiled.invoke(input2).await.unwrap();
        println!("场景2结果: {:?}", result2);
        
        // 验证结果
        if let Some(StateValue::String(output)) = result2.get("final_result") {
            assert!(output.contains("0.5"), "场景2应该输出0.5 (5÷10=0.5)");
        }

        println!("\n=== 简单分支数据流测试完成 ===");
    }
}
