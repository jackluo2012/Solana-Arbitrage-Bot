use std::str::FromStr;

use anyhow::Result;
use solana_program::pubkey::Pubkey;

/// 关于 pump AMM 配置的信息
#[derive(Debug)]
pub struct PumpAmmInfo {
    /// 基础代币的铸造地址
    pub base_mint: Pubkey,
    /// 报价代币的铸造地址
    pub quote_mint: Pubkey,
    /// 池的基础代币账户地址
    pub pool_base_token_account: Pubkey,
    /// 池的报价代币账户地址
    pub pool_quote_token_account: Pubkey,
    /// 代币创建者的金库授权地址
    pub coin_creator_vault_authority: Pubkey,
}
impl PumpAmmInfo {
    /// 从字节数据中加载并验证 PumpAmmInfo 结构体。
    ///
    /// 该函数解析传入的字节数据，提取必要的账户信息和公钥，并进行基本的数据长度校验。
    /// 如果数据格式不正确或长度不足，则返回错误。
    ///
    /// # 参数
    /// - `data`: 包含序列化 AMM 信息的字节切片
    ///
    /// # 返回值
    /// - `Ok(Self)`: 成功解析出的 PumpAmmInfo 实例
    // 用户想交易 → 需要知道:
    // 1. 交易哪两个代币? (base_mint, quote_mint)
    // 2. 池子地址在哪? (pool_base_token_account, pool_quote_token_account)
    // 3. 谁有权管理资金? (coin_creator_vault_authority)
    // 假设有人创建了一个叫"COIN"的代币：

    // base_mint = COIN代币地址
    // quote_mint = SOL代币地址
    // pool_base_token_account = 存放COIN代币的池账户
    // pool_quote_token_account = 存放SOL代币的池账户
    // coin_creator_vault_authority = 创建者资金管理权限地址
    // 当用户想用1个SOL买COIN时，协议会从pool_quote_token_account中取出SOL，从pool_base_token_account中给出相应数量的COIN。

    // 所以这些地址的存在是为了让AMM知道在哪里找到交易所需的代币和如何管理这些资金。
    /// - `Err(...)`: 数据不合法时返回错误信息
    pub fn load_checked(data: &[u8]) -> Result<Self> {
        // 跳过前缀数据（8字节signature + 1字节bump + 2字节version + 32字节padding）
        let data = &data[8 + 1 + 2 + 32..];

        // 检查剩余数据是否足够包含4个Pubkey（各32字节）和lp_supply（8字节）
        if data.len() < 4 * 32 + 8 {
            // 4 Pubkeys (32 bytes each) + lp_supply (8 bytes)
            return Err(anyhow::anyhow!("Invalid data length for PumpAmmInfo"));
        }

        // 提取基础代币和报价代币的 mint 地址
        // 为什么需要报价代币？ 在交易中，你需要知道"用什么换什么"。比如你想用SOL购买某个新代币，那么：

        // base_mint = 新代币的地址
        // quote_mint = SOL的地址（或USDC地址）
        //基础代币的铸造地址
        let base_mint = Pubkey::from(<[u8; 32]>::try_from(&data[0..32]).unwrap());
        //报价代币的铸造地址
        let quote_mint = Pubkey::from(<[u8; 32]>::try_from(&data[32..64]).unwrap());

        // 提取池中基础代币和报价代币的账户地址
        // AMM池中的基础代币账户地址
        let pool_base_token_account = Pubkey::from(<[u8; 32]>::try_from(&data[96..128]).unwrap());
        //  AMM池中的报价代币账户地址
        let pool_quote_token_account = Pubkey::from(<[u8; 32]>::try_from(&data[128..160]).unwrap());

        // 固定的 Pump Program ID
        let pump_program_id =
            Pubkey::from_str("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA").unwrap();
        println!("data: {:?}", data.len());

        // 解析代币创建者地址（如果存在）
        let coin_creator = if data.len() < 257 {
            Pubkey::default()
        } else {
            Pubkey::try_from(&data[168..200])?
        };

        // 根据代币创建者派生 vault 权限地址
        let key = Pubkey::find_program_address(
            &[b"creator_vault", coin_creator.as_ref()],
            &pump_program_id,
        );

        println!("coin_creator: {:?}", key.0);

        // 构造并返回结构体实例
        // 在Solana区块链上，当你需要与Pump协议的AMM交互时，需要从链上账户数据中解析出这些信息，以便：
        // 知道交易的是什么代币对
        // 知道池子的账户地址
        // 知道资金管理的权限地址
        // 简单来说，这段代码就是把链上原始的二进制数据转换成我们能理解和使用的结构化数据。
        Ok(Self {
            base_mint,
            quote_mint,
            pool_base_token_account,
            pool_quote_token_account,
            coin_creator_vault_authority: key.0,
        })
    }
}
