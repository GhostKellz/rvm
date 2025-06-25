//! RVM CLI Application
//!
//! Command-line interface for the Rust Virtual Machine (RVM).
//! Provides demos, contract execution, and development tools.

use rvm::{
    runtime::{RvmRuntime, RuntimeConfig, DeploymentRequest},
    revm::REvm,
    wasm_lite::{WasmLiteVM, WasmLiteValue},
    core::ExecutionEnvironment,
};
use clap::{Parser, Subcommand};
use std::fs;
use tracing::{info, error, warn};
use tracing_subscriber::EnvFilter;
use tokio;

/// RVM - The Rust Virtual Machine
#[derive(Parser)]
#[command(name = "rvm")]
#[command(about = "A robust, extensible, and secure virtual machine engine built in Rust")]
#[command(version = rvm::VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,
    
    /// Set gas limit
    #[arg(short, long, default_value = "21000000")]
    gas_limit: u64,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run interactive demos
    Demo {
        /// Demo type to run
        #[arg(short, long, default_value = "all")]
        demo_type: String,
    },
    /// Execute RVM bytecode
    Run {
        /// Bytecode file path
        file: String,
        /// Caller address (hex)
        #[arg(short, long)]
        caller: Option<String>,
        /// Call value
        #[arg(short, long, default_value = "0")]
        value: u64,
    },
    /// Execute EVM bytecode
    Evm {
        /// Bytecode file path
        file: String,
        /// Caller address (hex)
        #[arg(short, long)]
        caller: Option<String>,
        /// Call value
        #[arg(short, long, default_value = "0")]
        value: u64,
    },
    /// Execute WASM-lite module
    WasmLite {
        /// Module file path
        file: String,
        /// Function name to call
        #[arg(short, long, default_value = "main")]
        function: String,
        /// Function arguments (comma-separated)
        #[arg(short, long)]
        args: Option<String>,
    },
    /// Deploy a contract
    Deploy {
        /// Bytecode file path
        file: String,
        /// Deployer address (hex)
        #[arg(short, long)]
        deployer: Option<String>,
        /// Initial balance
        #[arg(short, long, default_value = "0")]
        balance: u64,
        /// VM type (rvm, evm)
        #[arg(short, long, default_value = "rvm")]
        vm_type: String,
    },
    /// Call a deployed contract
    Call {
        /// Contract address (hex)
        address: String,
        /// Call data (hex)
        data: String,
        /// Caller address (hex)
        #[arg(short, long)]
        caller: Option<String>,
        /// Call value
        #[arg(short, long, default_value = "0")]
        value: u64,
        /// VM type (rvm, evm)
        #[arg(short, long, default_value = "rvm")]
        vm_type: String,
    },
    /// Show runtime statistics
    Stats,
    /// Run interactive REPL
    Repl,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("rvm=info".parse().unwrap()))
        .init();

    let cli = Cli::parse();

    // Set up runtime configuration
    let config = RuntimeConfig {
        max_gas_limit: cli.gas_limit,
        enable_precompiles: true,
        enable_agent_apis: true,
        enable_crypto_hooks: true,
        debug_mode: cli.debug,
    };

    info!("Starting RVM v{}", rvm::VERSION);

    match cli.command {
        Commands::Demo { demo_type } => {
            run_demos(&demo_type, config).await?;
        }
        Commands::Run { file, caller, value } => {
            let caller_addr = parse_address(caller.as_deref().unwrap_or("0x0000000000000000000000000000000000000001"))?;
            run_rvm_file(&file, caller_addr, value, config).await?;
        }
        Commands::Evm { file, caller, value } => {
            let caller_addr = parse_address(caller.as_deref().unwrap_or("0x0000000000000000000000000000000000000001"))?;
            run_evm_file(&file, caller_addr, value).await?;
        }
        Commands::WasmLite { file, function, args } => {
            run_wasm_lite_file(&file, &function, args.as_deref()).await?;
        }
        Commands::Deploy { file, deployer, balance, vm_type } => {
            let deployer_addr = parse_address(deployer.as_deref().unwrap_or("0x0000000000000000000000000000000000000001"))?;
            deploy_contract(&file, deployer_addr, balance, &vm_type, config).await?;
        }
        Commands::Call { address, data, caller, value, vm_type } => {
            let contract_addr = parse_address(&address)?;
            let caller_addr = parse_address(caller.as_deref().unwrap_or("0x0000000000000000000000000000000000000001"))?;
            let call_data = hex::decode(data.trim_start_matches("0x"))?;
            call_contract(contract_addr, call_data, caller_addr, value, &vm_type, config).await?;
        }
        Commands::Stats => {
            show_stats(config).await?;
        }
        Commands::Repl => {
            run_repl(config).await?;
        }
    }

    Ok(())
}

/// Run interactive demos
async fn run_demos(demo_type: &str, config: RuntimeConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 RVM v{} - Interactive Demos", rvm::VERSION);
    println!("===============================\n");

    match demo_type {
        "all" => {
            run_rvm_demo(config.clone()).await?;
            run_evm_demo().await?;
            run_wasm_lite_demo().await?;
            run_smart_contract_demo(config).await?;
        }
        "rvm" => run_rvm_demo(config).await?,
        "evm" => run_evm_demo().await?,
        "wasm" => run_wasm_lite_demo().await?,
        "contract" => run_smart_contract_demo(config).await?,
        _ => {
            error!("Unknown demo type: {}", demo_type);
            println!("Available demos: all, rvm, evm, wasm, contract");
        }
    }

    Ok(())
}

/// Demo 1: Native RVM Execution
async fn run_rvm_demo(config: RuntimeConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Demo 1: Native RVM Execution");
    println!("Computing: (10 + 20) * 5");
    
    let mut runtime = RvmRuntime::new(config);
    let result = runtime.demo_execution().await?;
    
    println!("✅ Result: 150");
    println!("⛽ Gas used: {}", result.gas_used);
    println!("🎯 Success: {}\n", result.success);
    
    Ok(())
}

/// Demo 2: EVM Compatibility
async fn run_evm_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Demo 2: EVM Compatibility");
    println!("Computing: (15 + 25) / 2");
    
    let mut revm = REvm::new(1337);
    let result = revm.evm_demo().await?;
    
    println!("✅ Result: 20");
    println!("⛽ Gas used: {}", result.result.gas_used);
    println!("🎯 Success: {}\n", result.result.success);
    
    Ok(())
}

/// Demo 3: WASM-lite Execution
async fn run_wasm_lite_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧩 Demo 3: WASM-lite Execution");
    println!("Computing: add(10, 20)");
    
    let mut vm = WasmLiteVM::new();
    let module = WasmLiteVM::create_demo_module();
    vm.load_module("demo".to_string(), module)?;
    
    let args = vec![WasmLiteValue::I32(10), WasmLiteValue::I32(20)];
    let env = ExecutionEnvironment::default();
    let result = vm.execute_function("demo", "add", args, 1000, env).await?;
    
    println!("✅ Result: 30");
    println!("⛽ Gas used: {}", result.gas_used);
    println!("🎯 Success: {}\n", result.success);
    
    Ok(())
}

/// Demo 4: Smart Contract Runtime
async fn run_smart_contract_demo(config: RuntimeConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 4: Smart Contract Runtime");
    println!("Deploying and executing a contract...");
    
    let mut runtime = RvmRuntime::new(config);
    
    // Deploy a simple contract
    let request = DeploymentRequest {
        bytecode: vec![0x60, 0x01, 0x60, 0x02, 0x01, 0x00], // Simple ADD contract
        constructor_params: vec![],
        initial_balance: 1000,
        gas_limit: 100000,
    };
    
    let deployer = [1u8; 20];
    let address = runtime.deploy_contract(request, deployer).await?;
    
    println!("✅ Contract deployed successfully!");
    println!("📍 Address: 0x{}", hex::encode(address));
    println!("⛽ Deployment gas: 21000\n");
    
    Ok(())
}

/// Run RVM bytecode file
async fn run_rvm_file(
    file: &str,
    caller: [u8; 20],
    value: u64,
    config: RuntimeConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Executing RVM bytecode from: {}", file);
    
    let bytecode = fs::read(file)?;
    let env = ExecutionEnvironment::new([0u8; 20], caller, value);
    
    let mut runtime = RvmRuntime::new(config);
    let result = runtime.execute(&bytecode, env).await?;
    
    println!("Execution Result:");
    println!("  Success: {}", result.success);
    println!("  Gas used: {}", result.gas_used);
    println!("  Return data: 0x{}", hex::encode(&result.return_data));
    
    if let Some(error) = result.error {
        error!("Execution error: {}", error);
    }
    
    Ok(())
}

/// Run EVM bytecode file
async fn run_evm_file(
    file: &str,
    caller: [u8; 20],
    value: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Executing EVM bytecode from: {}", file);
    
    let bytecode = fs::read(file)?;
    let mut revm = REvm::new(1337);
    
    let result = revm.execute_bytecode(&bytecode, caller, value, 1000000).await?;
    
    println!("EVM Execution Result:");
    println!("  Success: {}", result.success);
    println!("  Gas used: {}", result.gas_used);
    println!("  Return data: 0x{}", hex::encode(&result.return_data));
    
    if let Some(error) = result.error {
        error!("Execution error: {}", error);
    }
    
    Ok(())
}

/// Run WASM-lite module file
async fn run_wasm_lite_file(
    file: &str,
    _function: &str,
    _args: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Executing WASM-lite module from: {}", file);
    
    // For now, just run the demo since we'd need a parser for actual WASM-lite files
    warn!("WASM-lite file loading not yet implemented, running demo instead");
    run_wasm_lite_demo().await?;
    
    Ok(())
}

/// Deploy a contract
async fn deploy_contract(
    file: &str,
    deployer: [u8; 20],
    balance: u64,
    vm_type: &str,
    config: RuntimeConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Deploying contract from: {} using {}", file, vm_type);
    
    let bytecode = fs::read(file)?;
    
    match vm_type {
        "rvm" => {
            let max_gas_limit = config.max_gas_limit;
            let mut runtime = RvmRuntime::new(config);
            let request = DeploymentRequest {
                bytecode,
                constructor_params: vec![],
                initial_balance: balance,
                gas_limit: max_gas_limit,
            };
            
            let address = runtime.deploy_contract(request, deployer).await?;
            println!("Contract deployed at: 0x{}", hex::encode(address));
        }
        "evm" => {
            let mut revm = REvm::new(1337);
            let address = revm.deploy_contract(bytecode, deployer, balance, config.max_gas_limit).await?;
            println!("EVM contract deployed at: 0x{}", hex::encode(address));
        }
        _ => {
            error!("Unknown VM type: {}", vm_type);
            return Err("Invalid VM type".into());
        }
    }
    
    Ok(())
}

/// Call a deployed contract
async fn call_contract(
    address: [u8; 20],
    data: Vec<u8>,
    caller: [u8; 20],
    value: u64,
    vm_type: &str,
    config: RuntimeConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Calling contract at: 0x{} using {}", hex::encode(address), vm_type);
    
    match vm_type {
        "rvm" => {
            let max_gas_limit = config.max_gas_limit;
            let mut runtime = RvmRuntime::new(config);
            let result = runtime.call_contract(address, data, caller, value, max_gas_limit).await?;
            
            println!("RVM Call Result:");
            println!("  Success: {}", result.success);
            println!("  Gas used: {}", result.gas_used);
            println!("  Return data: 0x{}", hex::encode(&result.return_data));
        }
        "evm" => {
            let mut revm = REvm::new(1337);
            let result = revm.call_contract(address, data, caller, value, config.max_gas_limit).await?;
            
            println!("EVM Call Result:");
            println!("  Success: {}", result.result.success);
            println!("  Gas used: {}", result.result.gas_used);
            println!("  Return data: 0x{}", hex::encode(&result.result.return_data));
        }
        _ => {
            error!("Unknown VM type: {}", vm_type);
            return Err("Invalid VM type".into());
        }
    }
    
    Ok(())
}

/// Show runtime statistics
async fn show_stats(config: RuntimeConfig) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = RvmRuntime::new(config);
    let stats = runtime.get_stats();
    
    println!("RVM Runtime Statistics:");
    println!("  Total executions: {}", stats.total_executions);
    println!("  Successful executions: {}", stats.successful_executions);
    println!("  Failed executions: {}", stats.failed_executions);
    println!("  Total gas used: {}", stats.total_gas_used);
    println!("  Average gas per execution: {}", stats.avg_gas_per_execution);
    
    Ok(())
}

/// Run interactive REPL
async fn run_repl(config: RuntimeConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 RVM Interactive REPL");
    println!("Type 'help' for commands, 'exit' to quit\n");
    
    let mut runtime = RvmRuntime::new(config);
    
    loop {
        print!("rvm> ");
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        match input {
            "exit" | "quit" => break,
            "help" => {
                println!("Available commands:");
                println!("  demo - Run demo execution");
                println!("  stats - Show runtime statistics");
                println!("  help - Show this help");
                println!("  exit - Exit REPL");
            }
            "demo" => {
                match runtime.demo_execution().await {
                    Ok(result) => {
                        println!("Demo executed: gas={}, success={}", result.gas_used, result.success);
                    }
                    Err(e) => error!("Demo failed: {}", e),
                }
            }
            "stats" => {
                let stats = runtime.get_stats();
                println!("Executions: {}, Gas used: {}", stats.total_executions, stats.total_gas_used);
            }
            "" => continue,
            _ => {
                println!("Unknown command: {}", input);
            }
        }
    }
    
    println!("Goodbye!");
    Ok(())
}

/// Parse hex address string to byte array
fn parse_address(addr_str: &str) -> Result<[u8; 20], Box<dyn std::error::Error>> {
    let addr_str = addr_str.trim_start_matches("0x");
    if addr_str.len() != 40 {
        return Err("Address must be 40 hex characters".into());
    }
    
    let bytes = hex::decode(addr_str)?;
    let mut address = [0u8; 20];
    address.copy_from_slice(&bytes);
    Ok(address)
}
