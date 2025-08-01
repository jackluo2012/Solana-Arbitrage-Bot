use crate::config::Config;
use crate::refresh::initialize_pool_data;
use crate::transaction::build_and_send_transaction;
use anyhow::Context;
use solana_client::rpc_client::RpcClient;
use solana_sdk::address_lookup_table::AddressLookupTableAccount;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::{
    address_lookup_table::state::AddressLookupTable, compute_budget::ComputeBudgetInstruction,
};
use spl_associated_token_account::{
    get_associated_token_address, get_associated_token_address_with_program_id,
};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

/// 启动并运行交易机器人。
///
/// 该函数负责加载配置、初始化 RPC 客户端、加载钱包密钥对、刷新最新 blockhash、
/// 检查或创建代币账户，并为每个代币配置启动独立的交易发送任务。
///
/// # 参数
/// * `config_path` - 配置文件路径，用于加载机器人运行所需的各项配置。
///
/// # 返回值
/// 返回 `anyhow::Result<()>`，表示运行过程中是否发生错误。
pub async fn run_bot(config_path: &str) -> anyhow::Result<()> {
    let config = Config::load(config_path)?;
    info!("Configuration loaded successfully");

    // 创建一个新的RPC客户端实例
    //
    // 该代码行执行以下操作：
    // 1. 从配置中克隆RPC服务器的URL地址
    // 2. 使用该URL创建一个新的RpcClient实例
    // 3. 将RpcClient包装在Arc智能指针中以支持多线程共享
    //
    // 返回值：Arc<RpcClient> - 线程安全的RPC客户端引用计数智能指针
    let rpc_client = Arc::new(RpcClient::new(config.rpc.url.clone()));

    // 根据配置决定RPC客户端列表的构建方式
    // 如果启用了spam配置，则使用配置中的多个RPC URL创建客户端列表
    // 否则只使用默认的rpc_client克隆版本
    let sending_rpc_clients = if let Some(spam_config) = &config.spam {
        // 检查是否启用spam功能
        if spam_config.enabled {
            // 当spam启用时，为每个配置的RPC URL创建新的RpcClient实例
            spam_config
                .sending_rpc_urls
                .iter()
                .map(|url| Arc::new(RpcClient::new(url.clone())))
                .collect::<Vec<_>>()
        } else {
            // spam配置存在但未启用时，使用默认RPC客户端
            vec![rpc_client.clone()]
        }
    } else {
        // 无spam配置时，使用默认RPC客户端
        vec![rpc_client.clone()]
    };
    // 加载钱包密钥对
    let wallet_kp =
        load_keypair(&config.wallet.private_key).context("Failed to load wallet keypair")?;
    info!("Wallet loaded: {}", wallet_kp.pubkey());

    // 获取最新的区块哈希值，用于后续的交易签名和验证
    // 该操作通过RPC客户端与区块链网络交互，获取当前最新的区块哈希
    let initial_blockhash = rpc_client.get_latest_blockhash()?;

    // 将获取到的初始区块哈希值包装为线程安全的共享引用
    // 使用Arc<Mutex<T>>结构实现多线程环境下的安全访问和修改
    // 这样可以在多个作用域或线程中共享和更新区块哈希值
    let cached_blockhash = Arc::new(Mutex::new(initial_blockhash));
    let refresh_interval = Duration::from_secs(10);
    let blockhash_client = rpc_client.clone();
    let blockhash_cache = cached_blockhash.clone();

    // 启动后台任务定期刷新 blockhash 缓存
    tokio::spawn(async move {
        blockhash_refresher(blockhash_client, blockhash_cache, refresh_interval).await;
    });

    // 遍历所有代币配置，检查并创建对应的关联代币账户（ATA）
    for mint_config in &config.routing.mint_config_list {
        // 获取代币的 owner program ID（如 Token Program 或 Token-2022）
        // 通过RPC客户端获取mint账户的所有者信息
        // 该代码块完成了以下操作：
        // 1. 将mint_config中的mint字符串转换为Pubkey类型
        // 2. 使用RPC客户端查询该账户的详细信息
        // 3. 提取账户的所有者(owner)字段
        // 它获取的是代币铸造账户（mint account）的所有者程序ID。
        let mint_owner = rpc_client
            .get_account(&Pubkey::from_str(&mint_config.mint).unwrap())
            .unwrap()
            .owner;

        // 获取钱包关联的代币账户地址
        //
        // 该函数通过钱包公钥、铸币地址和铸币所有者程序ID来计算关联的代币账户地址。
        // 主要用于SPL代币标准中，为特定钱包和代币组合生成确定性的关联账户地址。
        //
        // # 参数
        // * `wallet_kp.pubkey()` - 钱包的公钥，用于生成关联账户
        // * `Pubkey::from_str(&mint_config.mint).unwrap()` - 铸币地址，表示特定代币的mint地址
        // * `&mint_owner` - 铸币所有者程序ID，通常是代币程序的地址
        //
        // # 返回值
        // 返回计算得到的关联代币账户地址
        let wallet_token_account = get_associated_token_address_with_program_id(
            &wallet_kp.pubkey(),
            &Pubkey::from_str(&mint_config.mint).unwrap(),
            &mint_owner,
        );

        println!("   Token mint: {}", mint_config.mint);
        println!("   Wallet token ATA: {}", wallet_token_account);
        // 检查钱包的关联代币账户是否存在，若不存在则创建
        println!("\n   Checking if token account exists...");
        loop {
            match rpc_client.get_account(&wallet_token_account) {
                Ok(_) => {
                    println!("   token account exists!");
                    break;
                }
                Err(_) => {
                    println!("   token account does not exist. Creating it...");

                    // 构造创建 ATA 的指令（幂等创建）
                    let create_ata_ix =
                            spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                                &wallet_kp.pubkey(), // Funding account
                                &wallet_kp.pubkey(), // Wallet account
                                &Pubkey::from_str(&mint_config.mint).unwrap(),   // Token mint
                                &spl_token::ID,      // Token program
                            );

                    // 获取最新的 blockhash 用于交易签名
                    let blockhash = rpc_client.get_latest_blockhash()?;

                    // 创建设置计算单元价格的指令，参数为每计算单元的价格（微 lamports）
                    let compute_unit_price_ix =
                        ComputeBudgetInstruction::set_compute_unit_price(1_000_000);

                    // 创建设置计算单元限制的指令，参数为交易允许使用的最大计算单元数量
                    let compute_unit_limit_ix =
                        ComputeBudgetInstruction::set_compute_unit_limit(60_000);

                    // 构造并签名交易
                    let create_ata_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
                        &[compute_unit_price_ix, compute_unit_limit_ix, create_ata_ix],
                        Some(&wallet_kp.pubkey()),
                        &[&wallet_kp],
                        blockhash,
                    );

                    // 发送并确认交易
                    match rpc_client.send_and_confirm_transaction(&create_ata_tx) {
                        Ok(sig) => {
                            println!("   token account created successfully! Signature: {}", sig);
                        }
                        Err(e) => {
                            println!("   Failed to create token account: {:?}", e);
                            return Err(anyhow::anyhow!("Failed to create token account"));
                        }
                    }
                }
            }
        }
    }

    // 为每个代币配置初始化池数据并启动交易发送任务->这个只运行一次
    for mint_config in &config.routing.mint_config_list {
        info!("Processing mint: {}", mint_config.mint);

        let pool_data = initialize_pool_data(
            &mint_config.mint,
            &wallet_kp.pubkey().to_string(),
            mint_config.raydium_pool_list.as_ref(),
            mint_config.raydium_cp_pool_list.as_ref(),
            mint_config.pump_pool_list.as_ref(),
            mint_config.meteora_dlmm_pool_list.as_ref(),
            mint_config.whirlpool_pool_list.as_ref(),
            mint_config.raydium_clmm_pool_list.as_ref(),
            mint_config.meteora_damm_pool_list.as_ref(),
            mint_config.solfi_pool_list.as_ref(),
            mint_config.meteora_damm_v2_pool_list.as_ref(),
            mint_config.vertigo_pool_list.as_ref(),
            rpc_client.clone(),
        )
        .await?;

        let mint_pool_data = Arc::new(Mutex::new(pool_data));

        // TODO: Add logic to periodically refresh pool data

        // 克隆配置以在线程中使用
        let config_clone = config.clone();
        // 克隆当前代币配置以在线程中使用
        let mint_config_clone = mint_config.clone();
        // 克隆RPC客户端列表以在线程中使用
        let sending_rpc_clients_clone = sending_rpc_clients.clone();
        // 克隆区块哈希缓存以在线程中使用
        let cached_blockhash_clone = cached_blockhash.clone();
        // 获取钱包密钥对的字节表示，以便后续克隆
        let wallet_bytes = wallet_kp.to_bytes();
        // 从字节数据重新创建钱包密钥对以在线程中使用
        let wallet_kp_clone = Keypair::from_bytes(&wallet_bytes).unwrap();
        // 获取查找表账户列表，如果不存在则使用默认空列表
        let mut lookup_table_accounts = mint_config_clone.lookup_table_accounts.unwrap_or_default();
        // 添加默认的查找表账户地址到列表中
        lookup_table_accounts.push("4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_string());

        let mut lookup_table_accounts_list = vec![];

        // 加载地址查找表（Address Lookup Tables）用于交易优化
        // 处理查找表账户列表，加载并验证每个查找表账户
        //
        // 该函数遍历提供的查找表账户地址字符串列表，对每个地址进行以下操作：
        // 1. 将字符串解析为公钥(Pubkey)
        // 2. 从RPC客户端获取对应的账户数据
        // 3. 反序列化账户数据为地址查找表
        // 4. 将有效的查找表添加到结果列表中
        //
        // 参数:
        // * `lookup_table_accounts`: 包含查找表账户地址字符串的迭代器
        // * `rpc_client`: 用于获取账户数据的RPC客户端引用
        // * `lookup_table_accounts_list`: 用于存储成功加载的查找表账户的可变引用向量
        //
        // 返回值:
        // 无返回值，通过修改传入的lookup_table_accounts_list参数返回结果
        //
        // 错误处理:
        // - 无效的公钥字符串：记录错误日志并跳过该查找表
        // - 获取账户失败：记录错误日志并跳过该查找表
        // - 反序列化失败：记录错误日志并跳过该查找表
        // - 所有错误都不会中断整个处理流程，而是继续处理下一个查找表
        for lookup_table_account in lookup_table_accounts {
            // 尝试将查找表账户字符串解析为公钥
            match Pubkey::from_str(&lookup_table_account) {
                Ok(pubkey) => {
                    // 使用公钥从RPC客户端获取账户数据
                    match rpc_client.get_account(&pubkey) {
                        Ok(account) => {
                            // 尝试将账户数据反序列化为地址查找表
                            match AddressLookupTable::deserialize(&account.data) {
                                Ok(lookup_table) => {
                                    // 成功反序列化后，创建查找表账户对象并添加到结果列表
                                    let lookup_table_account = AddressLookupTableAccount {
                                        key: pubkey,
                                        addresses: lookup_table.addresses.into_owned(),
                                    };
                                    lookup_table_accounts_list.push(lookup_table_account);
                                    info!("   Successfully loaded lookup table: {}", pubkey);
                                }
                                Err(e) => {
                                    error!(
                                        "   Failed to deserialize lookup table {}: {}",
                                        pubkey, e
                                    );
                                    continue; // Skip this lookup table but continue processing others
                                }
                            }
                        }
                        Err(e) => {
                            error!("   Failed to fetch lookup table account {}: {}", pubkey, e);
                            continue; // Skip this lookup table but continue processing others
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "   Invalid lookup table pubkey string {}: {}",
                        lookup_table_account, e
                    );
                    continue; // Skip this lookup table but continue processing others
                }
            }
        }

        if lookup_table_accounts_list.is_empty() {
            warn!("   Warning: No valid lookup tables were loaded");
        } else {
            info!(
                "   Loaded {} lookup tables successfully",
                lookup_table_accounts_list.len()
            );
        }

        // 启动交易发送任务
        tokio::spawn(async move {
            let process_delay = Duration::from_millis(mint_config_clone.process_delay);

            loop {
                let latest_blockhash = {
                    let guard = cached_blockhash_clone.lock().await;
                    *guard
                };

                let guard = mint_pool_data.lock().await;

                match build_and_send_transaction(
                    &wallet_kp_clone,
                    &config_clone,
                    &*guard, // Dereference the guard here
                    &sending_rpc_clients_clone,
                    latest_blockhash,
                    &lookup_table_accounts_list,
                )
                .await
                {
                    Ok(signatures) => {
                        info!(
                            "Transactions sent successfully for mint {}",
                            mint_config_clone.mint
                        );
                        for signature in signatures {
                            info!("  Signature: {}", signature);
                        }
                    }
                    Err(e) => {
                        error!(
                            "Error sending transaction for mint {}: {}",
                            mint_config_clone.mint, e
                        );
                    }
                }

                tokio::time::sleep(process_delay).await;
            }
        });
    }

    // 主线程保持运行，防止程序退出
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// 异步函数，用于定期刷新并缓存最新的区块哈希值
///
/// 该函数会持续运行一个循环，定期从RPC客户端获取最新的区块哈希，
/// 并将其存储在共享的缓存中供其他组件使用。
///
/// # 参数
/// * `rpc_client` - RPC客户端的Arc引用，用于与区块链节点通信获取最新区块哈希
/// * `cached_blockhash` - 通过Arc<Mutex<Hash>>包装的共享区块哈希缓存
/// * `refresh_interval` - 刷新间隔时间，控制获取新区块哈希的频率
async fn blockhash_refresher(
    rpc_client: Arc<RpcClient>,
    cached_blockhash: Arc<Mutex<Hash>>,
    refresh_interval: Duration,
) {
    // 持续循环刷新区块哈希
    loop {
        // 尝试获取最新的区块哈希
        match rpc_client.get_latest_blockhash() {
            Ok(blockhash) => {
                // 成功获取区块哈希，更新缓存
                let mut guard = cached_blockhash.lock().await;
                *guard = blockhash;
                info!("Blockhash refreshed: {}", blockhash);
            }
            Err(e) => {
                // 获取区块哈希失败，记录错误日志
                error!("Failed to refresh blockhash: {:?}", e);
            }
        }
        // 等待指定的刷新间隔时间
        tokio::time::sleep(refresh_interval).await;
    }
}

/// 从字符串加载密钥对
///
/// 该函数尝试从给定的字符串加载Solana密钥对。它首先尝试将字符串解析为base58编码的
/// 私钥字节，如果失败则尝试将字符串作为文件路径读取密钥对文件。
///
/// # 参数
/// * `private_key` - 包含私钥的字符串，可以是base58编码的私钥或密钥对文件路径
///
/// # 返回值
/// * `Ok(Keypair)` - 成功加载的密钥对
/// * `Err(anyhow::Error)` - 加载失败时返回错误信息
fn load_keypair(private_key: &str) -> anyhow::Result<Keypair> {
    // 尝试将输入字符串解析为base58编码的密钥对
    if let Ok(keypair) = bs58::decode(private_key)
        .into_vec()
        .map_err(|e| anyhow::anyhow!("Failed to decode base58: {}", e))
        .and_then(|bytes| {
            Keypair::from_bytes(&bytes).map_err(|e| anyhow::anyhow!("Invalid keypair bytes: {}", e))
        })
    {
        return Ok(keypair);
    }

    // 如果base58解析失败，尝试将输入作为文件路径读取密钥对文件
    if let Ok(keypair) = solana_sdk::signature::read_keypair_file(private_key) {
        return Ok(keypair);
    }

    anyhow::bail!("Failed to load keypair from: {}", private_key)
}
