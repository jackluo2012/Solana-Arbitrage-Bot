use crate::{
    constants::SOL_MINT,
    dex::raydium::{clmm_info::POOL_TICK_ARRAY_BITMAP_SEED, raydium_clmm_program_id},
};
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct RaydiumPool {
    pub pool: Pubkey,
    pub token_vault: Pubkey,
    pub sol_vault: Pubkey,
}

#[derive(Debug, Clone)]
pub struct RaydiumCpPool {
    pub pool: Pubkey,
    pub token_vault: Pubkey,
    pub sol_vault: Pubkey,
    pub amm_config: Pubkey,
    pub observation: Pubkey,
}

#[derive(Debug, Clone)]
pub struct PumpPool {
    pub pool: Pubkey,
    pub token_vault: Pubkey,
    pub sol_vault: Pubkey,
    pub fee_token_wallet: Pubkey,
    pub coin_creator_vault_ata: Pubkey,
    pub coin_creator_vault_authority: Pubkey,
}

#[derive(Debug, Clone)]
pub struct DlmmPool {
    pub pair: Pubkey,
    pub token_vault: Pubkey,
    pub sol_vault: Pubkey,
    pub oracle: Pubkey,
    pub bin_arrays: Vec<Pubkey>,
    pub memo_program: Option<Pubkey>, // For Token 2022 support
}

#[derive(Debug, Clone)]
pub struct WhirlpoolPool {
    pub pool: Pubkey,
    pub oracle: Pubkey,
    pub x_vault: Pubkey,
    pub y_vault: Pubkey,
    pub tick_arrays: Vec<Pubkey>,
    pub memo_program: Option<Pubkey>, // For Token 2022 support
}

#[derive(Debug, Clone)]
pub struct RaydiumClmmPool {
    pub pool: Pubkey,
    pub amm_config: Pubkey,
    pub observation_state: Pubkey,
    pub bitmap_extension: Pubkey,
    pub x_vault: Pubkey,
    pub y_vault: Pubkey,
    pub tick_arrays: Vec<Pubkey>,
    pub memo_program: Option<Pubkey>, // For Token 2022 support
}

#[derive(Debug, Clone)]
pub struct MeteoraDAmmPool {
    pub pool: Pubkey,
    pub token_x_vault: Pubkey,
    pub token_sol_vault: Pubkey,
    pub token_x_token_vault: Pubkey,
    pub token_sol_token_vault: Pubkey,
    pub token_x_lp_mint: Pubkey,
    pub token_sol_lp_mint: Pubkey,
    pub token_x_pool_lp: Pubkey,
    pub token_sol_pool_lp: Pubkey,
    pub admin_token_fee_x: Pubkey,
    pub admin_token_fee_sol: Pubkey,
}

#[derive(Debug, Clone)]
pub struct SolfiPool {
    pub pool: Pubkey,
    pub token_x_vault: Pubkey,
    pub token_sol_vault: Pubkey,
}

#[derive(Debug, Clone)]
pub struct MeteoraDAmmV2Pool {
    pub pool: Pubkey,
    pub token_x_vault: Pubkey,
    pub token_sol_vault: Pubkey,
}

#[derive(Debug, Clone)]
pub struct VertigoPool {
    pub pool: Pubkey,
    pub pool_owner: Pubkey,
    pub token_x_vault: Pubkey,
    pub token_sol_vault: Pubkey,
}

#[derive(Debug, Clone)]
/// MintPoolData 结构体用于存储与特定铸币相关的池信息和账户数据
///
/// 该结构体包含了与特定铸币相关的各种去中心化交易所池信息，
/// 以及相关的钱包账户和程序账户信息，支持多种不同的DEX协议
pub struct MintPoolData {
    /// 铸币的公钥地址
    pub mint: Pubkey,
    /// 代币程序的公钥地址，支持Token和Token 2022两种代币标准
    pub token_program: Pubkey, // Support for both Token and Token 2022
    /// 钱包账户的公钥地址
    pub wallet_account: Pubkey,
    /// 钱包WSOL账户的公钥地址，用于处理SOL代币的包装和解包装
    pub wallet_wsol_account: Pubkey,
    /// Raydium协议的池信息列表
    pub raydium_pools: Vec<RaydiumPool>,
    /// Raydium集中流动性池信息列表
    pub raydium_cp_pools: Vec<RaydiumCpPool>,
    /// Pump协议的池信息列表
    pub pump_pools: Vec<PumpPool>,
    /// DLMM协议的池信息列表
    pub dlmm_pairs: Vec<DlmmPool>,
    /// Whirlpool协议的池信息列表
    pub whirlpool_pools: Vec<WhirlpoolPool>,
    /// Raydium CLMM协议的池信息列表
    pub raydium_clmm_pools: Vec<RaydiumClmmPool>,
    /// Meteora DAmm协议的池信息列表
    pub meteora_damm_pools: Vec<MeteoraDAmmPool>,
    /// Solfi协议的池信息列表
    pub solfi_pools: Vec<SolfiPool>,
    /// Meteora DAmm V2协议的池信息列表
    pub meteora_damm_v2_pools: Vec<MeteoraDAmmV2Pool>,
    /// Vertigo协议的池信息列表
    pub vertigo_pools: Vec<VertigoPool>,
}

impl MintPoolData {
    /// 创建一个新的实例
    ///
    /// # 参数
    /// * `mint` - 代币mint地址的字符串表示
    /// * `wallet_account` - 钱包账户地址的字符串表示
    /// * `token_program` - 代币程序的公钥
    ///
    /// # 返回值
    /// 返回Result包装的新实例，如果解析公钥失败则返回错误
    pub fn new(mint: &str, wallet_account: &str, token_program: Pubkey) -> anyhow::Result<Self> {
        // 解析SOL mint地址和钱包地址
        let sol_mint = Pubkey::from_str(SOL_MINT)?;
        let wallet_pk = Pubkey::from_str(wallet_account)?;

        // 计算钱包的WSOL关联代币地址
        let wallet_wsol_pk =
            spl_associated_token_account::get_associated_token_address(&wallet_pk, &sol_mint);

        // 构造并返回新实例，初始化所有池子列表为空
        Ok(Self {
            mint: Pubkey::from_str(mint)?,
            token_program,
            wallet_account: wallet_pk,
            wallet_wsol_account: wallet_wsol_pk,
            raydium_pools: Vec::new(),
            raydium_cp_pools: Vec::new(),
            pump_pools: Vec::new(),
            dlmm_pairs: Vec::new(),
            whirlpool_pools: Vec::new(),
            raydium_clmm_pools: Vec::new(),
            meteora_damm_pools: Vec::new(),
            solfi_pools: Vec::new(),
            meteora_damm_v2_pools: Vec::new(),
            vertigo_pools: Vec::new(),
        })
    }

    pub fn add_raydium_pool(
        &mut self,
        pool: &str,
        token_vault: &str,
        sol_vault: &str,
    ) -> anyhow::Result<()> {
        self.raydium_pools.push(RaydiumPool {
            pool: Pubkey::from_str(pool)?,
            token_vault: Pubkey::from_str(token_vault)?,
            sol_vault: Pubkey::from_str(sol_vault)?,
        });
        Ok(())
    }

    /// 向Raydium集中流动性池列表中添加一个新的池
    ///
    /// 该函数创建一个新的RaydiumCpPool实例并将其添加到内部存储中。
    /// 所有地址参数都必须是有效的Solana公钥字符串格式。
    ///
    /// # 参数
    /// * `pool` - 流动性池的公钥地址字符串
    /// * `token_vault` - 代币资金池的公钥地址字符串
    /// * `sol_vault` - SOL资金池的公钥地址字符串
    /// * `amm_config` - AMM配置账户的公钥地址字符串
    /// * `observation` - 价格观测账户的公钥地址字符串
    ///
    /// # 返回值
    /// 返回Result<(), anyhow::Error>，成功时返回Ok(())，失败时返回包含错误信息的Err
    ///
    /// # 错误
    /// 当任何公钥地址字符串格式无效时，会返回解析错误
    pub fn add_raydium_cp_pool(
        &mut self,
        pool: &str,
        token_vault: &str,
        sol_vault: &str,
        amm_config: &str,
        observation: &str,
    ) -> anyhow::Result<()> {
        // 创建新的Raydium集中流动性池实例并添加到列表中
        self.raydium_cp_pools.push(RaydiumCpPool {
            pool: Pubkey::from_str(pool)?,
            token_vault: Pubkey::from_str(token_vault)?,
            sol_vault: Pubkey::from_str(sol_vault)?,
            amm_config: Pubkey::from_str(amm_config)?,
            observation: Pubkey::from_str(observation)?,
        });
        Ok(())
    }

    /// 向泵池列表中添加一个新的泵池配置
    ///
    /// # 参数
    /// * `pool` - 泵池的公钥地址字符串
    /// * `token_vault` - 代币保险库的公钥地址字符串
    /// * `sol_vault` - SOL保险库的公钥地址字符串
    /// * `fee_token_wallet` - 手续费代币钱包的公钥地址字符串
    /// * `coin_creator_vault_ata` - 代币创建者保险库关联代币账户的公钥地址字符串
    /// * `coin_creator_authority` - 代币创建者权限账户的公钥地址字符串
    ///
    /// # 返回值
    /// 返回Result<(), anyhow::Error>，成功时返回Ok(())，失败时返回错误信息
    pub fn add_pump_pool(
        &mut self,
        pool: &str,
        token_vault: &str,
        sol_vault: &str,
        fee_token_wallet: &str,
        coin_creator_vault_ata: &str,
        coin_creator_authority: &str,
    ) -> anyhow::Result<()> {
        // 创建新的泵池结构体并添加到泵池列表中
        self.pump_pools.push(PumpPool {
            pool: Pubkey::from_str(pool)?,
            token_vault: Pubkey::from_str(token_vault)?,
            sol_vault: Pubkey::from_str(sol_vault)?,
            fee_token_wallet: Pubkey::from_str(fee_token_wallet)?,
            coin_creator_vault_ata: Pubkey::from_str(coin_creator_vault_ata)?,
            coin_creator_vault_authority: Pubkey::from_str(coin_creator_authority)?,
        });
        Ok(())
    }

    pub fn add_dlmm_pool(
        &mut self,
        pair: &str,
        token_vault: &str,
        sol_vault: &str,
        oracle: &str,
        bin_arrays: Vec<&str>,
        memo_program: Option<&str>,
    ) -> anyhow::Result<()> {
        let bin_array_pubkeys = bin_arrays
            .iter()
            .map(|&s| Pubkey::from_str(s))
            .collect::<Result<Vec<_>, _>>()?;

        let memo_program_pubkey = if let Some(memo) = memo_program {
            Some(Pubkey::from_str(memo)?)
        } else {
            None
        };

        self.dlmm_pairs.push(DlmmPool {
            pair: Pubkey::from_str(pair)?,
            token_vault: Pubkey::from_str(token_vault)?,
            sol_vault: Pubkey::from_str(sol_vault)?,
            oracle: Pubkey::from_str(oracle)?,
            bin_arrays: bin_array_pubkeys,
            memo_program: memo_program_pubkey,
        });
        Ok(())
    }

    pub fn add_whirlpool_pool(
        &mut self,
        pool: &str,
        oracle: &str,
        x_vault: &str,
        y_vault: &str,
        tick_arrays: Vec<&str>,
        memo_program: Option<&str>,
    ) -> anyhow::Result<()> {
        let tick_array_pubkeys = tick_arrays
            .iter()
            .map(|&s| Pubkey::from_str(s))
            .collect::<Result<Vec<_>, _>>()?;

        let memo_program_pubkey = if let Some(memo) = memo_program {
            Some(Pubkey::from_str(memo)?)
        } else {
            None
        };

        self.whirlpool_pools.push(WhirlpoolPool {
            pool: Pubkey::from_str(pool)?,
            oracle: Pubkey::from_str(oracle)?,
            x_vault: Pubkey::from_str(x_vault)?,
            y_vault: Pubkey::from_str(y_vault)?,
            tick_arrays: tick_array_pubkeys,
            memo_program: memo_program_pubkey,
        });
        Ok(())
    }

    pub fn add_raydium_clmm_pool(
        &mut self,
        pool: &str,
        amm_config: &str,
        observation_state: &str,
        x_vault: &str,
        y_vault: &str,
        tick_arrays: Vec<&str>,
        memo_program: Option<&str>,
    ) -> anyhow::Result<()> {
        let pool_pubkey = Pubkey::from_str(pool)?;
        let bitmap_extension = Pubkey::find_program_address(
            &[
                POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
                &pool_pubkey.as_ref(),
            ],
            &raydium_clmm_program_id(),
        )
        .0;
        let tick_array_pubkeys = tick_arrays
            .iter()
            .map(|&s| Pubkey::from_str(s))
            .collect::<Result<Vec<_>, _>>()?;

        let memo_program_pubkey = if let Some(memo) = memo_program {
            Some(Pubkey::from_str(memo)?)
        } else {
            None
        };

        self.raydium_clmm_pools.push(RaydiumClmmPool {
            pool: pool_pubkey,
            amm_config: Pubkey::from_str(amm_config)?,
            observation_state: Pubkey::from_str(observation_state)?,
            x_vault: Pubkey::from_str(x_vault)?,
            y_vault: Pubkey::from_str(y_vault)?,
            bitmap_extension,
            tick_arrays: tick_array_pubkeys,
            memo_program: memo_program_pubkey,
        });
        Ok(())
    }

    pub fn add_meteora_damm_pool(
        &mut self,
        pool: &str,
        token_x_vault: &str,
        token_sol_vault: &str,
        token_x_token_vault: &str,
        token_sol_token_vault: &str,
        token_x_lp_mint: &str,
        token_sol_lp_mint: &str,
        token_x_pool_lp: &str,
        token_sol_pool_lp: &str,
        admin_token_fee_x: &str,
        admin_token_fee_sol: &str,
    ) -> anyhow::Result<()> {
        self.meteora_damm_pools.push(MeteoraDAmmPool {
            pool: Pubkey::from_str(pool)?,
            token_x_vault: Pubkey::from_str(token_x_vault)?,
            token_sol_vault: Pubkey::from_str(token_sol_vault)?,
            token_x_token_vault: Pubkey::from_str(token_x_token_vault)?,
            token_sol_token_vault: Pubkey::from_str(token_sol_token_vault)?,
            token_x_lp_mint: Pubkey::from_str(token_x_lp_mint)?,
            token_sol_lp_mint: Pubkey::from_str(token_sol_lp_mint)?,
            token_x_pool_lp: Pubkey::from_str(token_x_pool_lp)?,
            token_sol_pool_lp: Pubkey::from_str(token_sol_pool_lp)?,
            admin_token_fee_x: Pubkey::from_str(admin_token_fee_x)?,
            admin_token_fee_sol: Pubkey::from_str(admin_token_fee_sol)?,
        });
        Ok(())
    }

    pub fn add_solfi_pool(
        &mut self,
        pool: &str,
        token_x_vault: &str,
        token_sol_vault: &str,
    ) -> anyhow::Result<()> {
        self.solfi_pools.push(SolfiPool {
            pool: Pubkey::from_str(pool)?,
            token_x_vault: Pubkey::from_str(token_x_vault)?,
            token_sol_vault: Pubkey::from_str(token_sol_vault)?,
        });
        Ok(())
    }

    pub fn add_meteora_damm_v2_pool(
        &mut self,
        pool: &str,
        token_x_vault: &str,
        token_sol_vault: &str,
    ) -> anyhow::Result<()> {
        self.meteora_damm_v2_pools.push(MeteoraDAmmV2Pool {
            pool: Pubkey::from_str(pool)?,
            token_x_vault: Pubkey::from_str(token_x_vault)?,
            token_sol_vault: Pubkey::from_str(token_sol_vault)?,
        });
        Ok(())
    }

    pub fn add_vertigo_pool(
        &mut self,
        pool: &str,
        pool_owner: &str,
        token_x_vault: &str,
        token_sol_vault: &str,
    ) -> anyhow::Result<()> {
        self.vertigo_pools.push(VertigoPool {
            pool: Pubkey::from_str(pool)?,
            pool_owner: Pubkey::from_str(pool_owner)?,
            token_x_vault: Pubkey::from_str(token_x_vault)?,
            token_sol_vault: Pubkey::from_str(token_sol_vault)?,
        });
        Ok(())
    }
}
