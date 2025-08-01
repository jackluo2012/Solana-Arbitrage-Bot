use anyhow::Result;
use solana_program::pubkey::Pubkey;

const AMM_CONFIG_OFFSET: usize = 8; // amm_config
const POOL_CREATOR_OFFSET: usize = 40; // pool_creator
const TOKEN_0_VAULT_OFFSET: usize = 72; // token_0_vault
const TOKEN_1_VAULT_OFFSET: usize = 104; // token_1_vault
const LP_MINT_OFFSET: usize = 136; // lp_mint
const TOKEN_0_MINT_OFFSET: usize = 168; // token_0_mint
const TOKEN_1_MINT_OFFSET: usize = 200; // token_1_mint
const TOKEN_0_PROGRAM_OFFSET: usize = 232; // token_0_program
const TOKEN_1_PROGRAM_OFFSET: usize = 264; // token_1_program
const OBSERVATION_KEY_OFFSET: usize = 296; // observation_key

#[derive(Debug)]
pub struct RaydiumCpAmmInfo {
    pub token_0_mint: Pubkey,
    pub token_1_mint: Pubkey,
    pub token_0_vault: Pubkey,
    pub token_1_vault: Pubkey,
    pub amm_config: Pubkey,
    pub observation_key: Pubkey,
}

impl RaydiumCpAmmInfo {
    /// 从字节数据中加载并验证 RaydiumCpAmmInfo 结构体
    ///
    /// 该函数负责解析原始字节数据，提取AMM池的相关账户信息，并进行基本的长度验证
    /// 以确保数据完整性。
    ///
    /// # 参数
    /// * `data` - 包含AMM信息的原始字节数据切片
    ///
    /// # 返回值
    /// * `Result<Self>` - 成功时返回解析后的 RaydiumCpAmmInfo 实例，失败时返回错误信息
    ///
    /// # 错误
    /// 当数据长度不足或无法解析公钥时会返回相应的错误
    pub fn load_checked(data: &[u8]) -> Result<Self> {
        // 验证数据长度是否足够包含所有必需的字段
        if data.len() < OBSERVATION_KEY_OFFSET + 32 {
            return Err(anyhow::anyhow!("Invalid data length for RaydiumCpAmmInfo"));
        }

        // 从指定偏移位置提取各个账户的公钥信息
        let token_0_vault =
            Pubkey::try_from(&data[TOKEN_0_VAULT_OFFSET..TOKEN_0_VAULT_OFFSET + 32])?;
        let token_1_vault =
            Pubkey::try_from(&data[TOKEN_1_VAULT_OFFSET..TOKEN_1_VAULT_OFFSET + 32])?;
        let token_0_mint = Pubkey::try_from(&data[TOKEN_0_MINT_OFFSET..TOKEN_0_MINT_OFFSET + 32])?;
        let token_1_mint = Pubkey::try_from(&data[TOKEN_1_MINT_OFFSET..TOKEN_1_MINT_OFFSET + 32])?;
        let amm_config = Pubkey::try_from(&data[AMM_CONFIG_OFFSET..AMM_CONFIG_OFFSET + 32])?;
        let observation_key =
            Pubkey::try_from(&data[OBSERVATION_KEY_OFFSET..OBSERVATION_KEY_OFFSET + 32])?;

        // 构造并返回解析后的结构体实例
        Ok(Self {
            token_0_mint,
            token_1_mint,
            token_0_vault,
            token_1_vault,
            amm_config,
            observation_key,
        })
    }
}
