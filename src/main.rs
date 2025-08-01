mod bot;
mod config;
mod constants;
mod dex;
mod pools;
mod refresh;
mod transaction;

use clap::{App, Arg};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    /// 构建一个格式化日志订阅器
    ///
    /// 该函数创建一个FmtSubscriber实例，用于格式化和输出日志信息。
    /// 通过设置最大日志级别来控制日志输出的详细程度。
    ///
    /// # 参数
    /// 无显式参数
    ///
    /// # 返回值
    /// 返回配置好的FmtSubscriber实例，可用于日志订阅
    ///
    /// # 示例
    /// ```
    /// let subscriber = FmtSubscriber::builder()
    ///         .with_max_level(Level::INFO)
    ///         .finish();
    /// ```
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    // 设置全局默认的 tracing 订阅者
    //
    // 该函数将指定的 subscriber 设置为全局默认的追踪订阅者，
    // 用于收集和处理应用程序中的追踪事件。
    //
    // # 参数
    // * `subscriber` - 要设置为全局默认的追踪订阅者实例
    //
    // # Panics
    // 当设置全局默认订阅者失败时，程序会 panic 并输出错误信息
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    info!("Starting Solana Onchain Bot");

    // 解析命令行参数，配置应用程序的基本信息和参数选项
    // 该函数创建一个命令行应用实例，设置应用名称、版本、作者和描述信息
    // 并定义一个可选的配置文件参数，支持短参数-c和长参数--config
    // 参数说明：
    //   config: 可选的配置文件路径，默认值为"config.toml"
    // 返回值：解析后的命令行参数匹配结果
    let matches = App::new("Solana Onchain Arbitrage Bot")
        .version("0.1.0")
        .author("Cetipo")
        .about("A simplified Solana onchain arbitrage bot")
        .arg(
            Arg::with_name("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true)
                .default_value("config.toml"),
        )
        .get_matches();

    // 获取配置文件路径参数
    let config_path = matches.value_of("config").unwrap();
    // 记录使用的配置文件路径
    info!("Using config file: {}", config_path);

    // 启动机器人服务
    bot::run_bot(config_path).await?;

    Ok(())
}
