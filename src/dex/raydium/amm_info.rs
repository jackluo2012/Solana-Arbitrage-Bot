use anyhow::Result;
use solana_program::pubkey::Pubkey;

const COIN_VAULT_OFFSET: usize = 336; // coinVault/tokenVaultA
const PC_VAULT_OFFSET: usize = 368; // pcVault/tokenVaultB
const COIN_MINT_OFFSET: usize = 400; // coinMint/tokenMintA
const PC_MINT_OFFSET: usize = 432; // pcMint/tokenMintB

#[derive(Debug)]
/// Raydium AMM 信息结构体
///
/// 该结构体存储了 Raydium 自动做市商(AMM)的核心账户信息，
/// 包括代币的铸造地址和对应的金库地址
pub struct RaydiumAmmInfo {
    /// 代币A的铸造地址(Pubkey)
    pub coin_mint: Pubkey,
    /// 代币B的铸造地址(Pubkey)
    pub pc_mint: Pubkey,
    /// 代币A的金库地址(Pubkey)
    pub coin_vault: Pubkey,
    /// 代币B的金库地址(Pubkey)
    pub pc_vault: Pubkey,
}

impl RaydiumAmmInfo {
    /// 从字节数据中加载并验证RaydiumAmmInfo结构体
    ///
    /// 该函数会检查数据长度是否足够，并从指定偏移位置提取四个公钥信息：
    /// - coin_mint: 代币A的铸币地址
    /// - pc_mint: 代币B的铸币地址  
    /// - coin_vault: 代币A的资金池地址
    /// - pc_vault: 代币B的资金池地址
    ///
    /// # 参数
    /// * `data` - 包含AMM信息的原始字节数据切片
    ///
    /// # 返回值
    /// * `Result<Self>` - 成功时返回解析出的RaydiumAmmInfo实例，失败时返回错误信息
    ///
    /// # 错误
    /// 当数据长度小于PC_MINT_OFFSET+32时会返回错误
    pub fn load_checked(data: &[u8]) -> Result<Self> {
        // 验证数据长度是否满足最小要求
        if data.len() < PC_MINT_OFFSET + 32 {
            return Err(anyhow::anyhow!("Invalid data length for RaydiumAmmInfo"));
        }

        // 从数据中提取四个公钥信息
        let coin_vault = Pubkey::try_from(&data[COIN_VAULT_OFFSET..COIN_VAULT_OFFSET + 32])?;
        let pc_vault = Pubkey::try_from(&data[PC_VAULT_OFFSET..PC_VAULT_OFFSET + 32])?;
        let coin_mint = Pubkey::try_from(&data[COIN_MINT_OFFSET..COIN_MINT_OFFSET + 32])?;
        let pc_mint = Pubkey::try_from(&data[PC_MINT_OFFSET..PC_MINT_OFFSET + 32])?;

        Ok(Self {
            coin_mint,
            pc_mint,
            coin_vault,
            pc_vault,
        })
    }
}
