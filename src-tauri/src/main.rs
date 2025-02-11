// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use log::info;
use reqwest::Client;
use serde::Serialize;
use std::io::Write;
use std::process::{Command, Stdio};
use std::str;
use std::thread;
use std::time::Duration;
use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};
use tauri::async_runtime::spawn_blocking;
use tokio::task;
use tokio::time::timeout;
// use tauri::event;
use tauri::Window;

struct AppState {
    window_states: Mutex<HashMap<String, bool>>, // 存储每个窗口的总在最上面状态
                                                 // Start of Selection
}

#[tauri::command]
fn set_window_always_on_top(window: Window, always_on_top: bool) {
    window.set_always_on_top(always_on_top).unwrap();
}

fn is_process_running(process_name: &str) -> bool {
    let output = Command::new("pgrep")
        .arg("-f") // 使用 -f 参数匹配完整命令行
        .arg(process_name)
        .output();

    match output {
        Ok(output) => !output.stdout.is_empty(), // 如果 stdout 不为空，则表示进程在运行
        Err(_) => false,                         // 出现错误则返回 false
    }
}

async fn stop_process(process_name: &str) -> Result<(), String> {
    let process_name = process_name.to_string(); // 将 &str 转换为 String
    task::spawn_blocking(move || {
        if is_process_running(&process_name) {
            Command::new(format!("/Applications/iNodeClient/{}", process_name))
                .arg("-k")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| e.to_string())?;

            let mut sec = 0;
            loop {
                if !is_process_running(&process_name) {
                    break;
                }
                thread::sleep(Duration::from_secs(1));
                sec += 1;

                if sec > 10 {
                    Command::new("killall")
                        .arg("-9")
                        .arg(&process_name)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

async fn start_process(process_name: &str, password: &str) -> Result<(), String> {
    let process_name = process_name.to_string(); // 将 &str 转换为 String
    let password = password.to_string(); // 将 &str 转换为 String
    task::spawn_blocking(move || {
        if !is_process_running(&process_name) {
            let mut child = Command::new("sudo")
                .arg("-S") // 从标准输入读取密码
                .arg(format!("/Applications/iNodeClient/{}", process_name))
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| e.to_string())?;

            child
                .stdin
                .as_mut()
                .ok_or("Failed to open stdin".to_string())?
                .write_all(password.as_bytes())
                .map_err(|e| e.to_string())?;

            // 检查进程是否启动成功
            let mut sec = 0;
            loop {
                if is_process_running(&process_name) {
                    break;
                }
                thread::sleep(Duration::from_secs(1));
                sec += 1;

                if sec > 20 {
                    return Err(format!(
                        "启动进程失败： {} （20s 内检测不到进程）",
                        process_name
                    ));
                }
            }
        }
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(target_os = "windows")]
fn windows_network_status() -> NetworkStatus {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut wifi_connected = false;
    let mut ethernet_connected = false;

    for interface in sys.networks().values() {
        if interface.name().to_lowercase().contains("wi-fi")
            || interface.name().to_lowercase().contains("wireless")
        {
            if interface.received() > 0 && interface.transmitted() > 0 {
                wifi_connected = true;
            }
        } else if interface.name().to_lowercase().contains("ethernet")
            || interface
                .name()
                .to_lowercase()
                .contains("local area connection")
        {
            if interface.received() > 0 && interface.transmitted() > 0 {
                ethernet_connected = true;
            }
        }
    }

    if wifi_connected {
        NetworkStatus::Wifi
    } else if ethernet_connected {
        NetworkStatus::Ethernet
    } else {
        NetworkStatus::Disconnected
    }
}

#[cfg(target_os = "macos")]
fn macos_network_status() -> NetworkStatus {
    let services_output = Command::new("networksetup")
        .arg("-listallnetworkservices")
        .output()
        .expect("查询网络状态失败");

    let services_string = String::from_utf8_lossy(&services_output.stdout).into_owned(); // Key change: into_owned()
    let services: Vec<&str> = services_string.lines().skip(1).collect();

    let hardware_ports_output = Command::new("networksetup")
        .arg("-listallhardwareports")
        .output()
        .expect("查询网络状态失败");

    let hardware_ports = String::from_utf8_lossy(&hardware_ports_output.stdout);

    let mut wifi_connected = false;
    let mut ethernet_connected = false;

    for service in services {
        let device = hardware_ports
            .lines()
            .skip_while(|line| !line.contains(service))
            .skip(1)
            .next()
            .and_then(|line| line.split_whitespace().nth(1));

        if let Some(device) = device {
            let ifconfig_output = Command::new("ifconfig").arg(device).output();

            if let Ok(output) = ifconfig_output {
                if String::from_utf8_lossy(&output.stdout).contains("status: active") {
                    if service.to_lowercase().contains("wi-fi") {
                        wifi_connected = true;
                    } else if service.to_lowercase().contains("ethernet")
                        || service.to_lowercase().contains("lan")
                    {
                        ethernet_connected = true;
                    }
                }
            }
        }
    }

    if wifi_connected {
        NetworkStatus::Wifi
    } else if ethernet_connected {
        NetworkStatus::Ethernet
    } else {
        NetworkStatus::Disconnected
    }
}

#[tauri::command]
fn get_network_status() -> NetworkStatus {
    #[cfg(target_os = "macos")]
    {
        macos_network_status()
    }
    #[cfg(target_os = "windows")]
    {
        windows_network_status()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // Fallback for other platforms
        NetworkStatus::Disconnected
    }
}

#[tauri::command]
async fn stop_inode_services() -> Result<String, String> {
    stop_process("iNodeMon").await?;
    stop_process("AuthenMngService").await?;
    Ok("1".to_string())
}

#[tauri::command]
async fn start_inode_services(password: String) -> Result<String, String> {
    start_process("AuthenMngService", &password).await?;
    start_process("iNodeMon", &password).await?;
    Ok("iNodeClient服务启动成功".to_string())
}

// #[tauri::command]
// async fn check_network() -> Result<bool, String> {
//     #[cfg(target_os = "windows")]
//     const PING_COMMAND: &str = "ping";
//     #[cfg(not(target_os = "windows"))]
//     const PING_COMMAND: &str = "ping";

//     #[cfg(target_os = "windows")]
//     const PING_ARGS: [&str; 3] = ["-n", "1", "8.8.8.8"];
//     #[cfg(not(target_os = "windows"))]
//     const PING_ARGS: [&str; 3] = ["-c", "1", "8.8.8.8"];

//     match tokio::time::timeout(
//         std::time::Duration::from_secs(2),
//         spawn_blocking(move || {
//             Command::new(PING_COMMAND)
//                 .args(&PING_ARGS)
//                 .stdout(Stdio::null())
//                 .stderr(Stdio::null())
//                 .status()
//                 .map(|status| status.success())
//                 .unwrap_or(false)
//         }),
//     )
//     .await
//     {
//         Ok(inner_result) => {
//             match inner_result {
//                 Ok(ping_successful) => Ok(ping_successful),
//                 Err(_) => Err("Error checking network: spawn_blocking failed".into()), // Improved error message and use String instead of &str
//             }
//         }
//         Err(_) => Ok(false),
//     }
// }

async fn check_ping() -> bool {
    // 定义多个目标地址
    let targets = ["8.8.8.8", "1.1.1.1"]; // 可以根据需要调整目标

    // 定义重试次数
    const RETRY_COUNT: usize = 3;

    // 遍历目标地址并重试
    for target in targets.iter() {
        for _ in 0..RETRY_COUNT {
            let ping_successful = ping_target(target.to_string()).await;
            if ping_successful {
                return true; // 如果任意一个目标成功，返回 true
            }
        }
    }

    false // 所有目标均失败，返回 false
}

async fn ping_target(target: String) -> bool {
    #[cfg(target_os = "windows")]
    const PING_COMMAND: &str = "ping";
    #[cfg(not(target_os = "windows"))]
    const PING_COMMAND: &str = "ping";

    match timeout(
        Duration::from_secs(2), // 超时时间
        spawn_blocking(move || {
            // 在这里创建 ping_args，确保它的生命周期正确
            let ping_args = {
                #[cfg(target_os = "windows")]
                {
                    vec!["-n", "1", target.as_str()] // 使用 target 的引用
                }
                #[cfg(not(target_os = "windows"))]
                {
                    vec!["-c", "1", target.as_str()] // 使用 target 的引用
                }
            };

            // 使用 Command 执行 ping
            Command::new(PING_COMMAND)
                .args(&ping_args) // 使用动态生成的参数
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|status| status.success())
                .unwrap_or(false)
        }),
    )
    .await
    {
        Ok(inner_result) => match inner_result {
            Ok(ping_successful) => ping_successful,
            Err(_) => false, // 如果 spawn_blocking 失败，返回 false
        },
        Err(_) => false, // 如果超时，返回 false
    }
}

async fn check_http() -> bool {
    let client = Client::new();
    match timeout(
        Duration::from_secs(5), // 超时时间
        client.get("https://www.baidu.com").send(),
    )
    .await
    {
        Ok(response) => match response {
            Ok(resp) => resp.status().is_success(), // 如果 HTTP 请求成功，返回 true
            Err(_) => false,                        // 如果请求失败，返回 false
        },
        Err(_) => false, // 如果超时，返回 false
    }
}

#[tauri::command]
async fn check_network() -> Result<bool, String> {
    // 检查网络连接（Ping 和 HTTP）
    let ping_successful = check_ping().await;
    let http_successful = check_http().await;

    // 如果任意一种方式成功，则认为网络连接正常
    Ok(ping_successful || http_successful)
}

#[derive(Debug, Serialize)]
enum NetworkStatus {
    Wifi,
    Ethernet,
    Disconnected,
}

fn main() {
    let app_state = Arc::new(AppState {
        window_states: Mutex::new(HashMap::new()),
    });

    // 初始化第二个窗口的状态
    {
        let mut states = app_state.window_states.lock().unwrap();
        states.insert("second".to_string(), false); // 默认状态为 false
    }

    env_logger::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        )) // 自动启动插件
        .setup(|_| {
            // 这里可以进行额外的应用初始化设置
            info!("Tauri application initialized!");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_window_always_on_top,
            check_network,
            stop_inode_services,
            start_inode_services,
            get_network_status // 添加新的命令处理器
        ])
        .run(tauri::generate_context!())
        .expect("error while running Sauter application");
}
