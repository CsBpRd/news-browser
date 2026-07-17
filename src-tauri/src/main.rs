// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // CLI 子命令：upgrade-config 将已存在的 ai-config.json 的 max_tokens 升级到 16000
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "upgrade-config" {
        let mut config = app_lib::ai_config::get_ai_config();
        println!("当前配置: endpoint={} model={} max_tokens={}",
                 config.api_endpoint, config.model_name, config.max_tokens);
        if config.max_tokens < 16000 {
            config.max_tokens = 16000;
            match app_lib::ai_config::save_ai_config(config.clone()) {
                Ok(_) => println!("✅ max_tokens 已升级到 {}", config.max_tokens),
                Err(e) => {
                    println!("❌ 升级失败: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            println!("max_tokens 已为 {}，无需升级", config.max_tokens);
        }
        return;
    }

    app_lib::run();
}
