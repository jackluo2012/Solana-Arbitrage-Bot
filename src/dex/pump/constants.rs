use solana_program::pubkey::Pubkey;
use std::str::FromStr;

pub const PUMP_PROGRAM_ID: &str = "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA";
pub const PUMP_FEE_WALLET: &str = "JCRGumoE9Qi5BBgULTgdgTLjSgkCMSbF62ZZfGs84JeU";

/// 获取 Pump 程序的公钥标识符
///
/// 该函数用于返回与 Pump 程序关联的唯一公钥标识符。
///
/// # 返回值
/// * `Pubkey` - Pump 程序的公钥
///
/// # Panics
/// 当 `PUMP_PROGRAM_ID` 字符串无法解析为有效公钥时会触发 panic
pub fn pump_program_id() -> Pubkey {
    // 将预定义的程序 ID 字符串转换为 Pubkey 类型
    Pubkey::from_str(PUMP_PROGRAM_ID).unwrap()
}

/// 获取Pump费用钱包的公钥
///
/// 该函数用于获取Pump协议的费用钱包地址，该地址用于接收协议收取的费用
///
/// # 返回值
/// * `Pubkey` - Pump费用钱包的公钥地址
///
/// # Panics
/// 当`PUMP_FEE_WALLET`字符串无法解析为有效公钥时会panic
pub fn pump_fee_wallet() -> Pubkey {
    Pubkey::from_str(PUMP_FEE_WALLET).unwrap()
}
